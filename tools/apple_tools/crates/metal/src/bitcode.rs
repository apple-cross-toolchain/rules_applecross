//! LLVM 15's bitcode as Xcode's Metal compiler writes it. llvm-as writes the
//! same records for the same IR but for what identifies the writer: the
//! identification string (Xcode's is empty), the producer in the symbol table
//! ("32023.922.4"), and six metadata kinds Xcode's LLVM doesn't register. So the
//! stream is parsed (the generic bitstream: blocks, abbreviations, records),
//! those records edited, and written again with the same abbreviations, the
//! offsets into it (VSTOFFSET, the function entries) following the blocks.

/// Which of Xcode's writers: the front end's (`metal -c`) or metallib's, whose
/// LLVMs register different metadata kinds past annotation (30).
#[derive(Clone, Copy)]
pub enum Writer {
    Frontend,
    Metallib,
}

impl Writer {
    /// LLVM 15's kinds this writer doesn't register.
    fn dropped(self) -> &'static [&'static str] {
        match self {
            Writer::Frontend => &["nosanitize", "func_sanitize", "exclude", "memprof", "callsite", "heapallocsite"],
            Writer::Metallib => &["nosanitize", "func_sanitize", "exclude", "memprof", "callsite"],
        }
    }
    /// Kinds it registers after LLVM's own.
    fn added(self) -> &'static [&'static str] {
        match self {
            Writer::Frontend => &[],
            // LLVM 15 registers heapallocsite only for some modules; metallib's always has it
            Writer::Metallib => &["heapallocsite", "air.function_groups"],
        }
    }
}

#[derive(Clone, Debug)]
enum Op {
    Literal(u64),
    Fixed(u32),
    Vbr(u32),
    Array,
    Char6,
    Blob,
}

#[derive(Clone, Debug)]
enum Item {
    Block { id: u64, width: u32, items: Vec<Item> },
    Abbrev(Vec<Op>),
    Record { abbrev: Option<usize>, code: u64, ops: Vec<u64>, blob: Option<Vec<u8>> },
}

struct Reader<'a> {
    d: &'a [u8],
    pos: usize,
    blockinfo: std::collections::HashMap<u64, Vec<Vec<Op>>>,
    /// Each block's length-word position in words, in pre-order.
    starts: Vec<usize>,
}

