//! AIR's argument and output descriptors (FORMAT.md, "Argument descriptors").

use crate::ast::{Function, Param};
use crate::entry::{handle_of, ret_type};
use crate::text::quote as s;
use crate::types::{itanium, scalar, strip_qual, Layout, Types};
use regex::Regex;
use std::sync::LazyLock;

macro_rules! regex {
    ($r:expr) => {{
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());
        &*RE
    }};
}

/// Numbered metadata nodes, in creation order.
#[derive(Default)]
pub struct Md {
    pub nodes: Vec<String>,
}

impl Md {
    pub fn add(&mut self, body: String) -> String {
        self.nodes.push(body);
        format!("!{}", self.nodes.len() - 1)
    }
    pub fn list(&mut self, items: &[String]) -> String {
        self.add(format!("!{{{}}}", items.join(", ")))
    }
}

const BUILTINS: &[&str] = &[
    "thread_position_in_grid", "threads_per_grid", "thread_position_in_threadgroup", "threads_per_threadgroup",
    "threadgroup_position_in_grid", "threadgroups_per_grid", "thread_index_in_threadgroup", "thread_index_in_simdgroup",
    "thread_index_in_quadgroup", "simdgroup_index_in_threadgroup", "quadgroup_index_in_threadgroup", "threads_per_simdgroup",
    "simdgroups_per_threadgroup", "quadgroups_per_threadgroup", "dispatch_threads_per_threadgroup", "grid_origin", "grid_size",
    "vertex_id", "instance_id", "base_vertex", "base_instance", "front_facing", "sample_id", "point_coord", "sample_mask", "primitive_id",
];

const INTERP: &[(&str, &[&str])] = &[
    ("flat", &["air.flat"]),
    ("center_perspective", &["air.center", "air.perspective"]),
    ("center_no_perspective", &["air.center", "air.no_perspective"]),
    ("centroid_perspective", &["air.centroid", "air.perspective"]),
    ("centroid_no_perspective", &["air.centroid", "air.no_perspective"]),
    ("sample_perspective", &["air.sample", "air.perspective"]),
    ("sample_no_perspective", &["air.sample", "air.no_perspective"]),
];

fn interp_of(attrs: &[String]) -> Option<&'static [&'static str]> {
    attrs.iter().find_map(|a| INTERP.iter().find(|(k, _)| k == a).map(|(_, v)| *v))
}

fn slot(a: &str) -> u32 {
    regex!(r"\((\d+)\)").captures(a).map(|c| c[1].parse().unwrap()).unwrap_or(0)
}

/// The address space a pointer/reference type string points into (1 if unspecified).
pub fn aspace_of(t: &str) -> u32 {
    if let Some(c) = regex!(r"address_space\((\d)\)").captures(t) {
        return c[1].parse().unwrap();
    }
    if regex!(r"\bconstant\b").is_match(t) {
        2
    } else if regex!(r"\bthreadgroup\b").is_match(t) {
        3
    } else {
        1
    }
}

fn pointee(t: &str) -> String {
    regex!(r"\s*[*&]\s*$").replace(t, "").to_string()
}

pub struct Descriptors<'a, 'b> {
    pub types: &'b Types<'a>,
    pub air: (u32, u32),
    pub md: Md,
}

impl<'a, 'b> Descriptors<'a, 'b> {
    fn texture_name(&self, t: &str) -> Result<(String, String), String> {
        let st = strip_qual(t);
        let c = regex!(r"^(\w+)<(.*)>$").captures(&st).ok_or_else(|| format!("not a texture type: {t}"))?;
        let targs: Vec<&str> = c[2].split(',').map(str::trim).collect();
        let acc = if targs.len() > 1 {
            targs[1].rsplit("::").next().unwrap().to_string()
        } else if &c[1] == "texture_buffer" {
            "write".to_string()
        } else {
            "sample".to_string()
        };
        Ok((format!("{}<{}, {}>", &c[1], self.types.msl_name(targs[0]), acc), acc))
    }

