//! NIB archives, as UIKit's nib encoder writes them; see docs/clean-room/ibtool.md.

use std::collections::HashMap;

/// A value an object encodes.
pub enum Value {
    Int(i64),
    Bool(bool),
    Float(f32),
    Double(f64),
    Data(Vec<u8>),
    Nil,
    /// A new object, encoded when this value is.
    New(Object),
    /// An object encoded earlier, by name.
    Ref(String),
}

pub struct Object {
    pub class: &'static str,
    /// A class to instantiate where `class` does not exist (UIColor's NSColor).
    pub fallback: Option<&'static str>,
    /// When set, later `Value::Ref`s with this name refer to this object.
    pub name: Option<String>,
    pub values: Vec<(&'static str, Value)>,
}

impl Object {
    pub fn new(class: &'static str, values: Vec<(&'static str, Value)>) -> Object {
        Object { class, fallback: None, name: None, values }
    }

    pub fn named(mut self, name: impl Into<String>) -> Object {
        self.name = Some(name.into());
        self
    }

    pub fn with_fallback(mut self, class: &'static str) -> Object {
        self.fallback = Some(class);
        self
    }
}

/// A reference to the object named `name`.
pub fn reference(name: impl Into<String>) -> Value {
    Value::Ref(name.into())
}

enum Stored {
    Int(i64),
    Bool(bool),
    Float(f32),
    Double(f64),
    Data(Vec<u8>),
    Nil,
    Object(u32),
}

struct Encoder {
    /// (class, values) per object number.
    objects: Vec<(&'static str, Option<&'static str>, Vec<(usize, Stored)>)>,
    keys: Vec<&'static str>,
    key_index: HashMap<&'static str, usize>,
    names: HashMap<String, u32>,
    /// References to objects not encoded yet: (object, value index, name),
    /// resolved once everything is (an object may refer ahead, as a
    /// navigation item to its bar).
    pending: Vec<(u32, usize, String)>,
}

impl Encoder {
    fn key(&mut self, k: &'static str) -> usize {
        if let Some(&i) = self.key_index.get(k) {
            return i;
        }
        self.keys.push(k);
        self.key_index.insert(k, self.keys.len() - 1);
        self.keys.len() - 1
    }

    fn encode(&mut self, o: Object) -> u32 {
        let me = self.objects.len() as u32;
        self.objects.push((o.class, o.fallback, Vec::new()));
        if let Some(n) = o.name {
            self.names.insert(n, me);
        }
        for (k, v) in o.values {
            let ki = self.key(k);
            let stored = match v {
                Value::Int(i) => Stored::Int(i),
                Value::Bool(b) => Stored::Bool(b),
                Value::Float(f) => Stored::Float(f),
                Value::Double(d) => Stored::Double(d),
                Value::Data(d) => Stored::Data(d),
                Value::Nil => Stored::Nil,
                Value::New(child) => Stored::Object(self.encode(child)),
                Value::Ref(name) => match self.names.get(&name) {
                    Some(&n) => Stored::Object(n),
                    None => {
                        let at = self.objects[me as usize].2.len();
                        self.pending.push((me, at, name));
                        Stored::Object(u32::MAX)
                    }
                },
            };
            self.objects[me as usize].2.push((ki, stored));
        }
        me
    }
}

fn varint(out: &mut Vec<u8>, mut v: usize) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b | 0x80);
            return;
        }
        out.push(b);
    }
}

