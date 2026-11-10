//! MSL type names and sizes, from AST type strings and clang's record layouts.

use crate::ast::Program;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

pub fn scalar(t: &str) -> Option<(u32, u32)> {
    Some(match t {
        "bfloat" | "short" | "ushort" | "half" => (2, 2),
        "bool" | "char" | "uchar" => (1, 1),
        "int" | "uint" | "float" => (4, 4),
        "long" | "ulong" => (8, 8),
        _ => return None,
    })
}

fn canon(t: &str) -> &str {
    match t {
        "unsigned int" => "uint",
        "unsigned char" => "uchar",
        "unsigned short" => "ushort",
        "unsigned long" => "ulong",
        "signed char" => "char",
        "_Float16" => "half",
        "_Bool" => "bool",
        _ => t,
    }
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub size: u32,
    pub align: u32,
    /// (offset, type, name)
    pub fields: Vec<(u32, String, String)>,
}

/// `-fdump-record-layouts` output -> layouts by record name.
pub fn record_layouts(text: &str) -> HashMap<String, Layout> {
    let mut out = HashMap::new();
    for block in text.split("*** Dumping AST Record Layout").skip(1) {
        let mut lines = block.trim().lines();
        let Some(head) = lines.next().and_then(|l| regex!(r"^\s*0 \| (?:(?:struct|class|union) )?(.*)$").captures(l)) else { continue };
        let name = head[1].trim().to_string();
        let mut fields = Vec::new();
        for l in lines {
            // direct members only (three spaces)
            if let Some(c) = regex!(r"^\s*(\d+) \|   (\S.*)$").captures(l) {
                if let Some(d) = regex!(r"^(.*?)\s*(\w+)$").captures(&c[2]) {
                    fields.push((c[1].parse().unwrap(), d[1].replace("struct ", "").replace("class ", "").trim().to_string(), d[2].to_string()));
                }
            }
        }
        if let Some(sm) = regex!(r"\[sizeof=(\d+), dsize=\d+, align=(\d+)").captures(block) {
            out.insert(name, Layout { size: sm[1].parse().unwrap(), align: sm[2].parse().unwrap(), fields });
        }
    }
    out
}

pub fn strip_qual(t: &str) -> String {
    let t = regex!(r"\b(device|constant|threadgroup|thread|const|volatile)\b").replace_all(t, "");
    let t = regex!(r"__attribute__\(\(address_space\(\d\)\)\)").replace_all(&t, "");
    let t = t.replace("metal::", "");
    regex!(r"\s+").replace_all(&t, " ").trim().to_string()
}

pub struct Types<'a> {
    pub program: &'a Program,
    pub layouts: HashMap<String, Layout>,
}

impl<'a> Types<'a> {
    /// Expand known typedef names inside a type string.
    pub fn resolve(&self, t: &str) -> String {
        let mut t = t.to_string();
        for _ in 0..8 {
            let new = regex!(r"(?:\w+::)*\w+")
                .replace_all(&t, |c: &regex::Captures| {
                    let w = &c[0];
                    match self.program.typedefs.get(w) {
                        Some(v) if !regex!(r"^(?:metal::)?(?:u?(?:char|short|int|long)|half|float|bool)[234]?$").is_match(w) => v.clone(),
                        _ => w.to_string(),
                    }
                })
                .to_string();
            if new == t {
                return t;
            }
            t = new;
        }
        t
    }

    /// The name AIR's metadata uses: float4, uint, packed_float3, float4x4, a struct's name.
    pub fn msl_name(&self, t: &str) -> String {
        let t = strip_qual(t);
        if let Some(c) = regex!(r"^(\w[\w ]*?) __attribute__\(\(ext_vector_type\((\d)\)\)\)$").captures(&t) {
            return format!("{}{}", canon(&c[1]), &c[2]);
        }
        let builtin = regex!(r"^(\w+?)([234])(x[234])?$").captures(&t).map(|c| scalar(&c[1]).is_some()).unwrap_or(false);
        if let Some(r) = self.program.typedefs.get(&t) {
            if scalar(&t).is_none() && !builtin {
                let r = strip_qual(r);
                if (r.contains("ext_vector_type") || regex!(r"^(unsigned |signed )?\w+$").is_match(&r)) && r != t {
                    return self.msl_name(&r);
                }
            }
        }
        if let Some(c) = regex!(r"^matrix<(\w[\w ]*), (\d), (\d)>$").captures(&t) {
            return format!("{}{}x{}", canon(&c[1]), &c[2], &c[3]);
        }
        if let Some(c) = regex!(r"^__packed<(\w[\w ]*), (\d)>$").captures(&t) {
            return format!("packed_{}{}", canon(&c[1]), &c[2]);
        }
        canon(&t).to_string()
    }

