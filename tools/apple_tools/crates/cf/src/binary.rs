//! Binary property lists, as Interface Builder's compiler writes them.

use crate::hash::{sized_table_order, table_order};
use crate::Value;

struct Writer<'a> {
    objects: Vec<Vec<u8>>,
    ref_size: usize,
    /// Whether an equal string or number already written is written again
    /// or shared: Apple's writer shares by identity, so only the values a
    /// producer reuses (constants, small numbers, one object in two places)
    /// are shared.
    share: &'a dyn Fn(&Value, &Position) -> bool,
    written: Vec<(Value, usize)>,
    dictionaries: Dictionaries,
}

/// One step from a container to an object in it.
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Index(usize),
    /// The value of this key.
    Value(String),
    /// This key itself.
    Key(String),
}

/// Where an object is written: the steps to it from the top.
pub type Position = [Step];

/// How the writer's dictionaries were filled, which decides their keys'
/// order: one insertion at a time, or created whole.
#[derive(Clone, Copy)]
pub enum Dictionaries {
    Grown,
    Created,
}

fn count(v: &Value) -> usize {
    match v {
        Value::Array(a) => 1 + a.iter().map(count).sum::<usize>(),
        Value::Dictionary(d) => 1 + d.iter().map(|(_, v)| 1 + count(v)).sum::<usize>(),
        _ => 1,
    }
}

fn int_bytes(i: i64) -> (u8, Vec<u8>) {
    if (0..=0xff).contains(&i) {
        (0, vec![i as u8])
    } else if (0..=0xffff).contains(&i) {
        (1, (i as u16).to_be_bytes().to_vec())
    } else if (0..=0xffff_ffff).contains(&i) {
        (2, (i as u32).to_be_bytes().to_vec())
    } else {
        (3, i.to_be_bytes().to_vec())
    }
}

fn marker(out: &mut Vec<u8>, kind: u8, n: usize) {
    if n < 15 {
        out.push(kind << 4 | n as u8);
    } else {
        out.push(kind << 4 | 0xf);
        let (size, bytes) = int_bytes(n as i64);
        out.push(0x10 | size);
        out.extend(bytes);
    }
}

impl Writer<'_> {
    fn refs(&self, out: &mut Vec<u8>, r: usize) {
        out.extend_from_slice(&r.to_be_bytes()[8 - self.ref_size..]);
    }

    /// Writes `v` and everything below it; returns its object number.
    fn add(&mut self, v: &Value, at: &mut Vec<Step>) -> usize {
        let shareable = matches!(v, Value::String(_) | Value::Integer(_) | Value::Real(_)) && (self.share)(v, at);
        if shareable {
            if let Some((_, n)) = self.written.iter().find(|(w, _)| w == v) {
                return *n;
            }
        }
        let me = self.objects.len();
        if shareable {
            self.written.push((v.clone(), me));
        }
        self.objects.push(Vec::new());
        let mut out = Vec::new();
        match v {
            Value::Boolean(b) => out.push(if *b { 0x09 } else { 0x08 }),
            Value::Integer(i) => {
                let (size, bytes) = int_bytes(*i);
                out.push(0x10 | size);
                out.extend(bytes);
            }
            Value::Real(r) => {
                out.push(0x23);
                out.extend_from_slice(&r.to_be_bytes());
            }
            Value::Date(t) => {
                out.push(0x33);
                out.extend_from_slice(&t.to_be_bytes());
            }
            Value::Data(d) => {
                marker(&mut out, 0x4, d.len());
                out.extend_from_slice(d);
            }
            Value::String(s) if s.is_ascii() => {
                marker(&mut out, 0x5, s.len());
                out.extend_from_slice(s.as_bytes());
            }
            Value::String(s) => {
                let units: Vec<u16> = s.encode_utf16().collect();
                marker(&mut out, 0x6, units.len());
                for u in units {
                    out.extend_from_slice(&u.to_be_bytes());
                }
            }
            Value::Array(items) => {
                marker(&mut out, 0xa, items.len());
                let mut children = Vec::with_capacity(items.len());
                for (n, i) in items.iter().enumerate() {
                    at.push(Step::Index(n));
                    children.push(self.add(i, at));
                    at.pop();
                }
                for c in children {
                    self.refs(&mut out, c);
                }
            }
            Value::Dictionary(entries) => {
                marker(&mut out, 0xd, entries.len());
                let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
                let order = match self.dictionaries {
                    Dictionaries::Grown => table_order(&keys),
                    Dictionaries::Created => sized_table_order(&keys),
                };
                let mut key_refs = Vec::with_capacity(order.len());
                for &i in &order {
                    at.push(Step::Key(entries[i].0.clone()));
                    key_refs.push(self.add(&Value::String(entries[i].0.clone()), at));
                    at.pop();
                }
                let mut value_refs = Vec::with_capacity(order.len());
                for &i in &order {
                    at.push(Step::Value(entries[i].0.clone()));
                    value_refs.push(self.add(&entries[i].1, at));
                    at.pop();
                }
                for r in key_refs.into_iter().chain(value_refs) {
                    self.refs(&mut out, r);
                }
            }
        }
        self.objects[me] = out;
        me
    }
}

fn width(n: usize) -> usize {
    if n < 1 << 8 {
        1
    } else if n < 1 << 16 {
        2
    } else if n < 1 << 32 {
        4
    } else {
        8
    }
}

pub fn to_bytes(v: &Value) -> Vec<u8> {
    to_bytes_with(v, &|_, _| false, Dictionaries::Grown)
}

/// As `to_bytes`, sharing the equal strings and numbers `share` accepts,
/// with dictionaries filled as `dictionaries` says.
pub fn to_bytes_with(v: &Value, share: &dyn Fn(&Value, &Position) -> bool, dictionaries: Dictionaries) -> Vec<u8> {
    // References are sized for the objects written; count those first.
    let mut probe = Writer { objects: Vec::new(), ref_size: 8, share, written: Vec::new(), dictionaries };
    probe.add(v, &mut Vec::new());
    let n = probe.objects.len();
    debug_assert!(n <= count(v));
    let mut w = Writer { objects: Vec::with_capacity(n), ref_size: width(n), share, written: Vec::new(), dictionaries };
    w.add(v, &mut Vec::new());
    let mut out = b"bplist00".to_vec();
    let mut offsets = Vec::with_capacity(n);
    for o in &w.objects {
        offsets.push(out.len());
        out.extend_from_slice(o);
    }
    let table_offset = out.len();
    let offset_size = width(table_offset);
    for o in offsets {
        out.extend_from_slice(&o.to_be_bytes()[8 - offset_size..]);
    }
    out.extend_from_slice(&[0; 6]);
    out.push(offset_size as u8);
    out.push(w.ref_size as u8);
    out.extend_from_slice(&(n as u64).to_be_bytes());
    out.extend_from_slice(&0u64.to_be_bytes());
    out.extend_from_slice(&(table_offset as u64).to_be_bytes());
    out
}
