//! The `ureq` transport against a local server: the token header, form
//! posts, `Retry-After`, and connection failures.
#![allow(clippy::unwrap_used, clippy::too_many_lines, clippy::unnecessary_wraps)]

mod support;

use litigate_feed::courtlistener::{Request, Transport};
use litigate_feed::http::Ureq;
use std::time::Duration;
use support::{ok, serve};

#[test]
fn gets_send_the_token_and_read_status_body_and_retry_after() {
    let server = serve(vec![
        (
            "/limited",
            vec![(
                429,
                vec![("Retry-After".to_owned(), "7".to_owned())],
                "{}".to_owned(),
            )],
        ),
        ("/", vec![ok(r#"{"ok":true}"#)]),
    ]);
    let mut transport = Ureq::new(Some("secret".to_owned()));
    let got = transport
        .send(&Request::Get(format!("{}/api", server.base)))
        .unwrap();
    assert_eq!((got.status, got.retry_after), (200, None));
    assert_eq!(got.body, r#"{"ok":true}"#);
    let limited = transport
        .send(&Request::Get(format!("{}/limited", server.base)))
        .unwrap();
    assert_eq!((limited.status, limited.retry_after), (429, Some(7)));
    let seen = server.seen.lock().unwrap().clone();
    assert!(seen[0]
        .headers
        .contains(&("authorization".to_owned(), "Token secret".to_owned())));
}

#[test]
fn forms_post_without_a_token_when_none_is_given() {
    let server = serve(vec![("/", vec![ok("[]")])]);
    let mut transport = Ureq::new(Some(String::new()));
    let form = vec![("text".to_owned(), "551 U.S. 205".to_owned())];
    let got = transport
        .send(&Request::PostForm(format!("{}/lookup/", server.base), form))
        .unwrap();
    assert_eq!(got.status, 200);
    let seen = server.seen.lock().unwrap().clone();
    assert_eq!(seen[0].method, "POST");
    assert_eq!(seen[0].body, "text=551+U.S.+205");
    assert!(seen[0].headers.iter().all(|(k, _)| k != "authorization"));
    let mut tokened = Ureq::new(Some("t".to_owned()));
    let form = vec![("text".to_owned(), "x".to_owned())];
    tokened
        .send(&Request::PostForm(format!("{}/lookup/", server.base), form))
        .unwrap();
    let seen = server.seen.lock().unwrap().clone();
    assert!(seen[1]
        .headers
        .contains(&("authorization".to_owned(), "Token t".to_owned())));
}

#[test]
fn a_refused_connection_is_an_error_and_time_moves_on() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let mut transport = Ureq::new(None);
    assert!(transport
        .send(&Request::Get(format!("http://{addr}/")))
        .is_err());
    let before = transport.now();
    transport.sleep(Duration::from_millis(5));
    assert!(transport.now() >= before + Duration::from_millis(5));
}
