//! One poll over a scripted `CourtListener`: court and citing searches,
//! one bead per cluster with its external ref, the candidate elements, filing, and the client's
//! pacing and rate-limit retries.
#![allow(clippy::unwrap_used, clippy::too_many_lines, clippy::unnecessary_wraps)]

use litigate_feed::bead::{Draft, Filer};
use litigate_feed::courtlistener::{Client, Request, Response, Transport};
use litigate_feed::inventory::{Element, Inventory, Package, Precedent};
use litigate_feed::{default_since, poll, watched, Options};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;

/// Answers by URL substring, in order (the last answer repeats); a fake
/// clock that only sleeping advances.
#[derive(Default)]
struct Script {
    routes: Vec<(String, Vec<Result<Response, String>>)>,
    sent: Vec<Request>,
    slept: Vec<Duration>,
    clock: Duration,
}

impl Script {
    fn on(mut self, needle: &str, answers: Vec<Result<Response, String>>) -> Self {
        self.routes.push((needle.to_owned(), answers));
        self
    }
}

impl Transport for Script {
    fn send(&mut self, request: &Request) -> Result<Response, String> {
        self.sent.push(request.clone());
        let url = match request {
            Request::Get(url) | Request::PostForm(url, _) => url.clone(),
        };
        let answers = &mut self
            .routes
            .iter_mut()
            .find(|(needle, _)| url.contains(needle.as_str()))
            .unwrap_or_else(|| panic!("unscripted {url}"))
            .1;
        if answers.len() > 1 {
            answers.remove(0)
        } else {
            answers[0].clone()
        }
    }

    fn sleep(&mut self, duration: Duration) {
        self.slept.push(duration);
        self.clock += duration;
    }

    fn now(&mut self) -> Duration {
        self.clock
    }
}

fn ok(body: &serde_json::Value) -> Result<Response, String> {
    Ok(Response {
        status: 200,
        retry_after: None,
        body: body.to_string(),
    })
}

fn status(code: u16, retry_after: Option<u64>) -> Result<Response, String> {
    Ok(Response {
        status: code,
        retry_after,
        body: "slow down".to_owned(),
    })
}

fn result(
    cluster: u64,
    opinions: &[u64],
    court: &str,
    date: &str,
    name: &str,
) -> serde_json::Value {
    json!({
        "cluster_id": cluster,
        "opinions": opinions.iter().map(|id| json!({"id": id})).collect::<Vec<_>>(),
        "court_id": court,
        "dateFiled": date,
        "caseName": name,
        "absolute_url": format!("/opinion/{cluster}/x/"),
        "citation": ["100 F.4th 1"],
    })
}

fn package(name: &str, dir: &str, deps: &[&str], kind: &str) -> Package {
    Package {
        dir: dir.to_owned(),
        name: name.to_owned(),
        pack_id: Some(name.to_owned()),
        kind: kind.to_owned(),
        deps: deps.iter().map(|d| (*d).to_owned()).collect(),
        instrument: None,
        source_urls: Vec::new(),
        prefix_groups: vec![vec!["8th Cir. R.".to_owned()]],
    }
}

fn inventory(precedents: Vec<Precedent>) -> Inventory {
    Inventory {
        packages: vec![
            package("ca8", "lit/ca8", &["lit/frap"], "base"),
            package("frap", "lit/frap", &[], "base"),
            package("bowles", "lit/bowles", &["lit/frap"], "precedent"),
            package("orphan", "lit/orphan", &[], "precedent"),
            package("cited", "lit/cited", &[], "precedent"),
        ],
        courts: BTreeMap::from([("ca8".to_owned(), vec!["ca8".to_owned()])]),
        precedents,
    }
}

fn element(package: &str, id: &str, label: &str, cite: &str) -> Element {
    Element {
        package: package.to_owned(),
        id: id.to_owned(),
        label: label.to_owned(),
        cite: Some(cite.to_owned()),
    }
}

fn elements() -> BTreeMap<String, Vec<Element>> {
    BTreeMap::from([
        (
            "ca8".to_owned(),
            vec![element(
                "ca8",
                "briefing",
                "Briefing",
                "FRAP 31(a)(1); 8th Cir. R. 28A",
            )],
        ),
        (
            "frap".to_owned(),
            vec![element(
                "frap",
                "notice-of-appeal-due",
                "Notice of appeal due",
                "Fed. R. App. P. 4(a)(1)(A)",
            )],
        ),
    ])
}

fn options(max: usize) -> Options {
    Options {
        since: "2026-10-26".to_owned(),
        max_opinions: max,
    }
}

#[derive(Default)]
struct Recorder {
    filed: Vec<Draft>,
    refuse: Option<String>,
}

impl Filer for Recorder {
    fn file(&mut self, draft: &Draft) -> Result<(), String> {
        if self.refuse.as_deref() == Some(draft.court.as_str()) {
            return Err("store refused".to_owned());
        }
        self.filed.push(draft.clone());
        Ok(())
    }
}

