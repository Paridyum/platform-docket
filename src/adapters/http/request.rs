//! Parsing a request: the line, the headers, the body, and the limits on each.

use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};

/// Longest request line accepted, in bytes.
///
/// A path here is `/api/person/<slug>`; 8 KiB is far more than any real route and
/// small enough that a client sending an unbounded line is cut off rather than
/// growing this process's memory.
const MAX_LINE: usize = 8 * 1024;

/// Most headers accepted.
const MAX_HEADERS: usize = 64;

/// Largest body accepted, in bytes.
///
/// An enrolment form is the biggest write, and it is a few kilobytes of answers.
/// 256 KiB leaves room for that to grow by two orders of magnitude and still
/// refuses anything that could only be a mistake or an attack.
const MAX_BODY: usize = 256 * 1024;

/// What the client asked for.
#[derive(Debug)]
pub struct Request {
    /// Request method, uppercase, as sent.
    pub method: String,
    /// Path with any query string removed.
    pub path: String,
    /// Query string, absent when there was none.
    pub query: Option<String>,
    /// Headers, lowercased names. `BTreeMap` because STD-0004 disallows `HashMap`
    /// — iteration order here is also what makes a failure reproducible.
    pub headers: BTreeMap<String, String>,
    /// Body bytes, empty for a request that carried none.
    pub body: Vec<u8>,
}

impl Request {
    /// One header, by lowercase name.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }

    /// Path split on `/`, with empty segments dropped.
    ///
    /// `/api/person/rohan/` and `/api/person/rohan` produce the same segments,
    /// so a trailing slash cannot route somewhere different from the path an
    /// operator meant.
    #[must_use]
    pub fn segments(&self) -> Vec<&str> {
        self.path.split('/').filter(|s| !s.is_empty()).collect()
    }
}

/// Read and parse one request from a stream.
///
/// Strict on purpose: anything not understood is refused rather than guessed at.
/// A server that tolerates a malformed request is a server whose behaviour is
/// decided by whoever sends the malformed request.
///
/// # Errors
/// Fails on a malformed request line, a missing or unparsable `Content-Length`
/// where a body is present, or any limit above being exceeded.
pub fn parse<R: Read>(stream: R) -> Result<Request> {
    let mut reader = BufReader::new(stream);

    let line = read_line(&mut reader)?;
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(version)) = (parts.next(), parts.next(), parts.next())
    else {
        bail!("malformed request line");
    };
    if parts.next().is_some() {
        bail!("malformed request line: too many fields");
    }
    if !version.starts_with("HTTP/1.") {
        bail!("unsupported HTTP version: {version}");
    }
    if !target.starts_with('/') {
        // Refuses absolute-form targets (`GET http://host/path`) and the
        // asterisk-form used by OPTIONS. Neither is something the console sends,
        // and accepting an absolute form means deciding whether the host in it
        // matters.
        bail!("unsupported request target: {target}");
    }

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_owned(), Some(q.to_owned())),
        None => (target.to_owned(), None),
    };

    let mut headers = BTreeMap::new();
    loop {
        let header = read_line(&mut reader)?;
        if header.is_empty() {
            break;
        }
        if headers.len() >= MAX_HEADERS {
            bail!("too many headers");
        }
        let Some((name, value)) = header.split_once(':') else {
            bail!("malformed header");
        };
        // Lowercased on the way in, so a handler never has to guess the casing a
        // client chose. HTTP header names are case-insensitive and clients differ.
        headers.insert(name.trim().to_lowercase(), value.trim().to_owned());
    }

    let body = if let Some(raw) = headers.get("content-length") {
        let len: usize = raw
            .parse()
            .map_err(|_| anyhow::anyhow!("unparsable Content-Length: {raw}"))?;
        if len > MAX_BODY {
            bail!("body too large: {len} bytes");
        }
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        buf
    } else {
        // No Content-Length means no body. `Transfer-Encoding: chunked` is
        // refused rather than implemented: the console sends neither, and
        // chunked decoding is where a hand-written server earns its bugs.
        if headers.contains_key("transfer-encoding") {
            bail!("Transfer-Encoding is not supported; send Content-Length");
        }
        Vec::new()
    };

    Ok(Request {
        method: method.to_owned(),
        path,
        query,
        headers,
        body,
    })
}

/// Read one CRLF-terminated line, without the terminator, bounded by `MAX_LINE`.
fn read_line<R: BufRead>(reader: &mut R) -> Result<String> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        if buf.len() > MAX_LINE {
            bail!("request line too long");
        }
        let n = reader.read(&mut byte)?;
        if n == 0 {
            if buf.is_empty() {
                bail!("client closed the connection");
            }
            break;
        }
        // Compare the single byte read rather than indexing, which the lint
        // configuration denies.
        match byte.first() {
            // A newline ends the line; so does a zero-byte read, which the check
            // above has already established is not an empty first line.
            Some(b'\n') | None => break,
            Some(b) => buf.push(*b),
        }
    }
    // A bare LF is accepted as a line ending as well as CRLF. Being lenient about
    // exactly this is safe — it cannot change which request is understood — and it
    // makes the server testable with ordinary string literals.
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    Ok(String::from_utf8(buf)?)
}

