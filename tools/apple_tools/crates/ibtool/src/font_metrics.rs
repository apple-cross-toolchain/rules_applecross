//! The Mac's system fonts' text metrics, as AppKit measures a string
//! (`NSString.size(withAttributes:)`), for laying out content-sized
//! controls as Interface Builder does (appkit.rs). Measured on a Mac by
//! tools/font_metrics_probe (font_metrics.bin): a string's width is its characters'
//! widths and each adjacent pair's adjustment (its kerning) summed, which
//! holds for every character the tables have (measured); its height the
//! font's line height.

use std::collections::HashMap;
use std::sync::OnceLock;

struct Table {
    line_height: f64,
    index: HashMap<char, u16>,
    widths: Vec<f64>,
    pairs: HashMap<(u16, u16), f64>,
}

fn tables() -> &'static HashMap<(String, u64), Table> {
    static TABLES: OnceLock<HashMap<(String, u64), Table>> = OnceLock::new();
    TABLES.get_or_init(|| parse(include_bytes!("font_metrics.bin")).expect("font_metrics.bin"))
}

fn parse(b: &[u8]) -> Option<HashMap<(String, u64), Table>> {
    let mut at = 0;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = b.get(at..at + n)?;
        at += n;
        Some(s)
    };
    if take(4)? != b"IBFM" {
        return None;
    }
    let u32_ = |s: &[u8]| u32::from_le_bytes(s.try_into().expect("four bytes"));
    let f64_ = |s: &[u8]| f64::from_le_bytes(s.try_into().expect("eight bytes"));
    let _version = u32_(take(4)?);
    let count = u32_(take(4)?);
    let mut out = HashMap::new();
    for _ in 0..count {
        let n = take(1)?[0] as usize;
        let name = String::from_utf8(take(n)?.to_vec()).ok()?;
        let size = f64_(take(8)?);
        let line_height = f64_(take(8)?);
        let chars = u32_(take(4)?) as usize;
        let mut index = HashMap::new();
        for i in 0..chars {
            index.insert(char::from_u32(u32_(take(4)?))?, i as u16);
        }
        let widths = (0..chars).map(|_| take(8).map(f64_)).collect::<Option<Vec<f64>>>()?;
        let pairs_count = u32_(take(4)?) as usize;
        let mut pairs = HashMap::new();
        for _ in 0..pairs_count {
            let s = take(12)?;
            let (i, j) = (u16::from_le_bytes([s[0], s[1]]), u16::from_le_bytes([s[2], s[3]]));
            pairs.insert((i, j), f64_(&s[4..12]));
        }
        // The first table for a font and size (the menu font is the system
        // font's).
        out.entry((name, size.to_bits())).or_insert(Table { line_height, index, widths, pairs });
    }
    Some(out)
}

/// A string's width and line height in the font (its PostScript name) at
/// a size, `None` when the tables lack the font, the size or a character.
pub fn measure(font: &str, size: f64, text: &str) -> Option<(f64, f64)> {
    let t = tables().get(&(font.to_string(), size.to_bits()))?;
    let glyphs: Vec<u16> = text.chars().map(|c| t.index.get(&c).copied()).collect::<Option<_>>()?;
    let mut w: f64 = glyphs.iter().map(|g| t.widths[*g as usize]).sum();
    for pair in glyphs.windows(2) {
        w += t.pairs.get(&(pair[0], pair[1])).copied().unwrap_or(0.0);
    }
    Some((w, t.line_height))
}

#[cfg(test)]
mod tests {
    #[test]
    fn measures_as_appkit() {
        // AppKit's widths on a Mac (NSString.size(withAttributes:)).
        assert_eq!(super::measure(".SFNS-Regular", 13.0, "Hello"), Some((30.951171875, 16.0)));
        assert_eq!(super::measure(".SFNS-Regular", 13.0, "A longer title"), Some((77.23193359375, 16.0)));
        assert_eq!(super::measure(".SFNS-Bold", 13.0, "Hello"), Some((33.334716796875, 16.0)));
        assert_eq!(super::measure(".SFNS-Regular", 13.0, "\u{4e00}"), None);
    }
}
