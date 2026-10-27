//! The real [`Transport`]: HTTPS through `ureq`, wall-clock time.

use crate::courtlistener::{Request, Response, Transport};
use std::time::{Duration, Instant};

/// `ureq` with an optional `CourtListener` API token.
pub struct Ureq {
    agent: ureq::Agent,
    token: Option<String>,
    start: Instant,
}

impl Ureq {
    /// A transport sending `Authorization: Token <token>` when given.
    #[must_use]
    pub fn new(token: Option<String>) -> Self {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_mins(2)))
            .user_agent(concat!("litigate-feed/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        Self {
            agent,
            token: token.filter(|t| !t.is_empty()),
            start: Instant::now(),
        }
    }
}

impl Transport for Ureq {
    fn send(&mut self, request: &Request) -> Result<Response, String> {
        let auth = self.token.as_ref().map(|t| format!("Token {t}"));
        let sent = match request {
            Request::Get(url) => {
                let mut req = self.agent.get(url);
                if let Some(auth) = &auth {
                    req = req.header("Authorization", auth);
                }
                req.call()
            }
            Request::PostForm(url, form) => {
                let mut req = self.agent.post(url);
                if let Some(auth) = &auth {
                    req = req.header("Authorization", auth);
                }
                req.send_form(form.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            }
        };
        let mut response = sent.map_err(|e| e.to_string())?;
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok());
        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|e| e.to_string())?;
        Ok(Response {
            status: response.status().as_u16(),
            retry_after,
            body,
        })
    }

    fn sleep(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn now(&mut self) -> Duration {
        self.start.elapsed()
    }
}