    fn struct_info(&mut self, tname: &str, layouts: &std::collections::HashMap<String, Layout>) -> Result<String, String> {
        let lay = layouts.get(tname).ok_or_else(|| format!("no layout for {tname}"))?.clone();
        let mut parts = Vec::new();
        for (off, ftype, fname) in &lay.fields {
            let (base, count) = match regex!(r"^(.*)\[(\d+)\]$").captures(ftype) {
                Some(c) => (c[1].trim().to_string(), c[2].parse::<u32>().unwrap()),
                None => (ftype.clone(), 0),
            };
            let bname = self.types.msl_name(&base);
            let size = match layouts.get(&bname) {
                Some(l) => l.size,
                None => self.types.size(&bname)?.0,
            };
            if layouts.contains_key(&bname) && fname != "__s" {
                let inner = self.struct_info(&bname, layouts)?;
                parts.push(format!("!\"air.struct_type_info\", {inner}"));
            }
            parts.push(format!("i32 {off}, i32 {size}, i32 {count}, {}, {}", s(&bname), s(fname)));
        }
        Ok(self.md.list(&parts))
    }

    /// The struct name if `p` is a [[buffer(n)]] argument buffer (members with [[id(n)]]).
    fn argbuf_struct(&self, p: &Param) -> Option<String> {
        if !p.attrs.first().map(|a| a.starts_with("buffer(")).unwrap_or(false) {
            return None;
        }
        let name = self.types.msl_name(&pointee(&p.ty));
        let fields = self.types.program.records.get(&name)?;
        fields.iter().any(|f| f.attrs.iter().any(|a| a.starts_with("id("))).then_some(name)
    }

