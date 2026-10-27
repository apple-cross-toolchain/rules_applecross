//! A signing identity for `codesign -s NAME`: a private key and its
//! certificate chain, read from PEM files instead of a keychain; see
//! docs/clean-room/codesign.md.
//!
//! `RULES_APPLECROSS_CODESIGN_KEY` names the key (PKCS#8 or PKCS#1 PEM) and
//! `RULES_APPLECROSS_CODESIGN_CERTIFICATES` the certificates (PEM, in any
//! order): the key's certificate and those it chains to.

use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use sha1::{Digest, Sha1};
use x509_cert::der::{Decode, DecodePem, Encode};
use x509_cert::Certificate;

pub const KEY_ENV: &str = "RULES_APPLECROSS_CODESIGN_KEY";
pub const CERTIFICATES_ENV: &str = "RULES_APPLECROSS_CODESIGN_CERTIFICATES";

pub struct Identity {
    pub key: RsaPrivateKey,
    /// DER certificates, the key's first, each issued by the next.
    pub chain: Vec<Vec<u8>>,
    /// The key's certificate's issuer Name and serial number, DER.
    pub issuer: Vec<u8>,
    pub serial: Vec<u8>,
    /// The key's certificate's subject common name.
    pub common_name: String,
}

fn common_name(cert: &Certificate) -> Option<String> {
    // id-at-commonName, 2.5.4.3
    const CN: &str = "2.5.4.3";
    for rdn in cert.tbs_certificate.subject.0.iter() {
        for atv in rdn.0.iter() {
            if atv.oid.to_string() == CN {
                let bytes = atv.value.value();
                return Some(String::from_utf8_lossy(bytes).into_owned());
            }
        }
    }
    None
}

fn pem_certificates(text: &str) -> Result<Vec<Certificate>, String> {
    let mut out = Vec::new();
    let mut rest = text;
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    while let Some(start) = rest.find(BEGIN) {
        let end = rest[start..].find(END).ok_or("certificate PEM without its END line")? + start + END.len();
        out.push(Certificate::from_pem(&rest.as_bytes()[start..end]).map_err(|e| format!("certificate: {e}"))?);
        rest = &rest[end..];
    }
    if out.is_empty() {
        return Err("no certificates".into());
    }
    Ok(out)
}

fn der(cert: &Certificate) -> Result<Vec<u8>, String> {
    cert.to_der().map_err(|e| e.to_string())
}

impl Identity {
    /// The identity the environment names, if it does.
    pub fn from_environment() -> Result<Option<Identity>, String> {
        let (Some(key), Some(certs)) = (std::env::var_os(KEY_ENV), std::env::var_os(CERTIFICATES_ENV)) else {
            return Ok(None);
        };
        let key_text = std::fs::read_to_string(&key).map_err(|e| format!("{}: {e}", key.to_string_lossy()))?;
        let certs_text = std::fs::read_to_string(&certs).map_err(|e| format!("{}: {e}", certs.to_string_lossy()))?;
        Identity::from_pem(&key_text, &certs_text).map(Some)
    }

    pub fn from_pem(key_pem: &str, certs_pem: &str) -> Result<Identity, String> {
        let key = RsaPrivateKey::from_pkcs8_pem(key_pem)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(key_pem))
            .map_err(|e| format!("private key: {e} (only RSA keys are supported)"))?;
        let certs = pem_certificates(certs_pem)?;
        let modulus = key.n().to_bytes_be();
        // The key's own certificate: the one whose RSA public key is the key's.
        let leaf = certs
            .iter()
            .position(|c| {
                let spki = c.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes();
                rsa::pkcs1::RsaPublicKey::from_der(spki).map(|p| p.modulus.as_bytes() == modulus.as_slice()).unwrap_or(false)
            })
            .ok_or("no certificate holds the private key's public key")?;
        // Follow issuers to a self-signed certificate.
        let mut chain = vec![leaf];
        loop {
            let last = &certs[*chain.last().unwrap()];
            if last.tbs_certificate.issuer == last.tbs_certificate.subject {
                break;
            }
            match certs.iter().position(|c| c.tbs_certificate.subject == last.tbs_certificate.issuer) {
                Some(i) if !chain.contains(&i) => chain.push(i),
                _ => break,
            }
        }
        let leaf_cert = &certs[leaf];
        Ok(Identity {
            key,
            chain: chain.iter().map(|&i| der(&certs[i])).collect::<Result<_, _>>()?,
            issuer: leaf_cert.tbs_certificate.issuer.to_der().map_err(|e| e.to_string())?,
            serial: leaf_cert.tbs_certificate.serial_number.as_bytes().to_vec(),
            common_name: common_name(leaf_cert).unwrap_or_default(),
        })
    }

    /// Whether `codesign -s NAME` names this identity: by its certificate's
    /// SHA-1, in hex, or by its common name or a prefix of it.
    pub fn matches(&self, name: &str) -> bool {
        let sha1 = hex(&Sha1::digest(&self.chain[0]));
        name.eq_ignore_ascii_case(&sha1) || (!name.is_empty() && self.common_name.starts_with(name))
    }

    /// The SHA-1 of the chain's last certificate, which a designated
    /// requirement anchors to when the chain isn't Apple's.
    pub fn anchor_sha1(&self) -> [u8; 20] {
        Sha1::digest(self.chain.last().unwrap()).into()
    }

    /// Whether the chain ends in one of Apple's roots (organization "Apple
    /// Inc."), whose designated requirement codesign writes differently.
    pub fn apple_anchored(&self) -> bool {
        const O: &str = "2.5.4.10";
        let Ok(root) = Certificate::from_der(self.chain.last().unwrap()) else { return false };
        root.tbs_certificate.subject.0.iter().flat_map(|rdn| rdn.0.iter()).any(|atv| {
            atv.oid.to_string() == O && atv.value.value() == b"Apple Inc."
        })
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}
