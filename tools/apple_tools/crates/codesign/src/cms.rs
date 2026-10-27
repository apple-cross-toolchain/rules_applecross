//! The CMS signature (slot 0x10000) of an identity signature; see
//! docs/clean-room/codesign.md.
//!
//! A SignedData over the code directory as detached content, the identity's
//! certificate chain, and one SignerInfo whose signed attributes are, in DER
//! order: contentType, signingTime, messageDigest (the code directory's
//! SHA-256), Apple's 1.2.840.113635.100.9.2 (each code directory's hash by
//! algorithm) and 1.2.840.113635.100.9.1 (a property list of the cdhashes).

use crate::identity::Identity;
use rsa::Pkcs1v15Sign;
use sha2::{Digest, Sha256};

const OID_DATA: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x07, 0x01];
const OID_SIGNED_DATA: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x07, 0x02];
const OID_CONTENT_TYPE: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x09, 0x03];
const OID_MESSAGE_DIGEST: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x09, 0x04];
const OID_SIGNING_TIME: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x09, 0x05];
const OID_SHA256: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
const OID_SHA256_RSA: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b];
const OID_APPLE_CDHASHES_PLIST: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x63, 0x64, 0x09, 0x01];
const OID_APPLE_CDHASHES: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x63, 0x64, 0x09, 0x02];
const OID_TIMESTAMP_TOKEN: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x09, 0x10, 0x02, 0x0e];

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

/// BER's indefinite form, ended by an end-of-contents: codesign writes the
/// ContentInfo, its [0], the SignedData and the encapsulated content info so.
fn indefinite(tag: u8, parts: &[&[u8]]) -> Vec<u8> {
    let mut out = vec![tag, 0x80];
    out.extend(parts.concat());
    out.extend([0, 0]);
    out
}

fn seq(parts: &[&[u8]]) -> Vec<u8> {
    tlv(0x30, &parts.concat())
}

/// A DER SET OF: its elements sorted by their encodings.
fn set_of(tag: u8, mut items: Vec<Vec<u8>>) -> Vec<u8> {
    items.sort();
    tlv(tag, &items.concat())
}

fn oid(o: &[u8]) -> Vec<u8> {
    tlv(0x06, o)
}

fn algorithm(o: &[u8]) -> Vec<u8> {
    seq(&[&oid(o), &[0x05, 0x00]])
}

fn integer(bytes: &[u8]) -> Vec<u8> {
    let mut b: Vec<u8> = bytes.iter().copied().skip_while(|&x| x == 0).collect();
    if b.is_empty() || b[0] & 0x80 != 0 {
        b.insert(0, 0);
    }
    tlv(0x02, &b)
}

fn attribute(o: &[u8], value: Vec<u8>) -> Vec<u8> {
    seq(&[&oid(o), &set_of(0x31, vec![value])])
}

/// UTCTime, YYMMDDHHMMSSZ, for seconds since the Unix epoch.
fn utc_time(secs: u64) -> Vec<u8> {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    let s = format!("{:02}{:02}{:02}{:02}{:02}{:02}Z", y % 100, m, d, rem / 3600, rem / 60 % 60, rem % 60);
    tlv(0x17, s.as_bytes())
}

/// The signing time: SOURCE_DATE_EPOCH when set, so that a build signs the
/// same bytes every time, else now.
pub fn signing_time() -> u64 {
    if let Some(t) = std::env::var("SOURCE_DATE_EPOCH").ok().and_then(|v| v.trim().parse().ok()) {
        return t;
    }
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// The SignedData signing `cd`, the image's (only) SHA-256 code directory,
/// with a secure timestamp from the authority at `timestamp_url`, if given.
pub fn signed_data(cd: &[u8], identity: &Identity, time: u64, timestamp_url: Option<&str>) -> Result<Vec<u8>, String> {
    let cd_digest = Sha256::digest(cd);
    let plist = cf::xml::to_bytes(&cf::Value::dict([(
        "cdhashes",
        cf::Value::Array(vec![cf::Value::Data(cd_digest[..20].to_vec())]),
    )]));
    let attributes = vec![
        attribute(OID_CONTENT_TYPE, oid(OID_DATA)),
        attribute(OID_SIGNING_TIME, utc_time(time)),
        attribute(OID_MESSAGE_DIGEST, tlv(0x04, &cd_digest)),
        attribute(OID_APPLE_CDHASHES, seq(&[&oid(OID_SHA256), &tlv(0x04, &cd_digest)])),
        attribute(OID_APPLE_CDHASHES_PLIST, tlv(0x04, &plist)),
    ];
    // The signature covers the attributes encoded as a SET; the SignerInfo
    // carries the same bytes under [0] IMPLICIT.
    let signed_attrs = set_of(0x31, attributes);
    let digest = Sha256::digest(&signed_attrs);
    let signature = identity
        .key
        .sign(Pkcs1v15Sign::new::<Sha256>(), &digest)
        .map_err(|e| format!("signing: {e}"))?;
    let mut implicit = signed_attrs.clone();
    implicit[0] = 0xa0;
    // A timestamp is an unsigned attribute: [1] IMPLICIT SET OF Attribute.
    let unsigned = match timestamp_url {
        Some(url) => tlv(0xa1, &seq(&[&oid(OID_TIMESTAMP_TOKEN), &set_of(0x31, vec![crate::timestamp::timestamp(url, &signature)?])])),
        None => Vec::new(),
    };
    let signer_info = seq(&[
        &integer(&[1]),
        &seq(&[&identity.issuer, &integer(&identity.serial)]),
        &algorithm(OID_SHA256),
        &implicit,
        &algorithm(OID_SHA256_RSA),
        &tlv(0x04, &signature),
        &unsigned,
    ]);
    let signed = indefinite(0x30, &[
        &integer(&[1]),
        &set_of(0x31, vec![algorithm(OID_SHA256)]),
        &indefinite(0x30, &[&oid(OID_DATA)]),
        &set_of(0xa0, identity.chain.clone()),
        &set_of(0x31, vec![signer_info]),
    ]);
    Ok(indefinite(0x30, &[&oid(OID_SIGNED_DATA), &indefinite(0xa0, &[&signed])]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Apple's codesign signed tests/data/identity's code directory with its
    /// test key at this time; the same inputs give the same bytes.
    #[test]
    fn signs_as_codesign_did() {
        let identity = Identity::from_pem(
            include_str!("../tests/data/identity/test-key.pem"),
            include_str!("../tests/data/identity/test-chain.pem"),
        )
        .unwrap();
        let cd = include_bytes!("../tests/data/identity/apple-cd.bin");
        let apple = include_bytes!("../tests/data/identity/apple-cms.der");
        assert_eq!(signed_data(cd, &identity, 1_793_079_973, None).unwrap(), apple.to_vec());
    }

    #[test]
    fn utc_time_formats_civil_dates() {
        // 2026-10-27T05:46:13Z, the signing time Apple's codesign wrote.
        assert_eq!(utc_time(1_793_079_973), tlv(0x17, b"261027054613Z"));
        assert_eq!(utc_time(0), tlv(0x17, b"700101000000Z"));
        assert_eq!(utc_time(951_782_400), tlv(0x17, b"000229000000Z"));
    }
}
