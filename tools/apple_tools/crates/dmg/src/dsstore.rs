//! The Finder's `.DS_Store` for a disk image's window, as the publicly
//! documented format has it (a buddy-allocated file holding one B-tree of
//! records), and the Alias Manager record that points the window's background
//! at a picture on the volume.

/// What the window shows.
pub struct Layout {
    /// The window's position and size: x, y, width, height.
    pub window: (i32, i32, i32, i32),
    pub icon_size: f64,
    pub text_size: f64,
    /// Each item's icon centre, by name.
    pub icons: Vec<(String, i32, i32)>,
    /// The background picture's alias (`alias`), if there is one.
    pub background: Option<Vec<u8>>,
}

fn put16(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_be_bytes());
}
fn put32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// One record: the file it's about, its kind and its value.
fn record(name: &str, code: &[u8; 4], kind: &[u8; 4], value: &[u8]) -> Vec<u8> {
    let mut r = Vec::new();
    let n = utf16(name);
    put32(&mut r, n.len() as u32);
    for c in n {
        put16(&mut r, c);
    }
    r.extend_from_slice(code);
    r.extend_from_slice(kind);
    r.extend_from_slice(value);
    r
}

fn blob(code: &[u8; 4], name: &str, data: &[u8]) -> (String, [u8; 4], Vec<u8>) {
    let mut v = Vec::new();
    put32(&mut v, data.len() as u32);
    v.extend_from_slice(data);
    (name.to_string(), *code, record(name, code, b"blob", &v))
}

