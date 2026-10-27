//! Secure timestamps for identity signatures: an RFC 3161 request to a
//! timestamp authority, Apple's unless `--timestamp=URL` names another; see
//! docs/clean-room/codesign.md.
//!
//! The request carries the SHA-256 of the signature value, a 64-bit nonce and
//! asks for the authority's certificates; the token returned goes, unchanged,
//! into the SignerInfo's unsigned attributes.

use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub const APPLE: &str = "http://timestamp.apple.com/ts01";

fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    let n = content.len();
    if n < 0x80 {
        out.push(n as u8);
    } else {
        let bytes: Vec<u8> = n.to_be_bytes().iter().copied().skip_while(|&b| b == 0).collect();
        out.push(0x80 | bytes.len() as u8);
        out.extend(bytes);
    }
    out.extend_from_slice(content);
    out
}

fn integer(bytes: &[u8]) -> Vec<u8> {
    let mut b: Vec<u8> = bytes.iter().copied().skip_while(|&x| x == 0).collect();
    if b.is_empty() || b[0] & 0x80 != 0 {
        b.insert(0, 0);
    }
    tlv(0x02, &b)
}

/// The TimeStampReq for a signature value.
pub fn request(signature: &[u8], nonce: &[u8; 8]) -> Vec<u8> {
    const OID_SHA256: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
    let algorithm = tlv(0x30, &[tlv(0x06, OID_SHA256), vec![0x05, 0x00]].concat());
    let imprint = tlv(0x30, &[algorithm, tlv(0x04, &Sha256::digest(signature))].concat());
    tlv(0x30, &[integer(&[1]), imprint, integer(nonce), vec![0x01, 0x01, 0xff]].concat())
}

/// One DER element at `at`: (tag, content start, end).
fn element(b: &[u8], at: usize) -> Option<(u8, usize, usize)> {
    let tag = *b.get(at)?;
    let first = *b.get(at + 1)?;
    let (len, start) = if first < 0x80 {
        (first as usize, at + 2)
    } else {
        let n = (first & 0x7f) as usize;
        let len = b.get(at + 2..at + 2 + n)?.iter().fold(0usize, |a, &x| (a << 8) | x as usize);
        (len, at + 2 + n)
    };
    (start + len <= b.len()).then_some((tag, start, start + len))
}

/// The token in a TimeStampResp, if the authority granted one.
pub fn token(response: &[u8]) -> Result<Vec<u8>, String> {
    let bad = || "the timestamp authority's response is malformed".to_string();
    let (_, start, end) = element(response, 0).ok_or_else(bad)?;
    let (_, status_start, status_end) = element(response, start).ok_or_else(bad)?;
    let (_, int_start, int_end) = element(response, status_start).ok_or_else(bad)?;
    let status = response[int_start..int_end].iter().fold(0u32, |a, &x| (a << 8) | x as u32);
    // granted (0) or grantedWithMods (1)
    if status > 1 {
        return Err(format!("the timestamp authority refused the request (status {status})"));
    }
    if status_end >= end {
        return Err("the timestamp authority returned no token".into());
    }
    let (_, _, token_end) = element(response, status_end).ok_or_else(bad)?;
    Ok(response[status_end..token_end].to_vec())
}

/// POSTs `body` to an http:// URL and returns the response body.
fn post(url: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    let rest = url.strip_prefix("http://").ok_or_else(|| format!("{url}: only http:// timestamp authorities are supported"))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let host = authority.split(':').next().unwrap_or(authority);
    let address = if authority.contains(':') { authority.to_string() } else { format!("{authority}:80") };
    let mut stream = TcpStream::connect(&address).map_err(|e| format!("{url}: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(60))).ok();
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/timestamp-query\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).and_then(|_| stream.write_all(body)).map_err(|e| format!("{url}: {e}"))?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|e| format!("{url}: {e}"))?;
    let split = response.windows(4).position(|w| w == b"\r\n\r\n").ok_or_else(|| format!("{url}: malformed HTTP response"))?;
    let headers = String::from_utf8_lossy(&response[..split]).to_ascii_lowercase();
    let status_line = headers.lines().next().unwrap_or_default().to_string();
    if !status_line.split_whitespace().nth(1).is_some_and(|s| s == "200") {
        return Err(format!("{url}: {status_line}"));
    }
    let content = response[split + 4..].to_vec();
    if headers.lines().any(|l| l.starts_with("transfer-encoding:") && l.contains("chunked")) {
        return dechunk(&content).ok_or_else(|| format!("{url}: malformed chunked response"));
    }
    Ok(content)
}

fn dechunk(mut b: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = b.windows(2).position(|w| w == b"\r\n")?;
        let size = usize::from_str_radix(std::str::from_utf8(&b[..line_end]).ok()?.split(';').next()?.trim(), 16).ok()?;
        b = &b[line_end + 2..];
        if size == 0 {
            return Some(out);
        }
        out.extend_from_slice(b.get(..size)?);
        b = b.get(size + 2..)?;
    }
}

/// A timestamp token for a signature value from the authority at `url`.
pub fn timestamp(url: &str, signature: &[u8]) -> Result<Vec<u8>, String> {
    let mut nonce = [0u8; 8];
    rsa::rand_core::RngCore::fill_bytes(&mut rsa::rand_core::OsRng, &mut nonce);
    token(&post(url, &request(signature, &nonce))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_der() {
        let r = request(b"signature", &[0x84, 1, 2, 3, 4, 5, 6, 7]);
        // version 1, sha256 with NULL, imprint, a nonce made positive, certReq
        assert_eq!(&r[..4], &[0x30, 0x44, 0x02, 0x01]);
        assert!(r.ends_with(&[0x02, 0x09, 0x00, 0x84, 1, 2, 3, 4, 5, 6, 7, 0x01, 0x01, 0xff]));
    }

    #[test]
    fn chunked_bodies_join() {
        assert_eq!(dechunk(b"3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n").unwrap(), b"abcde");
    }
}
