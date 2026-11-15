//! `dmg`: a Mac disk image from a folder, on any platform (no hdiutil):
//! an HFS+ volume (hfs) in a UDIF image (udif).
//!
//!     dmg -o Out.dmg --volname Name --srcfolder DIR [--format UDZO|ULFO|UDRO]
//!         [--time UNIX_SECONDS] [--hide NAME]... [--open-folder PATH] [--free BYTES]
//!         [--uid N] [--gid N]
//!
//! The image is the same for the same inputs (`--time` defaults to
//! SOURCE_DATE_EPOCH, else 0).

mod hfs;
mod udif;

use std::path::PathBuf;

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
            _ => return Err(format!("{a}: unknown argument")),
        }
    }
    let out = out.ok_or("-o is required")?;
    let name = name.ok_or("--volname is required")?;
    let src = src.ok_or("--srcfolder is required")?;
    let mut root = hfs::read_dir(&src)?;
    for h in &hide {
        let hfs::Node::Folder { children, .. } = &mut root else { unreachable!() };
        match children.get_mut(h.trim_matches('/')) {
            Some(hfs::Node::Folder { finder_flags, .. }) | Some(hfs::Node::File { finder_flags, .. }) => *finder_flags |= hfs::INVISIBLE,
            _ => return Err(format!("--hide {h}: no such file or folder at the top of the volume")),
        }
    }
    let volume = hfs::write(&hfs::Volume { name, root, time, uid, gid, open_folder: open, free })?;
    let image = udif::write(&volume, format)?;
    std::fs::write(&out, image).map_err(|e| format!("{}: {e}", out.display()))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("dmg: {e}");
        std::process::exit(1);
    }
}
