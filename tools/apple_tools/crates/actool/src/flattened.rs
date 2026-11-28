//! Xcode's flattened images for an Icon Composer document, rendered by
//! Xcode on a Mac (tools/icon_flattened.py) into the document's own
//! `Flattened/` folder. This actool doesn't draw a document's flattened
//! images as Xcode does (docs/clean-room/actool.md); with these beside the
//! document, unchanged since they were made, its output is Xcode's.
//!
//! `Flattened/index.json` names the document's digest (`document_key`) and
//! each image: a lossless 8-bit RGBA PNG of exactly the bytes it names,
//! straight RGBA (`rgba`) or premultiplied BGRA as a catalog stores it
//! (`bgra-premultiplied`).

use crate::image::{self, Rgba};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

pub const FOLDER: &str = "Flattened";
const FORMAT: &str = "rules_applecross flattened Icon Composer images 1";

pub struct Flattened {
    images: BTreeMap<String, Rgba>,
}

/// SHA-256 over the document's files but `Flattened/`, in byte order of
/// their paths: each as its path relative to the document (UTF-8, `/`), a
/// NUL, its length (8 bytes little-endian) and its bytes. The same as
/// tools/icon_flattened.py's.
pub fn document_key(doc: &Path) -> Result<String, String> {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, std::path::PathBuf)>) -> Result<(), String> {
        for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let e = e.map_err(|e| e.to_string())?;
            let name = e.file_name().to_string_lossy().into_owned();
            let path = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if rel.is_empty() && name == FOLDER {
                continue;
            }
            if e.file_type().map_err(|e| e.to_string())?.is_dir() {
                walk(&e.path(), &path, out)?;
            } else {
                out.push((path, e.path()));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(doc, "", &mut files)?;
    files.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut h = Sha256::new();
    for (rel, path) in files {
        let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        h.update(rel.as_bytes());
        h.update([0u8]);
        h.update((data.len() as u64).to_le_bytes());
        h.update(&data);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Straight RGBA whose premultiplication (as `Rgba::premultiplied_bgra`
/// rounds) gives exactly `bgra`: every value at or under its alpha has one.
fn unpremultiply_exact(width: usize, height: usize, bgra: &[u8]) -> Result<Rgba, String> {
    let mut data = Vec::with_capacity(bgra.len());
    for p in bgra.chunks_exact(4) {
        let a = p[3] as u32;
        let straight = |c: u8| -> Result<u8, String> {
            if a == 0 {
                return if c == 0 { Ok(0) } else { Err("a transparent pixel with colour".into()) };
            }
            // The least value premultiplying to c.
            let u = ((c as u32 * 255).saturating_sub(127)).div_ceil(a).min(255);
            if (u * a + 127) / 255 == c as u32 { Ok(u as u8) } else { Err("a colour over its alpha".into()) }
        };
        data.extend_from_slice(&[straight(p[2])?, straight(p[1])?, straight(p[0])?, p[3]]);
    }
    let img = Rgba { width, height, data };
    if img.premultiplied_bgra() != bgra {
        return Err("premultiplied pixels this actool can't reproduce".into());
    }
    Ok(img)
}

/// The document's `Flattened/`, if it has one: its images when it still
/// describes the document, else an error asking for them anew.
pub fn read(doc: &Path) -> Result<Option<Flattened>, String> {
    let dir = doc.join(FOLDER);
    let index_path = dir.join("index.json");
    if !index_path.exists() {
        return Ok(None);
    }
    let stale = |why: &str| format!("{}: {why}; run tools/icon_flattened.py on the document again (on a Mac with Xcode)", dir.display());
    let index: serde_json::Value = serde_json::from_slice(&std::fs::read(&index_path).map_err(|e| format!("{}: {e}", index_path.display()))?).map_err(|e| format!("{}: {e}", index_path.display()))?;
    if index.get("format").and_then(|v| v.as_str()) != Some(FORMAT) {
        return Err(stale("an index of another format"));
    }
    if index.get("document").and_then(|v| v.as_str()) != Some(document_key(doc)?.as_str()) {
        return Err(stale("images rendered from another version of the document"));
    }
    let mut images = BTreeMap::new();
    for (label, e) in index.get("images").and_then(|v| v.as_object()).ok_or_else(|| stale("an index without images"))? {
        let file = e.get("file").and_then(|v| v.as_str()).ok_or_else(|| stale("an image without a file"))?;
        if file.contains('/') || file.starts_with('.') {
            return Err(stale("an image outside the folder"));
        }
        let png = image::decode_png(&std::fs::read(dir.join(file)).map_err(|e| format!("{}: {e}", dir.join(file).display()))?)?;
        let dims = (e.get("width").and_then(|v| v.as_u64()), e.get("height").and_then(|v| v.as_u64()));
        if dims != (Some(png.width as u64), Some(png.height as u64)) {
            return Err(stale(&format!("{label}: a size other than its index's")));
        }
        let img = match e.get("bytes").and_then(|v| v.as_str()) {
            Some("rgba") => png,
            Some("bgra-premultiplied") => unpremultiply_exact(png.width, png.height, &png.data).map_err(|why| stale(&format!("{label}: {why}")))?,
            _ => return Err(stale(&format!("{label}: bytes of another kind"))),
        };
        images.insert(label.clone(), img);
    }
    Ok(Some(Flattened { images }))
}

impl Flattened {
    pub fn get(&self, label: &str) -> Result<&Rgba, String> {
        self.images.get(label).ok_or_else(|| format!("{FOLDER}/ has no {label} image; run tools/icon_flattened.py on the document again"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_premultiplied_value_unpremultiplies_exactly() {
        let mut bgra = Vec::new();
        for a in 0..=255u8 {
            for c in 0..=a {
                bgra.extend_from_slice(&[c, c, c, a]);
            }
        }
        let n = bgra.len() / 4;
        assert!(unpremultiply_exact(n, 1, &bgra).is_ok());
    }

    #[test]
    fn the_key_ignores_the_folder_and_follows_the_contents() {
        let dir = std::env::temp_dir().join(format!("flattened-key-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Assets")).unwrap();
        std::fs::write(dir.join("icon.json"), b"{}").unwrap();
        std::fs::write(dir.join("Assets/a.svg"), b"<svg/>").unwrap();
        let k = document_key(&dir).unwrap();
        std::fs::create_dir_all(dir.join(FOLDER)).unwrap();
        std::fs::write(dir.join(FOLDER).join("index.json"), b"{}").unwrap();
        assert_eq!(document_key(&dir).unwrap(), k);
        std::fs::write(dir.join("Assets/a.svg"), b"<svg />").unwrap();
        assert_ne!(document_key(&dir).unwrap(), k);
        std::fs::remove_file(dir.join(FOLDER).join("index.json")).unwrap();
        std::fs::remove_dir(dir.join(FOLDER)).unwrap();
        std::fs::remove_file(dir.join("Assets/a.svg")).unwrap();
        std::fs::remove_dir(dir.join("Assets")).unwrap();
        std::fs::remove_file(dir.join("icon.json")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