    fn indirect_info(&mut self, sname: &str) -> Result<String, String> {
        let layouts = self.types.layouts.clone();
        let offsets: std::collections::HashMap<String, u32> = layouts.get(sname).map(|l| l.fields.iter().map(|(o, _, n)| (n.clone(), *o)).collect()).unwrap_or_default();
        let fields = self.types.program.records.get(sname).cloned().unwrap_or_default();
        let mut parts = Vec::new();
        for (k, f) in fields.iter().enumerate() {
            let (ft, fd) = (f.ty.clone(), self.types.resolve(&f.desugared));
            let slot = f.attrs.iter().find(|a| a.starts_with("id(")).map(|a| slot(a)).unwrap_or(k as u32);
            let am = regex!(r"^array<(.*),\s*(\d+)>$").captures(&strip_qual(&fd)).map(|c| (c[1].to_string(), c[2].parse::<u32>().unwrap()));
            let is_handle = handle_of(&fd).is_some() || am.as_ref().map(|(i, _)| handle_of(i).is_some()).unwrap_or(false);
            let (tname, size, d);
            if is_handle {
                let inner = am.as_ref().map(|(i, _)| i.clone()).unwrap_or(fd.clone());
                let count = am.as_ref().map(|(_, n)| *n).unwrap_or(1);
                let (mut tn, desc) = if handle_of(&inner).as_deref() == Some("sampler") {
                    ("sampler".to_string(), vec![s("air.sampler"), "!\"air.location_index\"".into(), format!("i32 {slot}"), format!("i32 {count}")])
                } else {
                    let (tn, acc) = self.texture_name(&inner)?;
                    (tn, vec![s("air.texture"), "!\"air.location_index\"".into(), format!("i32 {slot}"), format!("i32 {count}"), s(&format!("air.{acc}"))])
                };
                if am.is_some() {
                    tn = format!("array<{tn}, {count}>");
                }
                let mut items = vec![format!("i32 {k}")];
                items.extend(desc);
                items.extend(["!\"air.arg_type_name\"".into(), s(&tn), "!\"air.arg_name\"".into(), s(&f.name)]);
                d = self.md.list(&items);
                tname = tn;
                size = 8 * count;
            } else if ft.trim_end().ends_with('*') || ft.trim_end().ends_with('&') {
                let pt = self.types.msl_name(&pointee(&ft));
                let aspace = aspace_of(&ft);
                let (ps, pa) = self.types.size(&pt)?;
                let mut desc = vec![format!("i32 {k}"), s("air.buffer")];
                if ft.trim_end().ends_with('&') && aspace == 2 {
                    desc.extend(["!\"air.buffer_size\"".into(), format!("i32 {ps}")]);
                }
                let ro = aspace == 2 || regex!(r"\bconst\b").is_match(&pointee(&ft));
                desc.extend(["!\"air.location_index\"".into(), format!("i32 {slot}"), "i32 1".into(), s(if ro { "air.read" } else { "air.read_write" })]);
                if self.air >= (2, 5) {
                    desc.extend(["!\"air.address_space\"".into(), format!("i32 {aspace}")]);
                }
                if layouts.contains_key(&pt) {
                    let info = self.struct_info(&pt, &layouts)?;
                    desc.extend(["!\"air.struct_type_info\"".into(), info]);
                }
                desc.extend(["!\"air.arg_type_size\"".into(), format!("i32 {ps}"), "!\"air.arg_type_align_size\"".into(), format!("i32 {pa}"), "!\"air.arg_type_name\"".into(), s(&pt), "!\"air.arg_name\"".into(), s(&f.name)]);
                d = self.md.list(&desc);
                tname = pt;
                size = 8;
            } else {
                tname = self.types.msl_name(&ft);
                size = self.types.size(&tname)?.0;
                d = self.md.add(format!(
                    "!{{i32 {k}, !\"air.indirect_constant\", !\"air.location_index\", i32 {slot}, i32 1, !\"air.arg_type_name\", {}, !\"air.arg_name\", {}}}",
                    s(&tname),
                    s(&f.name)
                ));
            }
            let off = offsets.get(&f.name).copied().unwrap_or(0);
            parts.push(format!("i32 {off}, i32 {size}, i32 0, {}, {}, !\"air.indirect_argument\", {d}", s(&tname), s(&f.name)));
        }
        Ok(self.md.list(&parts))
    }