#[cfg(test)]
// The lint configuration denies expect/unwrap/indexing for the crate, because a
// panic in a running tool is a defect. In a test an assertion IS the point, and
// the same convention is used by every other test module in this crate.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn parse_str(text: &str) -> Result<Request> {
        parse(text.as_bytes())
    }

    #[test]
    fn a_get_with_no_body_parses() {
        let r = parse_str("GET /api/roster HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").expect("parses");
        assert_eq!(r.method, "GET");
        assert_eq!(r.path, "/api/roster");
        assert_eq!(r.query, None);
        assert_eq!(r.header("host"), Some("127.0.0.1"));
        assert!(r.body.is_empty());
    }

    #[test]
    fn a_query_string_is_split_from_the_path() {
        let r = parse_str("GET /api/roster?standing=active HTTP/1.1\r\n\r\n").expect("parses");
        assert_eq!(r.path, "/api/roster");
        assert_eq!(r.query.as_deref(), Some("standing=active"));
    }

    #[test]
    fn header_names_are_matched_regardless_of_the_casing_a_client_chose() {
        let r = parse_str("GET / HTTP/1.1\r\nX-Docket-Operator: ruhani\r\n\r\n").expect("parses");
        assert_eq!(r.header("x-docket-operator"), Some("ruhani"));
    }

    #[test]
    fn a_body_is_read_to_its_declared_length() {
        let r =
            parse_str("POST /api/x HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello").expect("parses");
        assert_eq!(r.body, b"hello");
    }

    #[test]
    fn a_trailing_slash_routes_the_same_as_no_trailing_slash() {
        let a = parse_str("GET /api/person/rohan HTTP/1.1\r\n\r\n").expect("parses");
        let b = parse_str("GET /api/person/rohan/ HTTP/1.1\r\n\r\n").expect("parses");
        assert_eq!(a.segments(), b.segments());
        assert_eq!(a.segments(), vec!["api", "person", "rohan"]);
    }

    #[test]
    fn a_malformed_request_line_is_refused() {
        assert!(parse_str("GET\r\n\r\n").is_err());
        assert!(parse_str("GET /api/roster\r\n\r\n").is_err());
        assert!(parse_str("GET /api/roster HTTP/1.1 extra\r\n\r\n").is_err());
    }

    #[test]
    fn http_two_is_refused_rather_than_treated_as_one_point_one() {
        assert!(parse_str("GET / HTTP/2.0\r\n\r\n").is_err());
    }

    #[test]
    fn an_absolute_form_target_is_refused() {
        // Accepting this would mean deciding whether the host in it matters, and
        // the console never sends it.
        assert!(parse_str("GET http://example.com/api/roster HTTP/1.1\r\n\r\n").is_err());
    }

    #[test]
    fn chunked_encoding_is_refused_rather_than_half_implemented() {
        let err = parse_str("POST /api/x HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello")
            .expect_err("refused");
        assert!(format!("{err}").contains("Transfer-Encoding"));
    }

    #[test]
    fn an_unparsable_content_length_is_refused() {
        assert!(parse_str("POST /api/x HTTP/1.1\r\nContent-Length: seven\r\n\r\n").is_err());
    }

    #[test]
    fn an_oversized_body_is_refused_without_reading_it() {
        let err = parse_str("POST /api/x HTTP/1.1\r\nContent-Length: 999999999\r\n\r\n")
            .expect_err("refused");
        assert!(format!("{err}").contains("too large"));
    }

    #[test]
    fn an_overlong_request_line_is_refused() {
        let long = "a".repeat(MAX_LINE + 10);
        let err = parse_str(&format!("GET /{long} HTTP/1.1\r\n\r\n")).expect_err("refused");
        assert!(format!("{err}").contains("too long"));
    }

    #[test]
    fn too_many_headers_are_refused() {
        let mut text = String::from("GET / HTTP/1.1\r\n");
        for i in 0..(MAX_HEADERS + 5) {
            let _ = std::fmt::Write::write_fmt(&mut text, format_args!("X-H{i}: v\r\n"));
        }
        text.push_str("\r\n");
        let err = parse_str(&text).expect_err("refused");
        assert!(format!("{err}").contains("too many headers"));
    }

    #[test]
    fn a_bare_lf_line_ending_parses_so_the_server_is_testable_with_string_literals() {
        let r = parse_str("GET /api/roster HTTP/1.1\nHost: x\n\n").expect("parses");
        assert_eq!(r.path, "/api/roster");
        assert_eq!(r.header("host"), Some("x"));
    }
}
