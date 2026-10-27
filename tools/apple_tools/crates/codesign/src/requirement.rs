//! Code requirements: the designated requirements codesign writes, in their
//! binary form and in the text a bundle's seal records; see
//! docs/clean-room/codesign.md.

const CSMAGIC_REQUIREMENT: u32 = 0xfade_0c00;
const CSMAGIC_REQUIREMENTS: u32 = 0xfade_0c01;
const KIND_EXPR: u32 = 1;
const DESIGNATED: u32 = 3;

const OP_IDENT: u32 = 2;
const OP_ANCHOR_HASH: u32 = 4;
const OP_AND: u32 = 6;
const OP_CERT_FIELD: u32 = 11;
const OP_CERT_GENERIC: u32 = 14;
const OP_APPLE_GENERIC_ANCHOR: u32 = 15;
const MATCH_EXISTS: u32 = 0;
const MATCH_EQUAL: u32 = 1;

/// A certificate slot: 0 the leaf, 1 its issuer, …, -1 the root.
pub const LEAF: i32 = 0;
pub const ROOT: i32 = -1;

#[derive(Debug, PartialEq)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Identifier(String),
    /// `certificate SLOT = H"…"`: a certificate by its SHA-1.
    AnchorHash { slot: i32, hash: Vec<u8> },
    /// `anchor apple generic`
    AppleGenericAnchor,
    /// `certificate SLOT[FIELD] = VALUE`
    CertField { slot: i32, field: String, value: String },
    /// `certificate SLOT[field.OID] /* exists */`, the OID in DER.
    CertExtension { slot: i32, oid: Vec<u8> },
}

/// `a and b and c`, nested to the right as codesign compiles it.
pub fn and(mut parts: Vec<Expr>) -> Expr {
    let last = parts.pop().expect("an empty conjunction");
    parts.into_iter().rev().fold(last, |acc, e| Expr::And(Box::new(e), Box::new(acc)))
}

fn be32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn data(out: &mut Vec<u8>, d: &[u8]) {
    be32(out, d.len() as u32);
    out.extend_from_slice(d);
    out.resize(out.len().next_multiple_of(4), 0);
}

fn encode(out: &mut Vec<u8>, e: &Expr) {
    match e {
        Expr::And(a, b) => {
            be32(out, OP_AND);
            encode(out, a);
            encode(out, b);
        }
        Expr::Identifier(id) => {
            be32(out, OP_IDENT);
            data(out, id.as_bytes());
        }
        Expr::AnchorHash { slot, hash } => {
            be32(out, OP_ANCHOR_HASH);
            be32(out, *slot as u32);
            data(out, hash);
        }
        Expr::AppleGenericAnchor => be32(out, OP_APPLE_GENERIC_ANCHOR),
        Expr::CertField { slot, field, value } => {
            be32(out, OP_CERT_FIELD);
            be32(out, *slot as u32);
            data(out, field.as_bytes());
            be32(out, MATCH_EQUAL);
            data(out, value.as_bytes());
        }
        Expr::CertExtension { slot, oid } => {
            be32(out, OP_CERT_GENERIC);
            be32(out, *slot as u32);
            data(out, oid);
            be32(out, MATCH_EXISTS);
        }
    }
}

fn blob(magic: u32, payload: &[u8]) -> Vec<u8> {
    let mut b = Vec::with_capacity(payload.len() + 8);
    be32(&mut b, magic);
    be32(&mut b, (payload.len() + 8) as u32);
    b.extend_from_slice(payload);
    b
}

/// A requirement set holding `designated` as the designated requirement.
pub fn designated_set(designated: &Expr) -> Vec<u8> {
    let mut req = Vec::new();
    be32(&mut req, KIND_EXPR);
    encode(&mut req, designated);
    let req = blob(CSMAGIC_REQUIREMENT, &req);
    let mut set = Vec::new();
    be32(&mut set, 1);
    be32(&mut set, DESIGNATED);
    be32(&mut set, 20);
    set.extend(req);
    blob(CSMAGIC_REQUIREMENTS, &set)
}

/// The empty requirement set of an ad hoc signature.
pub fn empty_set() -> Vec<u8> {
    blob(CSMAGIC_REQUIREMENTS, &0u32.to_be_bytes())
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn u32(&mut self) -> Option<u32> {
        let v = u32::from_be_bytes(self.b.get(self.at..self.at + 4)?.try_into().ok()?);
        self.at += 4;
        Some(v)
    }
    fn data(&mut self) -> Option<Vec<u8>> {
        let n = self.u32()? as usize;
        let d = self.b.get(self.at..self.at + n)?.to_vec();
        self.at += n.next_multiple_of(4);
        Some(d)
    }
    fn expr(&mut self) -> Option<Expr> {
        Some(match self.u32()? {
            OP_AND => Expr::And(Box::new(self.expr()?), Box::new(self.expr()?)),
            OP_IDENT => Expr::Identifier(String::from_utf8(self.data()?).ok()?),
            OP_ANCHOR_HASH => Expr::AnchorHash { slot: self.u32()? as i32, hash: self.data()? },
            OP_APPLE_GENERIC_ANCHOR => Expr::AppleGenericAnchor,
            OP_CERT_FIELD => {
                let slot = self.u32()? as i32;
                let field = String::from_utf8(self.data()?).ok()?;
                if self.u32()? != MATCH_EQUAL {
                    return None;
                }
                Expr::CertField { slot, field, value: String::from_utf8(self.data()?).ok()? }
            }
            OP_CERT_GENERIC => {
                let slot = self.u32()? as i32;
                let oid = self.data()?;
                if self.u32()? != MATCH_EXISTS {
                    return None;
                }
                Expr::CertExtension { slot, oid }
            }
            _ => return None,
        })
    }
}

