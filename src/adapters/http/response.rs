//! Building HTTP responses. Serialises, sets headers, writes bytes.

use anyhow::Result;
use std::io::Write;

/// A response ready to write to a socket.
#[derive(Debug)]
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Response body. JSON for every route this server serves.
    pub body: Vec<u8>,
}

impl Response {
    /// A 200 carrying a JSON body.
    #[must_use]
    pub fn json(body: Vec<u8>) -> Self {
        Self { status: 200, body }
    }

    /// An error response, as JSON so a client never has to parse prose.
    ///
    /// The message is the thing the operator needs, not a generic status text.
    /// A refusal that says only "400 Bad Request" moves the diagnosis to whoever
    /// is reading a browser console, which for this tool is nobody.
    #[must_use]
    pub fn error(status: u16, message: &str) -> Self {
        // Hand-built rather than via serde_json, because this path must work even
        // when serialisation is what failed. `escape` covers the characters JSON
        // requires; a message containing anything else still produces valid JSON.
        let mut body = String::from("{\"error\":\"");
        body.push_str(&escape(message));
        body.push_str("\"}");
        Self {
            status,
            body: body.into_bytes(),
        }
    }

    /// Status line, headers and body, written in one pass.
    ///
    /// # Errors
    /// Fails if the socket cannot be written to, which for a client that has
    /// hung up is ordinary and not worth distinguishing.
    pub fn write_to<W: Write>(&self, out: &mut W) -> Result<()> {
        let reason = match self.status {
            200 => "OK",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            413 => "Payload Too Large",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        // Connection: close on every response. Keep-alive is what makes an HTTP
        // implementation complicated — a mis-framed body silently becomes the next
        // request — and one console making a handful of calls does not need it.
        //
        // The three security headers are cheap and rule out whole classes of
        // mistake in a client that this process cannot see: nosniff stops a JSON
        // body being executed if a browser is ever pointed here directly, and a
        // frame-ancestors of 'none' stops this being embedded in a page elsewhere.
        write!(
            out,
            "HTTP/1.1 {} {reason}\r\n\
             Content-Type: application/json; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             X-Content-Type-Options: nosniff\r\n\
             Cache-Control: no-store\r\n\
             Content-Security-Policy: default-src 'none'; frame-ancestors 'none'\r\n\
             \r\n",
            self.status,
            self.body.len()
        )?;
        out.write_all(&self.body)?;
        out.flush()?;
        Ok(())
    }
}

/// Escape a string for inclusion in a JSON string literal.
///
/// Control characters are escaped as `\u00XX` rather than dropped: dropping a
/// character would change a person's name silently, and this is used on error
/// messages that can contain a slug or a path.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                // No indexing, no unwrap: write! into a String cannot fail, and
                // the result is discarded deliberately rather than silently.
                let _ = std::fmt::Write::write_fmt(&mut out, format_args!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
// The lint configuration denies expect/unwrap/indexing for the crate, because a
// panic in a running tool is a defect. In a test an assertion IS the point, and
// the same convention is used by every other test module in this crate.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn an_error_body_is_valid_json() {
        let r = Response::error(404, "no such person");
        let parsed: serde_json::Value = serde_json::from_slice(&r.body).expect("valid json");
        assert_eq!(parsed["error"], "no such person");
    }

    #[test]
    fn a_message_containing_quotes_and_newlines_stays_valid_json() {
        let r = Response::error(400, "slug \"a\\b\"\n\tnot found");
        let parsed: serde_json::Value = serde_json::from_slice(&r.body).expect("valid json");
        assert_eq!(parsed["error"], "slug \"a\\b\"\n\tnot found");
    }

    #[test]
    fn a_control_character_is_escaped_not_dropped() {
        let r = Response::error(400, "bell\u{7}end");
        let parsed: serde_json::Value = serde_json::from_slice(&r.body).expect("valid json");
        assert_eq!(parsed["error"], "bell\u{7}end");
    }

    #[test]
    fn the_response_declares_its_length_and_closes() {
        let r = Response::json(b"{\"a\":1}".to_vec());
        let mut buf = Vec::new();
        r.write_to(&mut buf).expect("write");
        let text = String::from_utf8(buf).expect("utf8");
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Length: 7\r\n"));
        assert!(text.contains("Connection: close\r\n"));
        assert!(text.ends_with("\r\n\r\n{\"a\":1}"));
    }
}
