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

/// `a=1&b` is two pairs; a missing query is none. A value is percent-decoded,
/// so a browser's `q=%E7%A7%BB%E8%A1%8C` is the text it typed.
fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((key, value)) => (key.to_string(), decode(value, true)),
            None => (pair.to_string(), String::new()),
        })
        .collect()
}

/// Percent-decode text as UTF-8. `+` becomes a space when `plus_as_space` (a
/// query value, not a path). A broken escape is kept as it stands.
pub fn decode(text: &str, plus_as_space: bool) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' && index + 2 < bytes.len() {
            if let Some(value) = hex(&bytes[index + 1..index + 3]) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(if plus_as_space && byte == b'+' {
            b' '
        } else {
            byte
        });
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Two hex digits as one byte.
fn hex(pair: &[u8]) -> Option<u8> {
    let digit = |byte: u8| (byte as char).to_digit(16).map(|value| value as u8);
    Some(digit(pair[0])? * 16 + digit(pair[1])?)
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
