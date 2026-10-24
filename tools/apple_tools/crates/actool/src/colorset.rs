//! Colour sets (docs/clean-room/actool.md): a universal colour per set, for
//! any appearance, stored as a `COLR` rendition of its colour space and
//! components.

use crate::car::{self, Facet, Rendition};
use crate::imageset::Compiled;
use crate::read_json;
use std::path::PathBuf;

pub const PART_COLOR: u16 = 217;

/// A component as Xcode reads it: with a decimal point, a 32-bit float;
/// otherwise an integer (decimal or `0x` hex) out of 255.
fn component(s: &str) -> Result<f64, String> {
    if s.contains('.') {
        return s
            .parse::<f32>()
            .map(f64::from)
            .map_err(|_| format!("not a colour component: {s}"));
    }
    let n = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => s.parse::<u32>(),
    }
    .map_err(|_| format!("not a colour component: {s}"))?;
    Ok(n as f64 / 255.0)
}

pub fn compile(
    sets: &[(String, PathBuf)],
    identifier: impl Fn(&str) -> u16,
) -> Result<Compiled, String> {
    let mut out = Compiled::default();
    for (name, dir) in sets {
        let contents = read_json(&dir.join("Contents.json"))?;
        for key in contents
            .as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            if !["colors", "info"].contains(&key.as_str()) {
                return Err(format!(
                    "{}: the {key} key of a colour set is not supported by this actool",
                    dir.display()
                ));
            }
        }
        let entries: Vec<&serde_json::Value> = contents["colors"]
            .as_array()
            .ok_or_else(|| format!("{}: a colour set without a colors list", dir.display()))?
            .iter()
            .filter(|e| !e["color"].is_null())
            .collect();
        let id = identifier(name);
        out.facets.push(Facet {
            name: name.clone(),
            attrs: vec![
                (car::ATTR_ELEMENT, car::ELEMENT_ICON),
                (car::ATTR_PART, PART_COLOR),
                (car::ATTR_IDENTIFIER, id),
            ],
        });
        // A dark colour is appearance 1 and adds UIAppearanceDark to the
        // appearance keys, but which of those two Xcode allocates first
        // varies (7 of 12 small catalogs as Any first): not established.
        if entries.len() != 1 {
            return Err(format!("{}: colour sets with colours for other appearances or idioms are not supported by this actool", dir.display()));
        }
        let entry = entries[0];
        for k in entry.as_object().unwrap().keys() {
            if !["color", "idiom"].contains(&k.as_str()) {
                return Err(format!(
                    "{}: the {k} key of a colour is not supported by this actool",
                    dir.display()
                ));
            }
        }
        if entry["idiom"].as_str() != Some("universal") {
            return Err(format!(
                "{}: colours for idioms other than universal are not supported by this actool",
                dir.display()
            ));
        }
        let (space, values) = color_value(dir, &entry["color"])?;
        out.renditions.push(Rendition {
            attrs: vec![
                (car::ATTR_SCALE, 1),
                (car::ATTR_IDENTIFIER, id),
                (car::ATTR_ELEMENT, car::ELEMENT_ICON),
                (car::ATTR_PART, PART_COLOR),
            ],
            csi: car::color_csi(name, space, &values),
        });
    }
    Ok(out)
}

/// A colour's space code and components.
fn color_value(
    dir: &std::path::Path,
    color: &serde_json::Value,
) -> Result<(u32, Vec<f64>), String> {
    for k in color
        .as_object()
        .ok_or("a colour that isn't an object")?
        .keys()
    {
        if !["color-space", "components"].contains(&k.as_str()) {
            return Err(format!(
                "{}: the {k} key of a colour is not supported by this actool",
                dir.display()
            ));
        }
    }
    // Colour space codes; Display P3 when none is named.
    let (space, names): (u32, &[&str]) = match color["color-space"].as_str() {
        Some("srgb") => (1, &["red", "green", "blue", "alpha"]),
        Some("gray-gamma-22") => (2, &["white", "alpha"]),
        Some("display-p3") | None => (3, &["red", "green", "blue", "alpha"]),
        Some("extended-srgb") => (4, &["red", "green", "blue", "alpha"]),
        Some("extended-linear-srgb") => (5, &["red", "green", "blue", "alpha"]),
        Some("extended-gray") => (6, &["white", "alpha"]),
        Some(other) => {
            return Err(format!(
                "{}: colour space {other} is not supported by this actool",
                dir.display()
            ))
        }
    };
    let components = color["components"]
        .as_object()
        .ok_or_else(|| format!("{}: a colour without components", dir.display()))?;
    if components.len() != names.len() || names.iter().any(|n| !components.contains_key(*n)) {
        return Err(format!(
            "{}: these colour components are not supported by this actool",
            dir.display()
        ));
    }
    let values: Vec<f64> = names
        .iter()
        .map(|n| {
            components[*n]
                .as_str()
                .ok_or_else(|| format!("{}: a colour component that isn't a string", dir.display()))
                .and_then(component)
        })
        .collect::<Result<_, _>>()?;
    Ok((space, values))
}
