//! Colour sets: each colour a rendition of its appearance
//! (docs/clean-room/actool.md).

use crate::car::{self, Facet, Rendition};
use crate::read_json;

/// The appearances colours are keyed by, and their names in the catalog's
/// table, by value.
pub const APPEARANCES: [&str; 6] =
    ["UIAppearanceAny", "UIAppearanceDark", "UIAppearanceHighContrastAny", "UIAppearanceHighContrastDark", "UIAppearanceLight", "UIAppearanceHighContrastLight"];

/// Colour spaces by name, their code in a colour rendition and their
/// components.
const SPACES: [(&str, u32, &[&str]); 6] = [
    ("srgb", 1, &["red", "green", "blue", "alpha"]),
    ("gray-gamma-22", 2, &["white", "alpha"]),
    ("display-p3", 3, &["red", "green", "blue", "alpha"]),
    ("extended-srgb", 4, &["red", "green", "blue", "alpha"]),
    ("extended-linear-srgb", 5, &["red", "green", "blue", "alpha"]),
    ("extended-gray", 6, &["white", "alpha"]),
];

/// A component as Xcode reads it: with a decimal point, a float (single
/// precision, "0.333" is 0.3330000042915344); `0x` and hex digits over
/// 16^n - 1 ("0x1FF" is 511/4095); otherwise the leading integer over 255
/// ("1" is 1/255, "1e-1" too). Leading spaces are skipped; nothing is
/// clamped.
fn component(text: &str) -> Option<f64> {
    let t = text.trim_start();
    if t.contains('.') {
        return t.parse::<f32>().ok().map(f64::from);
    }
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        let digits: String = hex.chars().take_while(char::is_ascii_hexdigit).collect();
        let n = u64::from_str_radix(&digits, 16).ok()?;
        return Some(n as f64 / (16f64.powi(digits.len() as i32) - 1.0));
    }
    let end = t.char_indices().take_while(|&(i, c)| c.is_ascii_digit() || (i == 0 && c == '-')).count();
    t[..end].parse::<i64>().ok().map(|n| n as f64 / 255.0)
}

/// The value of an entry's appearances: luminosity (light or dark) and
/// contrast (high).
fn appearance(entry: &serde_json::Value) -> Option<u16> {
    let mut luminosity = None;
    let mut high = false;
    for a in entry["appearances"].as_array().map(Vec::as_slice).unwrap_or_default() {
        match (a["appearance"].as_str(), a["value"].as_str()) {
            (Some("luminosity"), Some(v @ ("light" | "dark"))) if luminosity.is_none() => luminosity = Some(v),
            (Some("contrast"), Some("high")) if !high => high = true,
            _ => return None,
        }
    }
    Some(match (luminosity, high) {
        (None, false) => 0,
        (Some("dark"), false) => 1,
        (None, true) => 2,
        (Some("dark"), true) => 3,
        (Some(_), false) => 4,
        (Some(_), true) => 5,
    })
}

/// Compiles colour sets, by name, for iOS: universal colours in the colour
/// spaces above, for any appearance. Colour sets with no colours compile to
/// nothing (the scan leaves them out).
pub fn compile(sets: &[(String, std::path::PathBuf)], identifier: impl Fn(&str) -> u16) -> Result<(Vec<Facet>, Vec<Rendition>), String> {
    let mut facets = Vec::new();
    let mut renditions = Vec::new();
    for (name, dir) in sets {
        let unsupported = |what: String| Err(format!("{}: {what} is not supported by this actool", dir.display()));
        let contents = read_json(&dir.join("Contents.json"))?;
        for key in contents.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if !["colors", "info"].contains(&key.as_str()) {
                return unsupported(format!("the {key} key of a colour set"));
            }
        }
        let id = identifier(name);
        let mut seen = Vec::new();
        for entry in contents["colors"].as_array().map(Vec::as_slice).unwrap_or_default() {
            let Some(color) = entry["color"].as_object() else { continue };
            for k in entry.as_object().unwrap().keys() {
                if !["idiom", "appearances", "color"].contains(&k.as_str()) {
                    return unsupported(format!("the {k} key of a colour"));
                }
            }
            if entry["idiom"].as_str() != Some("universal") {
                return unsupported("a colour for an idiom other than universal".into());
            }
            let Some(appearance) = appearance(entry) else {
                return unsupported(format!("the appearances {}", entry["appearances"]));
            };
            if seen.contains(&appearance) {
                return unsupported("two colours for one appearance".into());
            }
            seen.push(appearance);
            for k in color.keys() {
                if !["color-space", "components"].contains(&k.as_str()) {
                    return unsupported(format!("a colour's {k} key (system colours among them)"));
                }
            }
            // Display P3 when none is named.
            let space = color.get("color-space").and_then(|v| v.as_str()).unwrap_or("display-p3");
            let Some(&(_, code, names)) = SPACES.iter().find(|s| s.0 == space) else {
                return unsupported(format!("the colour space {space:?}"));
            };
            let given = color.get("components").and_then(|v| v.as_object()).cloned().unwrap_or_default();
            if given.len() != names.len() {
                return unsupported(format!("a {space} colour's components {}", serde_json::Value::Object(given)));
            }
            let mut components = Vec::new();
            for n in names {
                match given.get(*n).and_then(|v| v.as_str()).and_then(component) {
                    Some(v) => components.push(v),
                    None => return unsupported(format!("the {n} component {:?}", given.get(*n))),
                }
            }
            let mut attrs = vec![(car::ATTR_SCALE, 1), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_COLOR)];
            if appearance != 0 {
                attrs.push((car::ATTR_APPEARANCE, appearance));
            }
            renditions.push(Rendition { attrs, csi: car::color_csi(name, code, &components) });
        }
        if !seen.is_empty() {
            facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_COLOR), (car::ATTR_IDENTIFIER, id)] });
        }
    }
    Ok((facets, renditions))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Components as Xcode 27.0 stored them.
    #[test]
    fn components() {
        let cases: [(&str, f64); 10] = [
            ("0.123456789012345678", 0.12345679104328156),
            ("0.333", 0.3330000042915344),
            (".5", 0.5),
            (" 0.5", 0.5),
            ("1e-1", 1.0 / 255.0),
            ("0x1FF", 511.0 / 4095.0),
            ("0xff", 1.0),
            ("256", 256.0 / 255.0),
            ("-0.25", -0.25),
            ("1.5", 1.5),
        ];
        for (text, value) in cases {
            assert_eq!(component(text), Some(value), "{text}");
        }
    }
}
