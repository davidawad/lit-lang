//! A local HTTP server answering canned responses by path prefix, and
//! recording what it was sent: `CourtListener` for the tests.
#![allow(dead_code, clippy::unwrap_used)]

use std::fmt::Write as _;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// A request the server received.
#[derive(Debug, Clone)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// A canned response: status, extra headers, body.
pub type Canned = (u16, Vec<(String, String)>, String);

/// A running server.
pub struct Server {
    pub base: String,
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

/// Serves `routes` (path prefix -> responses, the last one repeating) on a
/// free local port until the test exits.
pub fn serve(routes: Vec<(&str, Vec<Canned>)>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let mut routes: Vec<(String, Vec<Canned>)> =
        routes.into_iter().map(|(p, r)| (p.to_owned(), r)).collect();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                continue;
            }
            let mut parts = line.split_whitespace();
            let method = parts.next().unwrap_or_default().to_owned();
            let path = parts.next().unwrap_or_default().to_owned();
            let mut headers = Vec::new();
            let mut length = 0;
            loop {
                let mut h = String::new();
                reader.read_line(&mut h).unwrap();
                let h = h.trim_end();
                if h.is_empty() {
                    break;
                }
                if let Some((k, v)) = h.split_once(':') {
                    let (k, v) = (k.trim().to_ascii_lowercase(), v.trim().to_owned());
                    if k == "content-length" {
                        length = v.parse().unwrap_or(0);
                    }
                    headers.push((k, v));
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            log.lock().unwrap().push(Seen {
                method,
                path: path.clone(),
                headers,
                body: String::from_utf8_lossy(&body).into_owned(),
            });
            let (status, extra, text) = routes
                .iter_mut()
                .find(|(prefix, _)| path.starts_with(prefix.as_str()))
                .map_or((404, Vec::new(), "{}".to_owned()), |(_, responses)| {
                    if responses.len() > 1 {
                        responses.remove(0)
                    } else {
                        responses[0].clone()
                    }
                });
            let mut head = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n",
                text.len()
            );
            for (k, v) in extra {
                let _ = write!(head, "{k}: {v}\r\n");
            }
            head.push_str("\r\n");
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(text.as_bytes());
        }
    });
    Server { base, seen }
}

/// A 200 with `body`.
pub fn ok(body: &str) -> Canned {
    (200, Vec::new(), body.to_owned())
}