impl Reader<'_> {
    fn bits(&mut self, n: u32) -> Result<u64, String> {
        let mut v = 0u64;
        for i in 0..n {
            let byte = *self.d.get(self.pos / 8).ok_or("bitcode: truncated")?;
            v |= (((byte >> (self.pos % 8)) & 1) as u64) << i;
            self.pos += 1;
        }
        Ok(v)
    }
    fn vbr(&mut self, n: u32) -> Result<u64, String> {
        let (mut v, mut shift) = (0u64, 0);
        loop {
            let c = self.bits(n)?;
            v |= (c & ((1 << (n - 1)) - 1)) << shift;
            if c & (1 << (n - 1)) == 0 {
                return Ok(v);
            }
            shift += n - 1;
        }
    }
    fn align(&mut self) {
        self.pos = self.pos.div_ceil(32) * 32;
    }
    fn field(&mut self, op: &Op) -> Result<u64, String> {
        Ok(match op {
            Op::Literal(v) => *v,
            Op::Fixed(w) => self.bits(*w)?,
            Op::Vbr(w) => self.vbr(*w)?,
            Op::Char6 => self.bits(6)?,
            Op::Array | Op::Blob => return Err("bitcode: nested array or blob".into()),
        })
    }
    fn abbrev(&mut self) -> Result<Vec<Op>, String> {
        let n = self.vbr(5)?;
        let mut ops = Vec::new();
        for _ in 0..n {
            if self.bits(1)? == 1 {
                ops.push(Op::Literal(self.vbr(8)?));
            } else {
                ops.push(match self.bits(3)? {
                    1 => Op::Fixed(self.vbr(5)? as u32),
                    2 => Op::Vbr(self.vbr(5)? as u32),
                    3 => Op::Array,
                    4 => Op::Char6,
                    5 => Op::Blob,
                    e => return Err(format!("bitcode: abbreviation encoding {e}")),
                });
            }
        }
        Ok(ops)
    }
    /// The items of a block (or the top level, id None) up to its END_BLOCK.
    fn items(&mut self, id: Option<u64>, width: u32) -> Result<Vec<Item>, String> {
        let mut abbrevs: Vec<Vec<Op>> = id.and_then(|i| self.blockinfo.get(&i).cloned()).unwrap_or_default();
        let mut items = Vec::new();
        let mut info_target = None;
        loop {
            if id.is_none() && self.pos + 32 > self.d.len() * 8 {
                return Ok(items);
            }
            let a = self.bits(width)?;
            match a {
                0 => {
                    self.align();
                    return Ok(items);
                }
                1 => {
                    let bid = self.vbr(8)?;
                    let w = self.vbr(4)? as u32;
                    self.align();
                    self.starts.push(self.pos / 32);
                    self.bits(32)?;
                    let inner = self.items(Some(bid), w)?;
                    items.push(Item::Block { id: bid, width: w, items: inner });
                }
                2 => {
                    let ops = self.abbrev()?;
                    if id == Some(0) {
                        let t = info_target.ok_or("bitcode: abbreviation before SETBID")?;
                        self.blockinfo.entry(t).or_default().push(ops.clone());
                    } else {
                        abbrevs.push(ops.clone());
                    }
                    items.push(Item::Abbrev(ops));
                }
                3 => {
                    let code = self.vbr(6)?;
                    let n = self.vbr(6)?;
                    let ops = (0..n).map(|_| self.vbr(6)).collect::<Result<Vec<_>, _>>()?;
                    if id == Some(0) && code == 1 {
                        info_target = ops.first().copied();
                    }
                    items.push(Item::Record { abbrev: None, code, ops, blob: None });
                }
                k => {
                    let def = abbrevs.get(k as usize - 4).ok_or(format!("bitcode: no abbreviation {k}"))?.clone();
                    let code = self.field(&def[0])?;
                    let (mut ops, mut blob) = (Vec::new(), None);
                    let mut i = 1;
                    while i < def.len() {
                        match &def[i] {
                            Op::Array => {
                                let n = self.vbr(6)?;
                                for _ in 0..n {
                                    ops.push(self.field(&def[i + 1])?);
                                }
                                i += 1;
                            }
                            Op::Blob => {
                                let n = self.vbr(6)? as usize;
                                self.align();
                                let start = self.pos / 8;
                                blob = Some(self.d.get(start..start + n).ok_or("bitcode: truncated blob")?.to_vec());
                                self.pos += n * 8;
                                self.align();
                            }
                            op => ops.push(self.field(op)?),
                        }
                        i += 1;
                    }
                    items.push(Item::Record { abbrev: Some(k as usize), code, ops, blob });
                }
            }
        }
    }
}

#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    nacc: u32,
    blockinfo: std::collections::HashMap<u64, Vec<Vec<Op>>>,
    starts: Vec<usize>,
}