fn bplist(d: plist::Dictionary) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    plist::Value::Dictionary(d).to_writer_binary(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// The `.DS_Store` for `layout`.
pub fn write(layout: &Layout) -> Result<Vec<u8>, String> {
    let (x, y, w, h) = layout.window;
    let mut bwsp = plist::Dictionary::new();
    bwsp.insert("ContainerShowSidebar".into(), false.into());
    bwsp.insert("PreviewPaneVisibility".into(), false.into());
    bwsp.insert("ShowPathbar".into(), false.into());
    bwsp.insert("ShowSidebar".into(), false.into());
    bwsp.insert("ShowStatusBar".into(), false.into());
    bwsp.insert("ShowTabView".into(), false.into());
    bwsp.insert("ShowToolbar".into(), false.into());
    bwsp.insert("SidebarWidth".into(), 0.into());
    bwsp.insert("WindowBounds".into(), format!("{{{{{x}, {y}}}, {{{w}, {h}}}}}").into());
    let mut icvp = plist::Dictionary::new();
    icvp.insert("arrangeBy".into(), "none".into());
    icvp.insert("backgroundColorBlue".into(), 1.0.into());
    icvp.insert("backgroundColorGreen".into(), 1.0.into());
    icvp.insert("backgroundColorRed".into(), 1.0.into());
    if let Some(a) = &layout.background {
        icvp.insert("backgroundImageAlias".into(), plist::Value::Data(a.clone()));
    }
    icvp.insert("backgroundType".into(), (if layout.background.is_some() { 2 } else { 0 }).into());
    icvp.insert("gridOffsetX".into(), 0.0.into());
    icvp.insert("gridOffsetY".into(), 0.0.into());
    icvp.insert("gridSpacing".into(), 100.0.into());
    icvp.insert("iconSize".into(), layout.icon_size.into());
    icvp.insert("labelOnBottom".into(), true.into());
    icvp.insert("showIconPreview".into(), true.into());
    icvp.insert("showItemInfo".into(), false.into());
    icvp.insert("textSize".into(), layout.text_size.into());
    icvp.insert("viewOptionsVersion".into(), 1.into());
    let mut records = vec![blob(b"bwsp", ".", &bplist(bwsp)?), blob(b"icvp", ".", &bplist(icvp)?)];
    let mut v = Vec::new();
    put32(&mut v, 1);
    records.push((".".into(), *b"vSrn", record(".", b"vSrn", b"long", &v)));
    for (name, ix, iy) in &layout.icons {
        // the icon's centre, then 0xffffffff ffff0000
        let mut d = Vec::new();
        put32(&mut d, *ix as u32);
        put32(&mut d, *iy as u32);
        d.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0, 0]);
        records.push(blob(b"Iloc", name, &d));
    }
    // the B-tree's order: by name without case, then by code
    records.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()).then(a.1.cmp(&b.1)));
    // one leaf node: no right child, the record count, the records
    let mut leaf = Vec::new();
    put32(&mut leaf, 0);
    put32(&mut leaf, records.len() as u32);
    for (_, _, r) in &records {
        leaf.extend_from_slice(r);
    }
    if leaf.len() > 4096 {
        return Err("the window's records are more than one B-tree node".into());
    }
    // the buddy allocator over 2^31 bytes (offsets from byte 4): the header at 0, the
    // B-tree's information at 32, the allocator's root block at 2048, the leaf at 4096;
    // every other block free, each in its size's list
    const DSDB: u32 = 32;
    const ROOT: u32 = 2048;
    const LEAF: u32 = 4096;
    let mut file = vec![0u8; 4 + 8192];
    file[0..4].copy_from_slice(&1u32.to_be_bytes());
    file[4..8].copy_from_slice(b"Bud1");
    file[8..12].copy_from_slice(&ROOT.to_be_bytes());
    file[12..16].copy_from_slice(&2048u32.to_be_bytes());
    file[16..20].copy_from_slice(&ROOT.to_be_bytes());
    file[20..36].copy_from_slice(&[0, 0, 0x10, 0x0c, 0, 0, 0, 0x87, 0, 0, 0x20, 0x0b, 0, 0, 0, 0]);
    let mut dsdb = Vec::new();
    put32(&mut dsdb, 2); // the root node: block 2
    put32(&mut dsdb, 0); // levels above the leaves
    put32(&mut dsdb, records.len() as u32);
    put32(&mut dsdb, 1); // nodes
    put32(&mut dsdb, 0x1000); // page size
    let mut root = Vec::new();
    // the blocks: address | log2(size)
    let blocks = [ROOT | 11, DSDB | 5, LEAF | 12];
    put32(&mut root, blocks.len() as u32);
    put32(&mut root, 0);
    for b in blocks {
        put32(&mut root, b);
    }
    root.resize(8 + 4 * 256, 0);
    // the table of contents: DSDB is block 1
    put32(&mut root, 1);
    root.push(4);
    root.extend_from_slice(b"DSDB");
    put32(&mut root, 1);
    for size in 0..32u32 {
        let free: Vec<u32> = match size {
            6..=10 => vec![1 << size],
            13..=30 => vec![1 << size],
            _ => vec![],
        };
        put32(&mut root, free.len() as u32);
        for f in free {
            put32(&mut root, f);
        }
    }
    let at = |a: u32| 4 + a as usize;
    file[at(DSDB)..at(DSDB) + dsdb.len()].copy_from_slice(&dsdb);
    file[at(ROOT)..at(ROOT) + root.len()].copy_from_slice(&root);
    file[at(LEAF)..at(LEAF) + leaf.len()].copy_from_slice(&leaf);
    Ok(file)
}

/// Where a file is on its volume, for its alias.
pub struct Target<'a> {
    pub volume: &'a str,
    /// The volume's creation date (HFS seconds).
    pub volume_date: u32,
    /// The path from the volume's root, by component.
    pub path: &'a [&'a str],
    /// The catalog node IDs of each folder on the path (the root's first) and of the file.
    pub folder_ids: &'a [u32],
    pub file_id: u32,
    pub file_date: u32,
}

