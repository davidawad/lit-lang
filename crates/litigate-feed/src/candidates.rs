//! The elements an opinion may edit: those whose cite the opinion cites
//! (or a subdivision of it, or a provision containing it), and those whose
//! name it uses.

use crate::cite::Extractor;
use crate::inventory::Element;
use serde::Serialize;

/// Words too common to make a name distinctive.
const STOPWORDS: [&str; 16] = [
    "a", "an", "and", "as", "at", "be", "by", "for", "in", "is", "of", "on", "or", "the", "to",
    "with",
];

/// An element the opinion may edit, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// The package's name.
    pub package: String,
    /// The element (node) id.
    pub element: String,
    /// Its label.
    pub label: String,
    /// Its cite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// What matched: `cite <provision>` per overlapping cite, `name "<label>"`.
    pub reasons: Vec<String>,
}

/// The candidates among `elements` for an opinion whose text is `text`, in
/// `elements` order.
#[must_use]
pub fn candidates(text: &str, elements: &[&Element], extractor: &Extractor) -> Vec<Candidate> {
    let cited = extractor.extract(text);
    let prose = format!(" {} ", words(text));
    let mut out = Vec::new();
    for element in elements {
        let own = element
            .cite
            .as_deref()
            .map(|c| extractor.extract(c))
            .unwrap_or_default();
        let mut reasons: Vec<String> = Vec::new();
        for hit in cited.iter().filter(|c| own.iter().any(|o| o.overlaps(c))) {
            let reason = format!("cite {hit}");
            if !reasons.contains(&reason) {
                reasons.push(reason);
            }
        }
        let name = words(&element.label);
        if distinctive(&name) && prose.contains(&format!(" {name} ")) {
            reasons.push(format!("name \"{}\"", element.label));
        }
        if !reasons.is_empty() {
            out.push(Candidate {
                package: element.package.clone(),
                element: element.id.clone(),
                label: element.label.clone(),
                cite: element.cite.clone(),
                reasons,
            });
        }
    }
    out
}

/// Lowercased words of letters and digits, one space apart.
fn words(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

/// At least two words that are not stopwords or numbers.
fn distinctive(name: &str) -> bool {
    name.split(' ')
        .filter(|w| !STOPWORDS.contains(w) && !w.chars().all(|c| c.is_ascii_digit()))
        .count()
        >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(id: &str, label: &str, cite: Option<&str>) -> Element {
        Element {
            package: "ca8-court-of-appeals".to_owned(),
            id: id.to_owned(),
            label: label.to_owned(),
            cite: cite.map(str::to_owned),
        }
    }

    #[test]
    fn cites_and_names_make_candidates_with_their_reasons() {
        let elements = [
            element(
                "notice-filed",
                "Notice of appeal filed",
                Some("FRAP 3(a)(1); FRAP 4(a)"),
            ),
            element("briefing", "Briefing", Some("FRAP 31(a)(1)")),
            element(
                "rehearing",
                "Petition for rehearing en banc",
                Some("FRAP 40"),
            ),
            element("uncited", "Mandate issued", None),
        ];
        let refs: Vec<&Element> = elements.iter().collect();
        let text = "The notice of appeal filed on day 31 was untimely under Fed. R. App. P. \
                    4(a)(1)(A) and FRAP 4(a)(1)(A); see also FRAP 31(b). The mandate issued.";
        let found = candidates(text, &refs, &Extractor::new(&[]));
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].element, "notice-filed");
        assert_eq!(
            found[0].reasons,
            ["cite frap 4(a)(1)(A)", "name \"Notice of appeal filed\""]
        );
        assert_eq!(found[1].element, "uncited");
        assert_eq!(found[1].reasons, ["name \"Mandate issued\""]);
        assert_eq!(found[1].cite, None);
    }

    #[test]
    fn a_name_needs_two_distinctive_words() {
        assert!(distinctive("notice of appeal"));
        assert!(!distinctive("briefing"));
        assert!(!distinctive("rule 4 of the"));
    }
}