    pub fn size(&self, name: &str) -> Result<(u32, u32), String> {
        let name = self.msl_name(name);
        if let Some(s) = scalar(&name) {
            return Ok(s);
        }
        if let Some(c) = regex!(r"^(\w+?)([234])$").captures(&name) {
            if let Some((s, _)) = scalar(&c[1]) {
                let n: u32 = c[2].parse().unwrap();
                let n = if n == 3 { 4 } else { n };
                return Ok((s * n, s * n));
            }
        }
        if let Some(c) = regex!(r"^packed_(\w+?)([234])$").captures(&name) {
            if let Some((e, _)) = scalar(&c[1]) {
                return Ok((e * c[2].parse::<u32>().unwrap(), e));
            }
        }
        if let Some(c) = regex!(r"^(\w+?)([234])x([234])$").captures(&name) {
            if let Some((s, _)) = scalar(&c[1]) {
                let rows: u32 = c[3].parse().unwrap();
                let rows = if rows == 3 { 4 } else { rows };
                return Ok((s * rows * c[2].parse::<u32>().unwrap(), s * rows));
            }
        }
        if let Some(c) = regex!(r"^(.*)\[(\d+)\]$").captures(&name) {
            let (s, a) = self.size(&c[1])?;
            return Ok((s * c[2].parse::<u32>().unwrap(), a));
        }
        if let Some(l) = self.layouts.get(&name) {
            return Ok((l.size, l.align));
        }
        let r = self.msl_name(&self.resolve(&name));
        if r != name {
            return self.size(&r);
        }
        Err(format!("unknown size of type {name}"))
    }
}

impl Types<'_> {
    /// Each record's fields as clang's !tbaa.struct lists them for a copy:
    /// (offset, size, the element type of a vector field), nested records flattened.
    pub fn tbaa_struct_fields(&self) -> Vec<Vec<(u32, u32, Option<String>)>> {
        fn walk(t: &Types, name: &str, base: u32, out: &mut Vec<(u32, u32, Option<String>)>, depth: u32) -> bool {
            let Some(l) = t.layouts.get(name) else { return false };
            for (off, ty, _) in &l.fields {
                let ty = t.msl_name(&t.resolve(ty));
                if depth < 8 && t.layouts.contains_key(&ty) {
                    if !walk(t, &ty, base + off, out, depth + 1) {
                        return false;
                    }
                    continue;
                }
                let Ok((size, _)) = t.size(&ty) else { return false };
                let elem = regex!(r"^(\w+?)([234])$").captures(&ty).filter(|c| scalar(&c[1]).is_some()).map(|c| c[1].to_string());
                out.push((base + off, size, elem));
            }
            true
        }
        let mut all = Vec::new();
        for name in self.layouts.keys() {
            let mut f = Vec::new();
            if walk(self, name, 0, &mut f, 0) && f.iter().any(|x| x.2.is_some()) {
                all.push(f);
            }
        }
        all
    }
}

/// Itanium mangling of a scalar or vector (function constants, generated keys).
pub fn itanium(t: &str) -> String {
    let code = |s: &str| -> String {
        match s {
            "bool" => "b",
            "char" => "c",
            "uchar" => "h",
            "short" => "s",
            "ushort" => "t",
            "int" => "i",
            "uint" => "j",
            "long" => "l",
            "ulong" => "m",
            "half" => "Dh",
            "float" => "f",
            other => other,
        }
        .to_string()
    };
    if let Some(c) = regex!(r"^(\w+?)([234])$").captures(t) {
        if scalar(&c[1]).is_some() {
            return format!("Dv{}_{}", &c[2], code(&c[1]));
        }
    }
    code(t)
}

/// MTLDataType (Metal's public enum).
pub fn mtl_data_type(t: &str) -> u8 {
    let c = regex!(r"^([a-z]+?)([234]?)$").captures(t);
    let Some(c) = c else { return 0 };
    let base = match &c[1] {
        "float" => 3,
        "half" => 16,
        "int" => 29,
        "uint" => 33,
        "short" => 37,
        "ushort" => 41,
        "char" => 45,
        "uchar" => 49,
        "bool" => 53,
        "long" => 81,
        "ulong" => 85,
        _ => return 0,
    };
    base + c[2].parse::<u8>().map(|n| n - 1).unwrap_or(0)
}