/// An Alias Manager record (version 2) for a file on a disk image's volume.
pub fn alias(t: &Target) -> Vec<u8> {
    let pascal = |s: &str, len: usize| -> Vec<u8> {
        let b: Vec<u8> = s.bytes().take(len - 1).collect();
        let mut v = vec![b.len() as u8];
        v.extend_from_slice(&b);
        v.resize(len, 0);
        v
    };
    let name = t.path.last().copied().unwrap_or("");
    let parent = *t.folder_ids.last().unwrap_or(&2);
    let mut a = Vec::new();
    put32(&mut a, 0); // application creator
    put16(&mut a, 0); // the record's size, below
    put16(&mut a, 2);
    put16(&mut a, 0); // a file
    a.extend_from_slice(&pascal(t.volume, 28));
    put32(&mut a, t.volume_date);
    a.extend_from_slice(b"H+");
    put16(&mut a, 5); // an ejectable disk
    put32(&mut a, parent);
    a.extend_from_slice(&pascal(name, 64));
    put32(&mut a, t.file_id);
    put32(&mut a, t.file_date);
    a.extend_from_slice(&[0u8; 8]); // type, creator
    put16(&mut a, 0xffff); // levels from
    put16(&mut a, 0xffff); // and to
    put32(&mut a, 0);
    put16(&mut a, 0);
    a.extend_from_slice(&[0u8; 10]);
    let tag = |a: &mut Vec<u8>, t: u16, data: &[u8]| {
        put16(a, t);
        put16(a, data.len() as u16);
        a.extend_from_slice(data);
        if data.len() % 2 == 1 {
            a.push(0);
        }
    };
    let folders = &t.path[..t.path.len().saturating_sub(1)];
    // the parent folder's name; the folders' IDs below the root, innermost first
    tag(&mut a, 0, folders.last().copied().unwrap_or(t.volume).as_bytes());
    let mut ids = Vec::new();
    for id in t.folder_ids.iter().skip(1).rev() {
        put32(&mut ids, *id);
    }
    tag(&mut a, 1, &ids);
    // the Carbon path, the Unicode name and volume name, the POSIX path, the mount point
    let carbon = std::iter::once(t.volume).chain(t.path.iter().copied()).collect::<Vec<_>>().join(":");
    tag(&mut a, 2, carbon.as_bytes());
    let unicode = |s: &str| -> Vec<u8> {
        let u = utf16(s);
        let mut v = Vec::new();
        put16(&mut v, u.len() as u16);
        for c in u {
            put16(&mut v, c);
        }
        v
    };
    tag(&mut a, 14, &unicode(name));
    tag(&mut a, 15, &unicode(t.volume));
    tag(&mut a, 18, format!("/{}", t.path.join("/")).as_bytes());
    tag(&mut a, 19, format!("/Volumes/{}", t.volume).as_bytes());
    put16(&mut a, 0xffff);
    put16(&mut a, 0);
    let size = a.len() as u16;
    a[4..6].copy_from_slice(&size.to_be_bytes());
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_store_has_its_allocator_and_records() {
        let l = Layout { window: (100, 100, 640, 400), icon_size: 128.0, text_size: 16.0, icons: vec![("Demo.app".into(), 160, 200), ("Applications".into(), 480, 200)], background: Some(vec![1, 2, 3]) };
        let f = write(&l).unwrap();
        assert_eq!(&f[4..8], b"Bud1");
        let leaf = &f[4 + 4096..];
        // records: ".", bwsp, icvp, vSrn; then Applications, Demo.app (no case)
        assert_eq!(u32::from_be_bytes(leaf[4..8].try_into().unwrap()), 5);
        let i = leaf.windows(4).position(|w| w == b"Iloc").unwrap();
        let first: Vec<u8> = "Applications".encode_utf16().flat_map(|c| c.to_be_bytes()).collect();
        assert_eq!(&leaf[i - first.len()..i], &first[..]);
    }

    #[test]
    fn an_alias_names_its_file_every_way() {
        let a = alias(&Target { volume: "Demo", volume_date: 1, path: &[".background", "bg.png"], folder_ids: &[2, 18], file_id: 19, file_date: 1 });
        assert_eq!(u16::from_be_bytes(a[4..6].try_into().unwrap()) as usize, a.len());
        assert!(a.windows(19).any(|w| w == b"/.background/bg.png"));
        assert!(a.windows(23).any(|w| w == b"Demo:.background:bg.png"));
    }
}
