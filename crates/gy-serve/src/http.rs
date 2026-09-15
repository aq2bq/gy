//! The HTTP shapes a handler sees; no framework type reaches here (ac-a49a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn new(
        method: &str,
        path: &str,
        query: Option<&str>,
        headers: &[(String, String)],
    ) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            query: query.map(parse_query).unwrap_or_default(),
            headers: headers.to_vec(),
        }
    }

    /// The first value of a header, its name compared without case.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The first value of a query parameter.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// `a=1&b` is two pairs; a missing query is none.
fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((key, value)) => (key.to_string(), value.to_string()),
            None => (pair.to_string(), String::new()),
        })
        .collect()
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl Response {
    pub fn json<T: serde::Serialize>(value: &T) -> Self {
        Response {
            status: 200,
            content_type: "application/json; charset=utf-8",
            body: serde_json::to_vec(value).unwrap_or_default(),
        }
    }

    pub fn text(status: u16, body: &str) -> Self {
        Response {
            status,
            content_type: "text/plain; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn html(body: String) -> Self {
        Response {
            status: 200,
            content_type: "text/html; charset=utf-8",
            body: body.into_bytes(),
        }
    }

    pub fn not_found() -> Self {
        Self::text(404, "not found")
    }

    pub fn method_not_allowed() -> Self {
        Self::text(405, "method not allowed")
    }
}
