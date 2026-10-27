//! The `CourtListener` REST API (v4): opinion search by court and by cited
//! opinion, opinion text, and citation lookup, over a [`Transport`] the
//! client paces. Rate limits are respected: requests are at least
//! `min_interval` apart, and a 429 or 503 waits its `Retry-After` (else a
//! doubling backoff) before retrying, up to `max_retries` times.

use serde_json::Value;
use std::time::Duration;

/// The public `CourtListener` host.
pub const BASE_URL: &str = "https://www.courtlistener.com";

/// An HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// `GET url`.
    Get(String),
    /// `POST url` with a form body.
    PostForm(String, Vec<(String, String)>),
}

/// An HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The status code.
    pub status: u16,
    /// `Retry-After`, in seconds, when given.
    pub retry_after: Option<u64>,
    /// The body.
    pub body: String,
}

/// Sends requests and keeps time.
pub trait Transport {
    /// Sends `request`.
    ///
    /// # Errors
    /// A connection or I/O failure (not an HTTP error status).
    fn send(&mut self, request: &Request) -> Result<Response, String>;
    /// Waits `duration`.
    fn sleep(&mut self, duration: Duration);
    /// Monotonic time since some fixed instant.
    fn now(&mut self) -> Duration;
}

/// One search result: an opinion cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The cluster id.
    pub cluster: u64,
    /// Its opinions' ids (lead first).
    pub opinions: Vec<u64>,
    /// The case name.
    pub case_name: String,
    /// The `CourtListener` court id.
    pub court: String,
    /// The date filed (`YYYY-MM-DD`).
    pub date: String,
    /// The opinion page URL.
    pub url: String,
    /// Its reporter citations.
    pub citations: Vec<String>,
}

/// A paced `CourtListener` client.
pub struct Client<T> {
    transport: T,
    base: String,
    min_interval: Duration,
    max_retries: u32,
    last: Option<Duration>,
    /// Requests sent, retries included.
    pub requests: u32,
}

impl<T: Transport> Client<T> {
    /// A client of `base` over `transport`.
    pub fn new(transport: T, base: &str) -> Self {
        Self {
            transport,
            base: base.trim_end_matches('/').to_owned(),
            min_interval: Duration::from_secs(1),
            max_retries: 4,
            last: None,
            requests: 0,
        }
    }

    /// Sets the least time between requests.
    #[must_use]
    pub fn min_interval(mut self, interval: Duration) -> Self {
        self.min_interval = interval;
        self
    }

