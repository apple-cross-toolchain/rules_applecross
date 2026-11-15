//! `dmg`: a Mac disk image from a folder, on any platform (no hdiutil): an
//! HFS+ volume (hfs) in a UDIF image (udif), with the Finder window's layout
//! (dsstore) if one is asked for.
//!
//!     dmg -o Out.dmg --volname Name --srcfolder DIR [--format UDZO|ULFO|UDRO]
//!         [--symlink NAME:TARGET]... [--background PICTURE]
//!         [--window X,Y,W,H] [--icon-size N] [--text-size N] [--icon NAME:X,Y]...
//!         [--hide NAME]... [--open-folder PATH] [--free BYTES]
//!         [--time UNIX_SECONDS] [--uid N] [--gid N]
//!
//! --symlink adds a symbolic link at the top of the volume (Applications:/Applications);
//! --background puts the picture in a hidden .background folder and makes it the
//! window's background; --window, --icon-size, --text-size and --icon lay the
//! window out (a .DS_Store at the top of the volume). The image is the same for
//! the same inputs (--time defaults to SOURCE_DATE_EPOCH, else 0).

mod dsstore;
mod hfs;
mod udif;

use std::path::PathBuf;

fn pair(s: &str, what: &str) -> Result<(String, String), String> {
    s.split_once(':').map(|(a, b)| (a.to_string(), b.to_string())).ok_or_else(|| format!("{what} {s}: NAME:VALUE"))
}

fn numbers(s: &str, n: usize, what: &str) -> Result<Vec<i32>, String> {
    let v: Vec<i32> = s.split(',').map(|x| x.trim().parse::<i32>()).collect::<Result<_, _>>().map_err(|e| format!("{what} {s}: {e}"))?;
    if v.len() != n {
        return Err(format!("{what} {s}: {n} numbers"));
    }
    Ok(v)
}

fn run() -> Result<(), String> {
    let mut out = None;
    let mut name = None;
    let mut src = None;
    let mut format = udif::Format::Zlib;
    let mut time: u64 = std::env::var("SOURCE_DATE_EPOCH").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut hide = Vec::new();
    let mut open = None;
    let mut free = 0u64;
    let (mut uid, mut gid) = (0u32, 0u32);
    let mut symlinks = Vec::new();
    let mut background = None;
    let mut window = None;
    let (mut icon_size, mut text_size) = (128.0, 16.0);
    let mut icons = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{a} takes a value"));
        match a.as_str() {
            "-o" => out = Some(PathBuf::from(value()?)),
            "--volname" => name = Some(value()?),
            "--srcfolder" => src = Some(PathBuf::from(value()?)),
            "--format" => format = udif::Format::parse(&value()?)?,
            "--time" => time = value()?.parse().map_err(|e| format!("--time: {e}"))?,
            "--hide" => hide.push(value()?),
            "--open-folder" => open = Some(value()?),
            "--free" => free = value()?.parse().map_err(|e| format!("--free: {e}"))?,
            "--uid" => uid = value()?.parse().map_err(|e| format!("--uid: {e}"))?,
            "--gid" => gid = value()?.parse().map_err(|e| format!("--gid: {e}"))?,
            "--symlink" => symlinks.push(pair(&value()?, "--symlink")?),
            "--background" => background = Some(PathBuf::from(value()?)),
            "--window" => window = Some(numbers(&value()?, 4, "--window")?),
            "--icon-size" => icon_size = value()?.parse().map_err(|e| format!("--icon-size: {e}"))?,
            "--text-size" => text_size = value()?.parse().map_err(|e| format!("--text-size: {e}"))?,
            "--icon" => {
                let (n, xy) = pair(&value()?, "--icon")?;
                let p = numbers(&xy, 2, "--icon")?;
                icons.push((n, p[0], p[1]));
            }
            _ => return Err(format!("{a}: unknown argument")),
        }
    }
    let out = out.ok_or("-o is required")?;
    let name = name.ok_or("--volname is required")?;
    let src = src.ok_or("--srcfolder is required")?;
    let mut root = hfs::read_dir(&src)?;
    let hfs::Node::Folder { children, .. } = &mut root else { unreachable!() };
    for (n, target) in symlinks {
        children.insert(n, hfs::Node::Symlink { target });
    }
    let picture = match &background {
        Some(p) => {
            let file = p.file_name().ok_or("--background: a file")?.to_string_lossy().to_string();
            let data = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            let mut bg = std::collections::BTreeMap::new();
            bg.insert(file.clone(), hfs::Node::File { data, mode: 0o644, finder_flags: 0, type_creator: None });
            children.insert(".background".into(), hfs::Node::Folder { children: bg, mode: 0o755, finder_flags: hfs::INVISIBLE });
            Some(file)
        }
        None => None,
    };
    let layout = window.is_some() || background.is_some() || !icons.is_empty();
    if layout {
        // its place first (catalog IDs follow names, not contents)
        children.insert(".DS_Store".into(), hfs::Node::File { data: Vec::new(), mode: 0o644, finder_flags: 0, type_creator: None });
    }
    for h in &hide {
        match children.get_mut(h.trim_matches('/')) {
            Some(hfs::Node::Folder { finder_flags, .. }) | Some(hfs::Node::File { finder_flags, .. }) => *finder_flags |= hfs::INVISIBLE,
            _ => return Err(format!("--hide {h}: no such file or folder at the top of the volume")),
        }
    }
    let mut v = hfs::Volume { name, root, time, uid, gid, open_folder: open, free };
    if layout {
        let (_, ids) = hfs::write(&v)?;
        let date = (time + hfs::EPOCH_OFFSET) as u32;
        let background = picture.as_ref().map(|file| {
            let path = [".background", file.as_str()];
            dsstore::alias(&dsstore::Target { volume: &v.name, volume_date: date, path: &path, folder_ids: &[2, ids[".background"]], file_id: ids[&format!(".background/{file}")], file_date: date })
        });
        let w = window.unwrap_or(vec![200, 120, 640, 400]);
        let store = dsstore::write(&dsstore::Layout { window: (w[0], w[1], w[2], w[3]), icon_size, text_size, icons, background })?;
        let hfs::Node::Folder { children, .. } = &mut v.root else { unreachable!() };
        children.insert(".DS_Store".into(), hfs::Node::File { data: store, mode: 0o644, finder_flags: 0, type_creator: None });
    }
    let (volume, _) = hfs::write(&v)?;
    let image = udif::write(&volume, format)?;
    std::fs::write(&out, image).map_err(|e| format!("{}: {e}", out.display()))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("dmg: {e}");
        std::process::exit(1);
    }
}
