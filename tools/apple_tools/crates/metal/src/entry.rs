//! Entry points: extern "C" functions with AIR's signatures that call the user's
//! (flat, address-space-free) function. Textures and samplers become AIR's opaque
//! pointers; a [[stage_in]] struct's fields become parameters; a vertex or
//! fragment output struct becomes a packed return.

use crate::ast::{Function, Param, Program};
use crate::rewrite::{AS1, AS2, AS3};
use crate::types::{strip_qual, Types};
use regex::Regex;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

/// MSL handle types -> the AIR kind of their opaque pointee.
pub const TEXTURES: &[(&str, &str)] = &[
    ("texture1d", "texture_1d"),
    ("texture1d_array", "texture_1d_array"),
    ("texture2d", "texture_2d"),
    ("texture2d_array", "texture_2d_array"),
    ("texture3d", "texture_3d"),
    ("texturecube", "texture_cube"),
    ("texturecube_array", "texture_cube_array"),
    ("texture2d_ms", "texture_2d_ms"),
    ("texture_buffer", "texture_buffer_1d"),
    ("depth2d", "depth_2d"),
    ("depth2d_array", "depth_2d_array"),
    ("depthcube", "depth_cube"),
    ("depth2d_ms", "depth_2d_ms"),
];

pub fn handle_type(kind: &str) -> Option<String> {
    if kind == "sampler" {
        return Some(format!("{AS2} _sampler_t *"));
    }
    TEXTURES.iter().find(|(m, _)| *m == kind).map(|(_, a)| format!("{AS1} _{a}_t *"))
}

/// The handle kind (texture2d, sampler, ...) of a type, if it is one.
pub fn handle_of(t: &str) -> Option<String> {
    let s = strip_qual(t);
    let k = regex!(r"^(\w+)").captures(&s)?[1].to_string();
    handle_type(&k).map(|_| k)
}

pub fn ret_type(f: &Function) -> String {
    f.ty.split('(').next().unwrap().trim().to_string()
}

impl<'a> Types<'a> {
    /// The fields of a vertex or fragment function's output struct.
    pub fn ret_fields(&self, f: &Function) -> Option<&'a Vec<Param>> {
        if f.stage == "kernel" {
            return None;
        }
        self.program.records.get(&self.msl_name(&ret_type(f)))
    }

    /// AIR's parameter list: a [[stage_in]] struct's fields replace the struct.
    pub fn flat_params(&self, f: &Function) -> Vec<Param> {
        let mut out = Vec::new();
        for p in &f.params {
            if p.attrs.iter().any(|a| a == "stage_in") {
                for fld in self.program.records.get(&self.msl_name(&p.ty)).into_iter().flatten() {
                    let mut q = fld.clone();
                    q.stage_in = true;
                    out.push(q);
                }
            } else {
                out.push(p.clone());
            }
        }
        out
    }
}

fn explicit_as(t: &str) -> String {
    regex!(r"\b(device|constant|threadgroup)\b")
        .replace_all(t, |c: &regex::Captures| match &c[1] {
            "device" => AS1,
            "constant" => AS2,
            _ => AS3,
        })
        .to_string()
}

fn flat(t: &str) -> String {
    let t = regex!(r"__attribute__\(\(address_space\(\d\)\)\)").replace_all(t, "");
    let t = regex!(r"\b(device|constant|threadgroup|thread)\b").replace_all(&t, "");
    regex!(r"\s+").replace_all(&t, " ").trim().to_string()
}

pub fn entry_points(types: &Types, program: &Program) -> String {
    let mut out = vec!["namespace __air_entry {".to_string()];
    for f in &program.functions {
        let (mut ps, mut args) = (Vec::new(), Vec::new());
        for p in &f.params {
            let t = p.ty.trim();
            if p.attrs.iter().any(|a| a == "stage_in") {
                let sname = types.msl_name(t);
                let fields = program.records.get(&sname).cloned().unwrap_or_default();
                for fld in &fields {
                    ps.push(format!("{} __si_{}", fld.ty, fld.name));
                }
                args.push(format!("{}{{{}}}", sname, fields.iter().map(|f| format!("__si_{}", f.name)).collect::<Vec<_>>().join(", ")));
            } else if let Some(h) = handle_of(&p.desugared) {
                ps.push(format!("{}{}", handle_type(&h).unwrap(), p.name));
                args.push(format!("{}{{{}}}", p.desugared, p.name));
            } else if t.ends_with('*') {
                ps.push(format!("{} {}", explicit_as(t), p.name));
                args.push(format!("({}){}", flat(t), p.name));
            } else if t.ends_with('&') {
                ps.push(format!("{} {}", explicit_as(t), p.name));
                args.push(format!("*({} *)&{}", flat(&t[..t.len() - 1]), p.name));
            } else {
                ps.push(format!("{} {}", t, p.name));
                args.push(p.name.clone());
            }
        }
        let ret = ret_type(f);
        let call = format!("::{}{}({})", f.ns, f.call, args.join(", "));
        match types.ret_fields(f) {
            Some(rf) => {
                out.push(format!(
                    "struct __attribute__((packed)) __ret_{} {{ {} }};",
                    f.name,
                    rf.iter().enumerate().map(|(k, fld)| format!("{} f{};", fld.ty, k)).collect::<Vec<_>>().join(" ")
                ));
                let copies = rf.iter().enumerate().map(|(k, fld)| format!("__r.f{} = __o.{};", k, fld.name)).collect::<Vec<_>>().join(" ");
                out.push(format!("extern \"C\" __ret_{n} {n}({}) {{ {ret} __o = {call}; __ret_{n} __r; {copies} return __r; }}", ps.join(", "), n = f.name));
            }
            None => out.push(format!("extern \"C\" {ret} {}({}) {{ return {call}; }}", f.name, ps.join(", "))),
        }
    }
    out.push("}".to_string());
    out.join("\n") + "\n"
}
