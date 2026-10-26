use super::*;
use serde_json::json;

fn forms() -> Vec<Form> {
    let document = || {
        (
            "document".to_owned(),
            Param::Enum(vec!["complaint".to_owned(), "amended_complaint".to_owned()]),
        )
    };
    vec![
        Form {
            name: "Answered".to_owned(),
            params: vec![document()],
        },
        Form {
            name: "MovedToDismiss".to_owned(),
            params: vec![document()],
        },
        Form {
            name: "Hearing".to_owned(),
            params: vec![],
        },
        Form {
            name: "other::OrderEntered".to_owned(),
            params: vec![("until".to_owned(), Param::Other("Date".to_owned()))],
        },
    ]
}

fn parties() -> Vec<Party> {
    vec![
        Party {
            id: "acme".to_owned(),
            actor: "Plaintiff".to_owned(),
        },
        Party {
            id: "widget".to_owned(),
            actor: "Defendant".to_owned(),
        },
    ]
}

fn verdict(text: &str) -> Verdict {
    judge(text, &forms(), &parties()).1
}

#[test]
fn words_split_names_and_stems_drop_inflection() {
    assert_eq!(
        words("MovedToDismiss amended_complaint, ECF 12"),
        [
            "moved",
            "to",
            "dismiss",
            "amended",
            "complaint",
            "ecf",
            "12"
        ]
    );
    assert_eq!(stem("Answered".to_lowercase().as_str()), "answer");
    assert_eq!(stem("hearing"), "hear");
    assert_eq!(stem("is"), "is");
    assert_eq!(stems("ANSWER to the Complaint"), stems("answers complaint"));
}

#[test]
fn an_exact_match_is_supported() {
    assert_eq!(
        verdict("ANSWER to Complaint by Defendant Widget Corp."),
        Verdict::Supported("Answered(complaint)".to_owned(), "by", "widget".to_owned())
    );
    // The most specific variant present settles the argument.
    assert_eq!(
        verdict("ANSWER to Amended Complaint upon acme by widget"),
        Verdict::Ambiguous("more than one party fits: by widget, on acme".to_owned())
    );
    assert_eq!(
        verdict("ANSWER to Amended Complaint by widget"),
        Verdict::Supported(
            "Answered(amended_complaint)".to_owned(),
            "by",
            "widget".to_owned()
        )
    );
}

#[test]
fn two_forms_that_fit_are_ambiguous() {
    assert_eq!(
        verdict("ANSWER to Complaint by widget; hearing held"),
        Verdict::Ambiguous("more than one event form fits: Answered, Hearing".to_owned())
    );
}

#[test]
fn an_entry_naming_no_form_is_unmatched() {
    let (candidates, v) = judge("NOTICE of Appearance by widget", &forms(), &parties());
    assert_eq!(v, Verdict::Unmatched);
    assert!(candidates.is_empty());
    let (candidates, v) = judge("MOTION to Dismiss by widget", &forms(), &parties());
    assert_eq!(v, Verdict::Unmatched);
    assert_eq!(candidates[0].event, "MovedToDismiss");
    assert!((candidates[0].confidence - 0.5).abs() < f64::EPSILON);
    assert_eq!(candidates[0].matched, ["dismis"]);
    assert_eq!(candidates[0].missing, ["mov"]);
}

#[test]
fn an_open_argument_or_party_leaves_the_entry_incomplete() {
    assert_eq!(
        verdict("ANSWER by widget"),
        Verdict::Incomplete(
            "argument `document`: none of complaint, amended_complaint is named".to_owned()
        )
    );
    assert_eq!(
        verdict("Order entered by the court"),
        Verdict::Incomplete("argument `until`: a `Date` is not read from docket text".to_owned())
    );
    assert_eq!(
        verdict("Hearing held"),
        Verdict::Incomplete(
            "no party of the matter is named after `by`, `on` or `upon`".to_owned()
        )
    );
    assert_eq!(
        verdict("Hearing on motion by Defendant"),
        Verdict::Supported("Hearing()".to_owned(), "by", "widget".to_owned())
    );
}

#[test]
fn a_variant_that_does_not_include_the_others_is_ambiguous() {
    let forms = vec![Form {
        name: "Served".to_owned(),
        params: vec![(
            "method".to_owned(),
            Param::Enum(vec![
                "Mail".to_owned(),
                "Personal".to_owned(),
                String::new(),
            ]),
        )],
    }];
    assert_eq!(
        judge(
            "Served by mail and personal delivery on widget",
            &forms,
            &parties()
        )
        .1,
        Verdict::Ambiguous(
            "ambiguous argument `method`: more than one of Mail, Personal,  fits".to_owned()
        )
    );
    let unnamed = vec![Form {
        name: "To".to_owned(),
        params: vec![],
    }];
    let (candidates, v) = judge("to", &unnamed, &parties());
    assert!(candidates.is_empty());
    assert_eq!(v, Verdict::Unmatched);
    assert!(candidate(&unnamed[0], &BTreeSet::new()).confidence.abs() < f64::EPSILON);
}

#[test]
fn entries_are_read_from_an_array_or_a_recap_export() {
    let entry = json!({"entry_number": 5, "date_filed": "2025-03-07", "description": "ANSWER", "recap_documents": []});
    for doc in [
        json!([entry]),
        json!({"docket_entries": [entry]}),
        json!({"results": [entry]}),
    ] {
        assert_eq!(
            entries(&doc),
            Ok(vec![Entry {
                source: entry.clone(),
                number: Some(5),
                date: Some("2025-03-07".to_owned()),
                text: "ANSWER".to_owned(),
            }])
        );
    }
    assert_eq!(
        entries(&json!([{}])).map(|e| (e[0].number, e[0].date.clone(), e[0].text.clone())),
        Ok((None, None, String::new()))
    );
    assert!(entries(&json!({"count": 1})).is_err());
    assert_eq!(
        entries(&json!([1])),
        Err("docket entry 0 is not an object".to_owned())
    );
}

#[test]
fn the_most_specific_form_that_fits_is_supported() {
    let form = |name: &str| Form {
        name: name.to_owned(),
        params: vec![],
    };
    let forms = vec![
        form("Filed"),
        form("ResponseFiled"),
        form("PreliminaryResponseFiled"),
        form("other::Filed"),
    ];
    let judged = |text| judge(text, &forms, &parties()).1;
    assert_eq!(
        judged("PRELIMINARY RESPONSE filed by widget"),
        Verdict::Supported(
            "PreliminaryResponseFiled()".to_owned(),
            "by",
            "widget".to_owned()
        )
    );
    // Two forms with the same words: neither is the more specific.
    assert_eq!(
        judged("Paper filed by widget"),
        Verdict::Ambiguous("more than one event form fits: Filed, other::Filed".to_owned())
    );
    assert_eq!(most_specific(&[]), None);
}