/// The designated requirement in a requirement set, if it has one. Forms
/// other than the ones above are refused.
pub fn designated_in_set(set: &[u8]) -> Result<Option<Expr>, String> {
    let refused = || "nested code's designated requirement is not one this codesign understands".to_string();
    let mut r = Reader { b: set, at: 0 };
    if r.u32() != Some(CSMAGIC_REQUIREMENTS) {
        return Err(refused());
    }
    r.u32();
    let count = r.u32().ok_or_else(refused)?;
    let mut offset = None;
    for _ in 0..count {
        let (kind, off) = (r.u32().ok_or_else(refused)?, r.u32().ok_or_else(refused)?);
        if kind == DESIGNATED {
            offset = Some(off as usize);
        }
    }
    let Some(offset) = offset else { return Ok(None) };
    let mut r = Reader { b: set, at: offset };
    if r.u32() != Some(CSMAGIC_REQUIREMENT) {
        return Err(refused());
    }
    let len = r.u32().ok_or_else(refused)? as usize;
    if r.u32() != Some(KIND_EXPR) {
        return Err(refused());
    }
    let e = r.expr().ok_or_else(refused)?;
    if r.at != offset + len {
        return Err(refused());
    }
    Ok(Some(e))
}

/// A string as the requirement language writes it: bare when it is letters,
/// digits and `_` only, otherwise quoted.
fn string(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn slot(s: i32) -> String {
    match s {
        LEAF => "leaf".into(),
        ROOT => "root".into(),
        n => n.to_string(),
    }
}

/// An OID's dotted form from its DER contents.
fn oid_text(der: &[u8]) -> String {
    let mut parts = Vec::new();
    if let Some(&first) = der.first() {
        parts.push((first / 40) as u64);
        parts.push((first % 40) as u64);
    }
    let mut v = 0u64;
    for &b in der.iter().skip(1) {
        v = (v << 7) | (b & 0x7f) as u64;
        if b & 0x80 == 0 {
            parts.push(v);
            v = 0;
        }
    }
    parts.iter().map(u64::to_string).collect::<Vec<_>>().join(".")
}

/// The requirement in codesign's text form.
pub fn text(e: &Expr) -> String {
    match e {
        Expr::And(a, b) => format!("{} and {}", text(a), text(b)),
        Expr::Identifier(id) => format!("identifier {}", string(id)),
        Expr::AnchorHash { slot: s, hash } => {
            format!("certificate {} = H\"{}\"", slot(*s), hash.iter().map(|b| format!("{b:02x}")).collect::<String>())
        }
        Expr::AppleGenericAnchor => "anchor apple generic".into(),
        Expr::CertField { slot: s, field, value } => format!("certificate {}[{field}] = {}", slot(*s), string(value)),
        Expr::CertExtension { slot: s, oid } => format!("certificate {}[field.{}] /* exists */", slot(*s), oid_text(oid)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Apple Development, as codesign signed an app with one (team and name
    /// of the leaf replaced; the structure is codesign's).
    #[test]
    fn apple_development_form_round_trips() {
        let e = and(vec![
            Expr::Identifier("com.example.App".into()),
            Expr::AppleGenericAnchor,
            Expr::CertField { slot: LEAF, field: "subject.CN".into(), value: "Apple Development: A B (ABCDE12345)".into() },
            Expr::CertExtension { slot: 1, oid: vec![0x2a, 0x86, 0x48, 0x86, 0xf7, 0x63, 0x64, 0x06, 0x02, 0x01] },
        ]);
        let set = designated_set(&e);
        assert_eq!(designated_in_set(&set).unwrap(), Some(e));
        assert_eq!(
            text(&designated_in_set(&set).unwrap().unwrap()),
            "identifier \"com.example.App\" and anchor apple generic and certificate leaf[subject.CN] = \"Apple Development: A B (ABCDE12345)\" and certificate 1[field.1.2.840.113635.100.6.2.1] /* exists */"
        );
    }

    /// Developer ID's form, compiled as codesign compiles it: checked against
    /// a Developer ID app's requirement set when this was written (team and
    /// identifier replaced here).
    #[test]
    fn developer_id_form_round_trips() {
        let e = and(vec![
            Expr::Identifier("com.example.App".into()),
            Expr::AppleGenericAnchor,
            Expr::CertExtension { slot: 1, oid: vec![0x2a, 0x86, 0x48, 0x86, 0xf7, 0x63, 0x64, 0x06, 0x02, 0x06] },
            Expr::CertExtension { slot: LEAF, oid: vec![0x2a, 0x86, 0x48, 0x86, 0xf7, 0x63, 0x64, 0x06, 0x01, 0x0d] },
            Expr::CertField { slot: LEAF, field: "subject.OU".into(), value: "ABCDE12345".into() },
        ]);
        assert_eq!(designated_in_set(&designated_set(&e)).unwrap(), Some(e));
    }
}