pub fn archive(root: Object) -> Vec<u8> {
    let mut e = Encoder { objects: Vec::new(), keys: Vec::new(), key_index: HashMap::new(), names: HashMap::new(), pending: Vec::new() };
    e.encode(root);
    for (object, at, name) in std::mem::take(&mut e.pending) {
        let n = *e.names.get(&name).unwrap_or_else(|| panic!("no object named {name}"));
        e.objects[object as usize].2[at].1 = Stored::Object(n);
    }

    let mut classes: Vec<&'static str> = Vec::new();
    for (c, _, _) in &e.objects {
        if !classes.contains(c) {
            classes.push(c);
        }
    }
    let mut fallbacks: Vec<(&'static str, &'static str)> = Vec::new();
    for (c, f, _) in &e.objects {
        if let Some(f) = f {
            if !fallbacks.iter().any(|x| x.0 == *c) {
                fallbacks.push((c, f));
            }
        }
    }
    // Fallback classes follow in byte order (NSColor, NSFont, UIView, for
    // UIColor, UIFont and a cell's content view used in any order).
    let mut missing: Vec<&'static str> = Vec::new();
    for (_, f) in &fallbacks {
        if !classes.contains(f) && !missing.contains(f) {
            missing.push(f);
        }
    }
    missing.sort();
    classes.extend(missing);

    let order = cf::hash::set_order(&e.keys);
    let mut key_position = vec![0usize; e.keys.len()];
    for (pos, &k) in order.iter().enumerate() {
        key_position[k] = pos;
    }

    let mut objects = Vec::new();
    let mut values = Vec::new();
    let mut first = 0usize;
    let mut nvalues = 0usize;
    for (c, _, vs) in &e.objects {
        varint(&mut objects, classes.iter().position(|x| x == c).unwrap());
        varint(&mut objects, first);
        varint(&mut objects, vs.len());
        first += vs.len();
        for (k, v) in vs {
            varint(&mut values, key_position[*k]);
            match v {
                Stored::Int(i) => {
                    // Negative values are 8 bytes wide.
                    if *i < 0 {
                        values.push(3);
                        values.extend_from_slice(&i.to_le_bytes());
                    } else if (-128..=127).contains(i) {
                        values.push(0);
                        values.push(*i as i8 as u8);
                    } else if (-32768..=32767).contains(i) {
                        values.push(1);
                        values.extend_from_slice(&(*i as i16).to_le_bytes());
                    } else if (i32::MIN as i64..=i32::MAX as i64).contains(i) {
                        values.push(2);
                        values.extend_from_slice(&(*i as i32).to_le_bytes());
                    } else {
                        values.push(3);
                        values.extend_from_slice(&i.to_le_bytes());
                    }
                }
                Stored::Bool(false) => values.push(4),
                Stored::Bool(true) => values.push(5),
                Stored::Float(f) => {
                    values.push(6);
                    values.extend_from_slice(&f.to_le_bytes());
                }
                Stored::Double(d) => {
                    values.push(7);
                    values.extend_from_slice(&d.to_le_bytes());
                }
                Stored::Data(d) => {
                    values.push(8);
                    varint(&mut values, d.len());
                    values.extend_from_slice(d);
                }
                Stored::Nil => values.push(9),
                Stored::Object(o) => {
                    values.push(10);
                    values.extend_from_slice(&o.to_le_bytes());
                }
            }
            nvalues += 1;
        }
    }
    let mut keys = Vec::new();
    for &k in &order {
        varint(&mut keys, e.keys[k].len());
        keys.extend_from_slice(e.keys[k].as_bytes());
    }
    let mut class_table = Vec::new();
    for c in &classes {
        varint(&mut class_table, c.len() + 1);
        let fallback = fallbacks.iter().find(|x| x.0 == *c).map(|x| classes.iter().position(|y| *y == x.1).unwrap());
        match fallback {
            Some(f) => {
                varint(&mut class_table, 1);
                class_table.extend_from_slice(&(f as i32).to_le_bytes());
            }
            None => varint(&mut class_table, 0),
        }
        class_table.extend_from_slice(c.as_bytes());
        class_table.push(0);
    }

    let header = 10 + 40;
    let objects_offset = header;
    let keys_offset = objects_offset + objects.len();
    let values_offset = keys_offset + keys.len();
    let classes_offset = values_offset + values.len();
    let mut out = b"NIBArchive".to_vec();
    for v in [1usize, 10, e.objects.len(), objects_offset, e.keys.len(), keys_offset, nvalues, values_offset, classes.len(), classes_offset] {
        out.extend_from_slice(&(v as u32).to_le_bytes());
    }
    out.extend(objects);
    out.extend(keys);
    out.extend(values);
    out.extend(class_table);
    out.extend_from_slice(b"LNE\0");
    out
}