    pub fn argument(&mut self, i: usize, p: &Param, used: bool) -> Result<String, String> {
        let a0 = p.attrs.first().cloned().unwrap_or_default();
        let kind = regex!(r"^(\w+)").captures(&a0).map(|c| c[1].to_string()).unwrap_or_default();
        let t = p.ty.clone();
        let mut f: Vec<String>;
        if let Some(sname) = self.argbuf_struct(p) {
            let aspace = aspace_of(&t);
            let l = self.types.layouts.get(&sname).ok_or_else(|| format!("no layout for {sname}"))?.clone();
            f = vec![format!("i32 {i}"), s("air.indirect_buffer"), "!\"air.buffer_size\"".into(), format!("i32 {}", l.size), "!\"air.location_index\"".into(), format!("i32 {}", slot(&a0)), "i32 1".into(), s(if aspace == 2 { "air.read" } else { "air.read_write" })];
            if self.air >= (2, 5) {
                f.extend(["!\"air.address_space\"".into(), format!("i32 {aspace}")]);
            }
            let info = self.indirect_info(&sname)?;
            f.extend(["!\"air.struct_type_info\"".into(), info, "!\"air.arg_type_size\"".into(), format!("i32 {}", l.size), "!\"air.arg_type_align_size\"".into(), format!("i32 {}", l.align), "!\"air.arg_type_name\"".into(), s(&sname), "!\"air.arg_name\"".into(), s(&p.name)]);
        } else if kind == "buffer" || kind == "threadgroup" {
            let reference = t.trim_end().ends_with('&');
            let aspace = if kind == "threadgroup" { 3 } else { aspace_of(&t) };
            let pt = pointee(&t);
            let readonly = aspace == 2 || regex!(r"\bconst\b").is_match(&pt);
            let mut tname = self.types.msl_name(&pt);
            let mut layouts = self.types.layouts.clone();
            let atomic = regex!(r"^_atomic<(\w[\w ]*)>").captures(&strip_qual(&pointee(&self.types.resolve(&p.desugared)))).map(|c| c[1].to_string());
            let (size, align) = if let Some(elem) = atomic {
                let elem = self.types.msl_name(&elem);
                let (sz, al) = scalar(&elem).ok_or_else(|| format!("atomic of {elem}"))?;
                tname = "metal::_atomic".into();
                layouts.insert(tname.clone(), Layout { size: sz, align: al, fields: vec![(0, elem, "__s".into())] });
                (sz, al)
            } else {
                self.types.size(&tname)?
            };
            f = vec![format!("i32 {i}"), s("air.buffer")];
            if reference && aspace == 2 {
                f.extend(["!\"air.buffer_size\"".into(), format!("i32 {size}")]);
            }
            f.extend(["!\"air.location_index\"".into(), format!("i32 {}", slot(&a0)), "i32 1".into(), s(if readonly { "air.read" } else { "air.read_write" })]);
            if self.air >= (2, 5) {
                f.extend(["!\"air.address_space\"".into(), format!("i32 {aspace}")]);
            }
            if layouts.contains_key(&tname) {
                let info = self.struct_info(&tname, &layouts)?;
                f.extend(["!\"air.struct_type_info\"".into(), info]);
            }
            f.extend(["!\"air.arg_type_size\"".into(), format!("i32 {size}"), "!\"air.arg_type_align_size\"".into(), format!("i32 {align}"), "!\"air.arg_type_name\"".into(), s(&tname), "!\"air.arg_name\"".into(), s(&p.name)]);
        } else if kind == "texture" {
            let (tname, acc) = self.texture_name(&self.types.resolve(&p.desugared))?;
            f = vec![format!("i32 {i}"), s("air.texture"), "!\"air.location_index\"".into(), format!("i32 {}", slot(&a0)), "i32 1".into(), s(&format!("air.{acc}")), "!\"air.arg_type_name\"".into(), s(&tname), "!\"air.arg_name\"".into(), s(&p.name)];
        } else if kind == "sampler" {
            f = vec![format!("i32 {i}"), s("air.sampler"), "!\"air.location_index\"".into(), format!("i32 {}", slot(&a0)), "i32 1".into(), "!\"air.arg_type_name\"".into(), s("sampler"), "!\"air.arg_name\"".into(), s(&p.name)];
        } else if kind == "position" {
            f = vec![format!("i32 {i}"), s("air.position"), s("air.center"), s("air.no_perspective"), "!\"air.arg_type_name\"".into(), s(&self.types.msl_name(&t)), "!\"air.arg_name\"".into(), s(&p.name)];
        } else if kind == "color" {
            f = vec![format!("i32 {i}"), s("air.render_target"), format!("i32 {}", slot(&a0)), "!\"air.arg_type_name\"".into(), s(&self.types.msl_name(&t)), "!\"air.arg_name\"".into(), s(&p.name)];
        } else if BUILTINS.contains(&kind.as_str()) {
            f = vec![format!("i32 {i}"), s(&format!("air.{kind}")), "!\"air.arg_type_name\"".into(), s(&self.types.msl_name(&t)), "!\"air.arg_name\"".into(), s(&p.name)];
        } else {
            return Err(format!("unsupported argument {} ({})", p.name, a0));
        }
        if !used && self.air >= (2, 5) {
            f.push(s("air.arg_unused"));
        }
        Ok(self.md.list(&f))
    }

    fn generated_key(&self, fld: &Param) -> String {
        format!("generated({}{}{})", fld.name.len(), fld.name, itanium(&self.types.msl_name(&fld.ty)))
    }

