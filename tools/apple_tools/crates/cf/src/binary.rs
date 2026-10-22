//! Binary property lists, as Interface Builder's compiler writes them.

use crate::hash::table_order;
use crate::Value;

struct Writer {
    objects: Vec<Vec<u8>>,
    ref_size: usize,
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

impl Writer {
    fn refs(&self, out: &mut Vec<u8>, r: usize) {
        out.extend_from_slice(&r.to_be_bytes()[8 - self.ref_size..]);
    }

    /// Writes `v` and everything below it; returns its object number.
    fn add(&mut self, v: &Value) -> usize {
        let me = self.objects.len();
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
                let children: Vec<usize> = items.iter().map(|i| self.add(i)).collect();
                for c in children {
                    self.refs(&mut out, c);
                }
            }
            Value::Dictionary(entries) => {
                marker(&mut out, 0xd, entries.len());
                let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
                let order = table_order(&keys);
                let key_refs: Vec<usize> = order.iter().map(|&i| self.add(&Value::String(entries[i].0.clone()))).collect();
                let value_refs: Vec<usize> = order.iter().map(|&i| self.add(&entries[i].1)).collect();
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
    let n = count(v);
    let mut w = Writer { objects: Vec::with_capacity(n), ref_size: width(n) };
    w.add(v);
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