impl BitWriter {
    fn bits(&mut self, v: u64, n: u32) {
        for i in 0..n {
            self.acc |= ((v >> i) & 1) << self.nacc;
            self.nacc += 1;
            if self.nacc == 8 {
                self.out.push(self.acc as u8);
                self.acc = 0;
                self.nacc = 0;
            }
        }
    }
    fn vbr(&mut self, mut v: u64, n: u32) {
        let hi = 1u64 << (n - 1);
        while v >= hi {
            self.bits((v & (hi - 1)) | hi, n);
            v >>= n - 1;
        }
        self.bits(v, n);
    }
    fn pos(&self) -> usize {
        self.out.len() * 8 + self.nacc as usize
    }
    fn align(&mut self) {
        while self.pos() % 32 != 0 {
            self.bits(0, 1);
        }
    }
    fn field(&mut self, op: &Op, v: u64) {
        match op {
            Op::Literal(_) => {}
            Op::Fixed(w) => self.bits(v, *w),
            Op::Vbr(w) => self.vbr(v, *w),
            Op::Char6 => self.bits(v, 6),
            Op::Array | Op::Blob => unreachable!(),
        }
    }
    fn items(&mut self, id: Option<u64>, width: u32, items: &[Item]) -> Result<(), String> {
        let mut abbrevs: Vec<Vec<Op>> = id.and_then(|i| self.blockinfo.get(&i).cloned()).unwrap_or_default();
        let mut info_target = None;
        for it in items {
            match it {
                Item::Block { id: bid, width: w, items: inner } => {
                    self.bits(1, width);
                    self.vbr(*bid, 8);
                    self.vbr(*w as u64, 4);
                    self.align();
                    let len_at = self.out.len();
                    self.starts.push(len_at / 4);
                    self.bits(0, 32);
                    self.items(Some(*bid), *w, inner)?;
                    self.bits(0, *w);
                    self.align();
                    let words = (self.out.len() - len_at) / 4 - 1;
                    self.out[len_at..len_at + 4].copy_from_slice(&(words as u32).to_le_bytes());
                }
                Item::Abbrev(ops) => {
                    self.bits(2, width);
                    self.vbr(ops.len() as u64, 5);
                    for op in ops {
                        match op {
                            Op::Literal(v) => {
                                self.bits(1, 1);
                                self.vbr(*v, 8);
                            }
                            Op::Fixed(w) | Op::Vbr(w) => {
                                self.bits(0, 1);
                                self.bits(if matches!(op, Op::Fixed(_)) { 1 } else { 2 }, 3);
                                self.vbr(*w as u64, 5);
                            }
                            Op::Array => {
                                self.bits(0, 1);
                                self.bits(3, 3);
                            }
                            Op::Char6 => {
                                self.bits(0, 1);
                                self.bits(4, 3);
                            }
                            Op::Blob => {
                                self.bits(0, 1);
                                self.bits(5, 3);
                            }
                        }
                    }
                    if id == Some(0) {
                        self.blockinfo.entry(info_target.ok_or("bitcode: abbreviation before SETBID")?).or_default().push(ops.clone());
                    } else {
                        abbrevs.push(ops.clone());
                    }
                }
                Item::Record { abbrev: None, code, ops, .. } => {
                    self.bits(3, width);
                    self.vbr(*code, 6);
                    self.vbr(ops.len() as u64, 6);
                    for v in ops {
                        self.vbr(*v, 6);
                    }
                    if id == Some(0) && *code == 1 {
                        info_target = ops.first().copied();
                    }
                }
                Item::Record { abbrev: Some(k), code, ops, blob } => {
                    let def = abbrevs.get(k - 4).ok_or(format!("bitcode: no abbreviation {k}"))?.clone();
                    self.bits(*k as u64, width);
                    self.field(&def[0], *code);
                    let (mut i, mut j) = (1, 0);
                    while i < def.len() {
                        match &def[i] {
                            Op::Array => {
                                self.vbr((ops.len() - j) as u64, 6);
                                while j < ops.len() {
                                    self.field(&def[i + 1], ops[j]);
                                    j += 1;
                                }
                                i += 1;
                            }
                            Op::Blob => {
                                let b = blob.as_deref().unwrap_or_default();
                                self.vbr(b.len() as u64, 6);
                                self.align();
                                for x in b {
                                    self.bits(*x as u64, 8);
                                }
                                self.align();
                            }
                            op => {
                                self.field(op, ops.get(j).copied().unwrap_or(0));
                                j += 1;
                            }
                        }
                        i += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

fn parse(bc: &[u8]) -> Result<(Vec<Item>, Vec<usize>), String> {
    if bc.get(..4) != Some(b"BC\xc0\xde") {
        return Err("bitcode: no magic".into());
    }
    let mut r = Reader { d: bc, pos: 32, blockinfo: Default::default(), starts: Vec::new() };
    let items = r.items(None, 2)?;
    Ok((items, r.starts))
}

fn write(items: &[Item]) -> Result<(Vec<u8>, Vec<usize>), String> {
    let mut w = BitWriter::default();
    w.out.extend_from_slice(b"BC\xc0\xde");
    w.items(None, 2, items)?;
    Ok((w.out, w.starts))
}

fn kind_name(ops: &[u64]) -> String {
    ops.iter().skip(1).map(|&c| c as u8 as char).collect()
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}

/// The symbol table's producer replaced, and the strings after it moved.
fn producer(symtab: &mut [u8], strtab: &[u8], new: &str) -> Option<Vec<u8>> {
    // irsymtab header: Version, Producer{off,size}, then ranges and strings
    let (off, size) = (u32_at(symtab, 4) as usize, u32_at(symtab, 8) as usize);
    let end = off + size;
    let delta = new.len() as i64 - size as i64;
    let mut s = strtab.get(..off)?.to_vec();
    s.extend_from_slice(new.as_bytes());
    s.extend_from_slice(strtab.get(end..)?);
    symtab[8..12].copy_from_slice(&(new.len() as u32).to_le_bytes());
    let shift = |symtab: &mut [u8], at: usize| {
        let o = u32_at(symtab, at) as usize;
        if o >= end {
            symtab[at..at + 4].copy_from_slice(&((o as i64 + delta) as u32).to_le_bytes());
        }
    };
    // TargetTriple, SourceFileName, COFFLinkerOpts
    for at in [44, 52, 60] {
        shift(symtab, at);
    }
    let range = |symtab: &[u8], at: usize| (u32_at(symtab, at) as usize, u32_at(symtab, at + 4) as usize);
    // Comdats {Name, SelectionKind}, Symbols {Name, IRName, ComdatIndex, Flags},
    // Uncommons {CommonSize, CommonAlign, COFFWeakExternFallbackName, SectionName},
    // DependentLibraries (Str)
    for (at, stride, fields) in [(20usize, 12usize, &[0usize][..]), (28, 24, &[0, 8]), (36, 24, &[8, 16]), (68, 8, &[0])] {
        let (start, n) = range(symtab, at);
        for k in 0..n {
            for f in fields {
                shift(symtab, start + k * stride + f);
            }
        }
    }
    Some(s)
}

/// Rewrite llvm-as's bitcode (with or without the wrapper header) as Xcode writes it.
pub fn as_xcode(bc: &[u8], producer_name: &str, writer: Writer) -> Result<Vec<u8>, String> {
    let wrapped = bc.get(..4) == Some(&[0xde, 0xc0, 0x17, 0x0b]);
    let (head, body) = if wrapped {
        let (off, size) = (u32_at(bc, 8) as usize, u32_at(bc, 12) as usize);
        (&bc[..off], &bc[off..off + size])
    } else {
        (&bc[..0], bc)
    };
    let (mut items, old_starts) = parse(body)?;
    // writing back unchanged must give the same bytes, or the parse missed something
    if write(&items)?.0 != body {
        return Err("bitcode: the stream doesn't round-trip".into());
    }
    let (mut symtab, mut strtab) = (None, None);
    for it in items.iter_mut() {
        let Item::Block { id, items: inner, .. } = it else { continue };
        match *id {
            // IDENTIFICATION: STRING
            13 => {
                for r in inner.iter_mut() {
                    if let Item::Record { code: 1, ops, .. } = r {
                        ops.clear();
                    }
                }
            }
            8 => edit_module(inner, writer)?,
            25 => symtab = Some(it),
            23 => strtab = Some(it),
            _ => {}
        }
    }
    if let (Some(Item::Block { items: sy, .. }), Some(Item::Block { items: st, .. })) = (symtab, strtab) {
        if let (Some(sy), Some(st)) = (blob(sy), blob(st)) {
            *st = producer(sy, st, producer_name).ok_or("bitcode: unexpected symbol table")?;
        }
    }
    // offsets into the stream (in words from its start) follow the blocks they name
    let (_, new_starts) = write(&items)?;
    let moved = |old: u64| -> u64 {
        // the block whose length word is at or just before `old`
        match old_starts.iter().rposition(|&s| s as u64 <= old) {
            Some(i) => (old as i64 + new_starts[i] as i64 - old_starts[i] as i64) as u64,
            None => old,
        }
    };
    for it in items.iter_mut() {
        let Item::Block { id: 8, items: inner, .. } = it else { continue };
        for r in inner.iter_mut() {
            match r {
                // MODULE_CODE_VSTOFFSET
                Item::Record { code: 13, ops, .. } if !ops.is_empty() => ops[0] = moved(ops[0]),
                // the module-level VST: VST_CODE_FNENTRY [valueid, offset]
                Item::Block { id: 14, items: vst, .. } => {
                    for e in vst.iter_mut() {
                        if let Item::Record { code: 3, ops, .. } = e {
                            if ops.len() >= 2 {
                                ops[1] = moved(ops[1]);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let (out, _) = write(&items)?;
    let mut res = head.to_vec();
    if wrapped {
        res[12..16].copy_from_slice(&(out.len() as u32).to_le_bytes());
    }
    res.extend_from_slice(&out);
    if wrapped {
        // the wrapped file is padded to 16 bytes
        res.resize(res.len().div_ceil(16) * 16, 0);
    }
    Ok(res)
}

/// A block's blob record (after its abbreviation definitions).
fn blob(items: &mut [Item]) -> Option<&mut Vec<u8>> {
    items.iter_mut().find_map(|i| match i {
        Item::Record { blob: Some(b), .. } => Some(b),
        _ => None,
    })
}

/// The module's metadata kinds as Xcode's LLVM numbers them.
fn edit_module(items: &mut [Item], writer: Writer) -> Result<(), String> {
    // MODULE_CODE_GLOBALVAR: LLVM 15 appends the sanitizer metadata field Xcode's LLVM doesn't write
    for it in items.iter_mut() {
        if let Item::Record { abbrev: None, code: 7, ops, .. } = it {
            if ops.len() == 19 && ops[18] == 0 {
                ops.pop();
            }
        }
    }
    // old kind id -> new
    let mut renumber: std::collections::HashMap<u64, Option<u64>> = Default::default();
    for it in items.iter_mut() {
        let Item::Block { id: 22, items: kinds, .. } = it else { continue };
        let mut removed = 0;
        let mut keep = Vec::new();
        for k in kinds.drain(..) {
            if let Item::Record { code: 6, ref ops, .. } = k {
                let name = kind_name(ops);
                if writer.dropped().contains(&name.as_str()) {
                    renumber.insert(ops[0], None);
                    removed += 1;
                    continue;
                }
                if removed > 0 {
                    renumber.insert(ops[0], Some(ops[0] - removed));
                }
            }
            keep.push(k);
        }
        let mut last = 0;
        for k in keep.iter_mut() {
            if let Item::Record { code: 6, ops, .. } = k {
                if let Some(Some(n)) = renumber.get(&ops[0]) {
                    ops[0] = *n;
                }
                last = last.max(ops[0]);
            }
        }
        let present: Vec<String> = keep.iter().filter_map(|k| if let Item::Record { code: 6, ops, .. } = k { Some(kind_name(ops)) } else { None }).collect();
        for name in writer.added().iter().filter(|n| !present.iter().any(|p| p == *n)) {
            last += 1;
            let mut ops = vec![last];
            ops.extend(name.bytes().map(u64::from));
            keep.push(Item::Record { abbrev: None, code: 6, ops, blob: None });
        }
        *kinds = keep;
    }
    if renumber.values().any(Option::is_some) {
        renumber_attachments(items, &renumber)?;
    }
    Ok(())
}

/// METADATA_ATTACHMENT records: [instruction?, (kind, node)*].
fn renumber_attachments(items: &mut [Item], map: &std::collections::HashMap<u64, Option<u64>>) -> Result<(), String> {
    for it in items.iter_mut() {
        let Item::Block { id, items: inner, .. } = it else { continue };
        if *id == 16 {
            for r in inner.iter_mut() {
                if let Item::Record { code: 11, ops, .. } = r {
                    let start = ops.len() % 2;
                    for k in (start..ops.len()).step_by(2) {
                        match map.get(&ops[k]) {
                            Some(Some(n)) => ops[k] = *n,
                            Some(None) => return Err("bitcode: an attachment of a kind Xcode's LLVM lacks".into()),
                            None => {}
                        }
                    }
                }
            }
        } else {
            renumber_attachments(inner, map)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stream with BLOCKINFO, an abbreviated record (char6 array and blob) and nested blocks.
    fn sample() -> Vec<Item> {
        vec![
            Item::Block { id: 0, width: 2, items: vec![Item::Record { abbrev: None, code: 1, ops: vec![13], blob: None }, Item::Abbrev(vec![Op::Literal(1), Op::Array, Op::Char6])] },
            Item::Block { id: 13, width: 5, items: vec![Item::Record { abbrev: Some(4), code: 1, ops: b"LLVM15".iter().map(|&c| match c { b'0'..=b'9' => 52 + (c - b'0') as u64, b'A'..=b'Z' => 26 + (c - b'A') as u64, _ => 0 }).collect(), blob: None }] },
            Item::Block {
                id: 8,
                width: 3,
                items: vec![
                    Item::Abbrev(vec![Op::Literal(7), Op::Vbr(6), Op::Fixed(3), Op::Blob]),
                    Item::Record { abbrev: Some(4), code: 7, ops: vec![300, 5], blob: Some(b"abc".to_vec()) },
                    Item::Block { id: 22, width: 3, items: vec![Item::Record { abbrev: None, code: 6, ops: vec![31, 120], blob: None }] },
                ],
            },
        ]
    }

    #[test]
    fn round_trips() {
        let (bytes, _) = write(&sample()).unwrap();
        let (items, _) = parse(&bytes).unwrap();
        assert_eq!(write(&items).unwrap().0, bytes);
    }

    #[test]
    fn identification_string_emptied() {
        let (bytes, _) = write(&sample()).unwrap();
        let out = as_xcode(&bytes, "32023.922.4", Writer::Frontend).unwrap();
        let (items, _) = parse(&out).unwrap();
        let Item::Block { items: ident, .. } = &items[1] else { panic!() };
        assert!(matches!(&ident[0], Item::Record { code: 1, ops, .. } if ops.is_empty()));
    }
}