    pub fn stage_in(&mut self, i: usize, fld: &Param, stage: &str, used: bool) -> String {
        let (t, a) = (self.types.msl_name(&fld.ty), &fld.attrs);
        let tail = vec!["!\"air.arg_type_name\"".to_string(), s(&t), "!\"air.arg_name\"".into(), s(&fld.name)];
        let mut f;
        if stage == "vertex" {
            let loc = a.iter().find_map(|x| regex!(r"^attribute\((\d+)\)").captures(x).map(|c| c[1].to_string())).unwrap_or("0".into());
            f = vec![format!("i32 {i}"), s("air.vertex_input"), "!\"air.location_index\"".into(), format!("i32 {loc}"), "i32 1".into()];
            if self.air <= (2, 1) {
                // 2.1 has it, 2.3 doesn't; 2.2 unprobed
                f.push(s("generated(__air_placeholder__)"));
            }
            f.extend(tail);
        } else if a.iter().any(|x| x == "position") {
            f = vec![format!("i32 {i}"), s("air.position"), s("air.center"), s("air.no_perspective")];
            f.extend(tail);
        } else {
            f = vec![format!("i32 {i}"), s("air.fragment_input"), s(&self.generated_key(fld))];
            f.extend(interp_of(a).unwrap_or(&["air.center", "air.perspective"]).iter().map(|x| s(x)));
            f.extend(tail);
        }
        if !used && self.air >= (2, 5) {
            f.push(s("air.arg_unused"));
        }
        self.md.list(&f)
    }

    pub fn outputs(&mut self, f: &Function) -> String {
        let mut out = Vec::new();
        match self.types.ret_fields(f) {
            None => {
                let t = self.types.msl_name(&ret_type(f));
                if f.stage == "fragment" {
                    out.push(self.md.add(format!("!{{!\"air.render_target\", i32 0, i32 0, !\"air.arg_type_name\", {}}}", s(&t))));
                } else if f.stage == "vertex" && t != "void" {
                    out.push(self.md.add(format!("!{{!\"air.position\", !\"air.arg_type_name\", {}}}", s(&t))));
                }
            }
            Some(rf) => {
                for fld in rf {
                    let (t, a) = (self.types.msl_name(&fld.ty), &fld.attrs);
                    let tail = format!("!\"air.arg_type_name\", {}, !\"air.arg_name\", {}", s(&t), s(&fld.name));
                    let color = a.iter().find_map(|x| regex!(r"^color\((\d+)\)").captures(x).map(|c| c[1].to_string()));
                    let depth = a.iter().find_map(|x| regex!(r"^depth\((\w+)\)").captures(x).map(|c| c[1].to_string()));
                    if a.iter().any(|x| x == "position") {
                        out.push(self.md.add(format!("!{{!\"air.position\", {tail}}}")));
                    } else if a.iter().any(|x| x == "point_size") {
                        out.push(self.md.add(format!("!{{!\"air.point_size\", {tail}}}")));
                    } else if let Some(c) = color {
                        let idx = a.iter().find_map(|x| regex!(r"^index\((\d+)\)").captures(x).map(|c| c[1].to_string())).unwrap_or("0".into());
                        out.push(self.md.add(format!("!{{!\"air.render_target\", i32 {c}, i32 {idx}, {tail}}}")));
                    } else if let Some(d) = depth {
                        out.push(self.md.add(format!("!{{!\"air.depth\", !\"air.depth_qualifier\", {}, {tail}}}", s(&format!("air.{d}")))));
                    } else {
                        let q = if self.air <= (2, 1) { interp_of(a) } else { None };
                        let qs: String = q.map(|q| q.iter().map(|x| s(x) + ", ").collect()).unwrap_or_default();
                        let key = self.generated_key(fld);
                        out.push(self.md.add(format!("!{{!\"air.vertex_output\", {}, {qs}{tail}}}", s(&key))));
                    }
                }
            }
        }
        self.md.list(&out)
    }
}