    /// Sets how many times a rate-limited request is retried.
    #[must_use]
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// The transport.
    pub fn transport(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Sends `request`, paced, retrying a rate limit; its JSON body.
    ///
    /// # Errors
    /// A transport failure, a status other than 2xx after the retries, or
    /// a body that is not JSON.
    pub fn json(&mut self, request: &Request) -> Result<Value, String> {
        let mut backoff = Duration::from_secs(30);
        let mut attempt = 0;
        loop {
            if let Some(last) = self.last {
                let since = self.transport.now().saturating_sub(last);
                if since < self.min_interval {
                    self.transport
                        .sleep(self.min_interval.saturating_sub(since));
                }
            }
            self.requests += 1;
            let response = self.transport.send(request);
            self.last = Some(self.transport.now());
            let response = response?;
            match response.status {
                200..=299 => {
                    return serde_json::from_str(&response.body)
                        .map_err(|e| format!("{}: not JSON: {e}", url_of(request)))
                }
                429 | 503 if attempt < self.max_retries => {
                    let wait = response.retry_after.map_or_else(
                        || {
                            backoff *= 2;
                            backoff / 2
                        },
                        Duration::from_secs,
                    );
                    self.transport.sleep(wait);
                    attempt += 1;
                }
                status => {
                    let body: String = response.body.chars().take(200).collect();
                    return Err(format!("{}: HTTP {status}: {body}", url_of(request)));
                }
            }
        }
    }

    /// Opinions filed on or after `since` matching `query` (search API
    /// parameters, e.g. `court`, `q`), following every page.
    ///
    /// # Errors
    /// As [`Client::json`].
    pub fn search(&mut self, query: &[(&str, &str)], since: &str) -> Result<Vec<Hit>, String> {
        let mut params: Vec<(&str, &str)> = vec![("type", "o"), ("order_by", "dateFiled asc")];
        params.extend_from_slice(query);
        params.push(("filed_after", since));
        let mut url = format!("{}/api/rest/v4/search/?{}", self.base, encode(&params));
        let mut hits = Vec::new();
        loop {
            let page = self.json(&Request::Get(url.clone()))?;
            hits.extend(
                page["results"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(hit),
            );
            match page["next"].as_str() {
                Some(next) => next.clone_into(&mut url),
                None => return Ok(hits),
            }
        }
    }

    /// The text of opinion `id`: its plain text, else its HTML with the
    /// tags dropped.
    ///
    /// # Errors
    /// As [`Client::json`].
    pub fn opinion_text(&mut self, id: u64) -> Result<String, String> {
        let url = format!(
            "{}/api/rest/v4/opinions/{id}/?fields=plain_text,html_with_citations,html",
            self.base
        );
        let opinion = self.json(&Request::Get(url))?;
        let text = ["plain_text", "html_with_citations", "html"]
            .iter()
            .filter_map(|field| opinion[field].as_str())
            .find(|text| !text.trim().is_empty())
            .unwrap_or_default();
        Ok(if text.contains('<') {
            strip_tags(text)
        } else {
            text.to_owned()
        })
    }

    /// The opinion ids of cluster `id`.
    ///
    /// # Errors
    /// As [`Client::json`].
    pub fn cluster_opinions(&mut self, id: u64) -> Result<Vec<u64>, String> {
        let url = format!(
            "{}/api/rest/v4/clusters/{id}/?fields=sub_opinions",
            self.base
        );
        Ok(opinion_ids(&self.json(&Request::Get(url))?["sub_opinions"]))
    }

    /// The opinion ids of the clusters `citation` resolves to.
    ///
    /// # Errors
    /// As [`Client::json`].
    pub fn lookup(&mut self, citation: &str) -> Result<Vec<u64>, String> {
        let url = format!("{}/api/rest/v4/citation-lookup/", self.base);
        let found = self.json(&Request::PostForm(
            url,
            vec![("text".to_owned(), citation.to_owned())],
        ))?;
        Ok(found
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|c| c["clusters"].as_array().into_iter().flatten())
            .flat_map(|cluster| opinion_ids(&cluster["sub_opinions"]))
            .collect())
    }
}

fn url_of(request: &Request) -> &str {
    match request {
        Request::Get(url) | Request::PostForm(url, _) => url,
    }
}

/// The ids at the end of a list of opinion URLs (`.../opinions/123/`).
fn opinion_ids(urls: &Value) -> Vec<u64> {
    urls.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|u| u.trim_end_matches('/').rsplit('/').next()?.parse().ok())
        .collect()
}

fn hit(result: &Value) -> Option<Hit> {
    let strings = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()
    };
    let date = result["dateFiled"].as_str().unwrap_or_default();
    let opinions: Vec<u64> = result["opinions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| o["id"].as_u64())
        .collect();
    if opinions.is_empty() {
        return None;
    }
    Some(Hit {
        cluster: result["cluster_id"].as_u64()?,
        opinions,
        case_name: result["caseName"].as_str().unwrap_or_default().to_owned(),
        court: result["court_id"].as_str().unwrap_or_default().to_owned(),
        date: date.get(..10).unwrap_or(date).to_owned(),
        url: format!(
            "{BASE_URL}{}",
            result["absolute_url"].as_str().unwrap_or_default()
        ),
        citations: strings(&result["citation"]),
    })
}

/// `key=value&...`, percent-encoded.
#[must_use]
pub fn encode(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", escape(k), escape(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn escape(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(b).to_string()
            }
            b' ' => "+".to_owned(),
            b => format!("%{b:02X}"),
        })
        .collect()
}

/// HTML as text: tags dropped, the common entities decoded.
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&sect;", "§")
        .replace("&#167;", "§")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_values_are_percent_encoded() {
        assert_eq!(
            encode(&[("q", "cites:(1 OR 2)"), ("court", "ca8 cadc")]),
            "q=cites%3A%281+OR+2%29&court=ca8+cadc"
        );
    }

    #[test]
    fn html_reads_as_text() {
        assert_eq!(
            strip_tags("<p>FRAP&nbsp;4(a) &amp; 28 U.S.C. &sect; 2107</p>"),
            " FRAP 4(a) & 28 U.S.C. § 2107 "
        );
    }

    #[test]
    fn a_result_without_a_cluster_or_an_opinion_is_no_hit() {
        assert_eq!(hit(&serde_json::json!({"opinions": [{"id": 1}]})), None);
        assert_eq!(hit(&serde_json::json!({"cluster_id": 7})), None);
        let h = hit(&serde_json::json!({
            "cluster_id": 7, "opinions": [{"id": 1}], "dateFiled": "2026-10-26T00:00:00-07:00"
        }));
        assert_eq!(h.map(|h| h.date), Some("2026-10-26".to_owned()));
    }
}