#[test]
fn a_court_opinion_and_a_citing_opinion_are_filed_once_each_with_their_candidates() {
    let precedents = vec![
        Precedent {
            package: "bowles".to_owned(),
            citation: Some("551 U.S. 205".to_owned()),
            cluster: None,
        },
        Precedent {
            package: "cited".to_owned(),
            citation: None,
            cluster: Some(42),
        },
        Precedent {
            package: "orphan".to_owned(),
            citation: None,
            cluster: None,
        },
    ];
    let inventory = inventory(precedents);
    let script = Script::default()
        .on(
            "court=ca8",
            vec![ok(&json!({
                "next": "https://cl.test/api/rest/v4/search/?page=2",
                "results": [result(1, &[11], "ca8", "2026-10-26", "Doe v. Roe"), {"caseName": "no cluster"}],
            }))],
        )
        .on(
            "page=2",
            vec![ok(&json!({"results": [result(3, &[31], "ca9", "2026-10-26", "Elsewhere v. Court")]}))],
        )
        .on(
            "cites%3A%28501+OR+502%29",
            vec![ok(&json!({"results": [
                result(1, &[11], "ca8", "2026-10-26", "Doe v. Roe"),
                result(2, &[21, 22], "cadc", "2026-10-27", "Poe v. Moe"),
            ]}))],
        )
        .on("cites%3A%28601%29", vec![ok(&json!({"results": []}))])
        .on(
            "citation-lookup",
            vec![ok(&json!([{"clusters": [{"sub_opinions": [
                "https://cl.test/api/rest/v4/opinions/501/",
                "https://cl.test/api/rest/v4/opinions/502/"
            ]}]}]))],
        )
        .on(
            "clusters/42/",
            vec![ok(&json!({"sub_opinions": ["https://cl.test/api/rest/v4/opinions/601/"]}))],
        )
        .on(
            "opinions/11/",
            vec![ok(&json!({"plain_text": "The notice of appeal due under Fed. R. App. P. 4(a)(1)(A) was missed; 8th Cir. R. 28A(h)."}))],
        )
        .on("opinions/21/", vec![ok(&json!({"plain_text": "", "html_with_citations": "<p>Bowles &amp; FRAP 4(a)</p>"}))])
        .on("opinions/22/", vec![ok(&json!({"plain_text": "Dissent."}))]);
    let mut client =
        Client::new(script, "https://cl.test/").min_interval(Duration::from_millis(500));
    let mut filer = Recorder::default();
    let report = poll(
        &inventory,
        &elements(),
        &mut client,
        Some(&mut filer),
        &options(10),
    )
    .unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.beads.len(), 2);
    assert_eq!(filer.filed, report.beads);
    let doe = &report.beads[0];
    assert_eq!(doe.court, "ca8");
    assert_eq!(doe.url, "https://www.courtlistener.com/opinion/1/x/");
    assert_eq!(
        doe.why,
        ["from ca8, modeled by ca8", "cites precedent layer bowles"]
    );
    let found: Vec<(&str, &str)> = doe
        .candidates
        .iter()
        .map(|c| (c.package.as_str(), c.element.as_str()))
        .collect();
    assert_eq!(
        found,
        [("ca8", "briefing"), ("frap", "notice-of-appeal-due")]
    );
    assert!(
        doe.description.contains("cite 8thcirr 28A(h)"),
        "{}",
        doe.description
    );
    assert!(doe.description.contains("name \"Notice of appeal due\""));
    assert!(doe.labels.contains(&"pkg:frap".to_owned()));
    let poe = &report.beads[1];
    assert_eq!(poe.opinions, [21, 22]);
    assert_eq!(doe.external_ref, "courtlistener:opinion:11");
    assert_eq!(poe.external_ref, "courtlistener:opinion:21");
    assert_eq!(poe.candidates.len(), 1, "{:?}", poe.candidates);
    assert_eq!(report.precedents["bowles"], [501, 502]);
    assert_eq!(report.precedents["cited"], [601]);
    assert!(report.warnings[0].contains("`orphan`"));
    assert_eq!(report.requests, 9);
    // Requests after the first are paced: the fake clock never moves on its
    // own, so each waits the whole interval.
    assert_eq!(
        client.transport().slept,
        vec![Duration::from_millis(500); 8]
    );

    // The next poll over the same window drafts the same beads with the
    // same refs: the store dedupes on them.
    let mut again = Recorder::default();
    let report = poll(
        &inventory,
        &elements(),
        &mut client,
        Some(&mut again),
        &options(10),
    )
    .unwrap();
    let refs =
        |beads: &[Draft]| -> Vec<String> { beads.iter().map(|b| b.external_ref.clone()).collect() };
    assert_eq!(refs(&again.filed), refs(&filer.filed));
    assert_eq!(report.beads, filer.filed);
}

#[test]
fn deferred_failed_refused_and_dry_runs() {
    let inventory = inventory(Vec::new());
    let script = || {
        Script::default()
            .on(
                "court=ca8",
                vec![ok(&json!({"results": [
                    result(1, &[11], "ca8", "2026-10-26", "A"),
                    result(2, &[21], "ca8", "2026-10-26", "B"),
                ]}))],
            )
            .on("opinions/11/", vec![ok(&json!({"plain_text": "nothing"}))])
            .on("opinions/21/", vec![Err("connection reset".to_owned())])
    };
    let mut filer = Recorder::default();
    let mut client = Client::new(script(), "https://cl.test");
    let report = poll(
        &inventory,
        &elements(),
        &mut client,
        Some(&mut filer),
        &options(1),
    )
    .unwrap();
    assert_eq!((report.beads.len(), report.deferred), (1, 1));
    assert!(report.beads[0].title.ends_with("0 candidate elements"));
    assert!(report.beads[0].description.contains("none matched"));

    let mut client = Client::new(script(), "https://cl.test");
    let report = poll(
        &inventory,
        &elements(),
        &mut client,
        Some(&mut filer),
        &options(5),
    )
    .unwrap();
    assert_eq!(report.errors, ["cluster 2: connection reset"]);

    let mut refusing = Recorder {
        refuse: Some("ca8".to_owned()),
        ..Recorder::default()
    };
    let mut client = Client::new(script(), "https://cl.test");
    let report = poll(
        &inventory,
        &elements(),
        &mut client,
        Some(&mut refusing),
        &options(5),
    )
    .unwrap();
    assert_eq!(report.errors[0], "cluster 1: store refused");

    let mut client = Client::new(script(), "https://cl.test");
    let report = poll(&inventory, &elements(), &mut client, None, &options(5)).unwrap();
    assert!(!report.filed && report.beads.len() == 1);
}

#[test]
fn an_empty_inventory_sends_nothing() {
    let empty = Inventory {
        packages: Vec::new(),
        courts: BTreeMap::new(),
        precedents: Vec::new(),
    };
    let mut client = Client::new(Script::default(), "https://cl.test");
    let report = poll(&empty, &BTreeMap::new(), &mut client, None, &options(5)).unwrap();
    assert_eq!(report.requests, 0);
    assert!(watched(&empty).is_empty());
}

#[test]
fn rate_limits_wait_retry_after_then_a_doubling_backoff_and_give_up() {
    let script = Script::default()
        .on(
            "test/a",
            vec![
                status(429, Some(3)),
                status(503, None),
                status(429, None),
                ok(&json!({"x": 1})),
            ],
        )
        .on("test/b", vec![status(429, None)])
        .on("test/c", vec![status(500, None)])
        .on(
            "test/d",
            vec![Ok(Response {
                status: 200,
                retry_after: None,
                body: "<html>".to_owned(),
            })],
        )
        .on("test/e", vec![Err("dns".to_owned())]);
    let mut client = Client::new(script, "https://cl.test")
        .min_interval(Duration::ZERO)
        .max_retries(3);
    assert_eq!(
        client
            .json(&Request::Get("https://cl.test/a".to_owned()))
            .unwrap()["x"],
        1
    );
    assert_eq!(
        client.transport().slept,
        [
            Duration::from_secs(3),
            Duration::from_secs(30),
            Duration::from_mins(1)
        ]
    );
    let err = client
        .json(&Request::Get("https://cl.test/b".to_owned()))
        .unwrap_err();
    assert!(err.contains("HTTP 429: slow down"), "{err}");
    let err = client
        .json(&Request::Get("https://cl.test/c".to_owned()))
        .unwrap_err();
    assert!(err.contains("HTTP 500"), "{err}");
    let err = client
        .json(&Request::PostForm(
            "https://cl.test/d".to_owned(),
            Vec::new(),
        ))
        .unwrap_err();
    assert!(err.contains("not JSON"), "{err}");
    assert_eq!(
        client
            .json(&Request::Get("https://cl.test/e".to_owned()))
            .unwrap_err(),
        "dns"
    );
}

#[test]
fn a_failed_search_fails_the_poll() {
    let inventory = inventory(vec![Precedent {
        package: "bowles".to_owned(),
        citation: Some("551 U.S. 205".to_owned()),
        cluster: None,
    }]);
    let script = Script::default()
        .on("court=ca8", vec![ok(&json!({"results": []}))])
        .on("citation-lookup", vec![status(400, None)]);
    let mut client = Client::new(script, "https://cl.test").min_interval(Duration::ZERO);
    let err = poll(&inventory, &elements(), &mut client, None, &options(5)).unwrap_err();
    assert!(err.contains("citation-lookup"), "{err}");
}

#[test]
fn polls_start_yesterday() {
    assert_eq!(default_since("2026-10-27"), "2026-10-26");
    assert_eq!(default_since("2026-03-01"), "2026-02-28");
    assert_eq!(default_since("garbled"), "garbled");
}
