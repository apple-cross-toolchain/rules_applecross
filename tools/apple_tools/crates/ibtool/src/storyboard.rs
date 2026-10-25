//! The supported subset of iOS and tvOS storyboards and the nibs it compiles
//! to; see docs/clean-room/ibtool.md.

use crate::fonts;
use crate::nib::{self, reference, Object, Value};
use roxmltree::{Document, Node};
use std::collections::HashMap;

pub struct Compiled {
    /// (file name, contents) inside the .storyboardc directory.
    pub files: Vec<(String, Vec<u8>)>,
}

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

fn attr<'a>(n: Node<'a, '_>, name: &str) -> Result<&'a str, String> {
    n.attribute(name).ok_or_else(|| format!("<{}> has no {name} attribute", n.tag_name().name()))
}

fn children<'a, 'i>(n: Node<'a, 'i>, tag: &'a str) -> impl Iterator<Item = Node<'a, 'i>> + 'a {
    n.children().filter(move |c| c.is_element() && c.tag_name().name() == tag)
}

fn child<'a, 'i>(n: Node<'a, 'i>, tag: &'a str, key: &str) -> Option<Node<'a, 'i>> {
    children(n, tag).find(|c| c.attribute("key") == Some(key))
}

/// Interface Builder's own settings for how a controller is drawn while
/// editing, which compiled storyboards don't record (five probes); a
/// freeform size is another matter (see `compile`).
const EDITOR_ONLY: [&str; 8] = [
    "simulatedMetricsContainer", "simulatedNavigationBarMetrics", "simulatedStatusBarMetrics", "simulatedTabBarMetrics",
    "simulatedToolbarMetrics", "simulatedOrientationMetrics", "simulatedScreenMetrics", "freeformSimulatedSizeMetrics",
];

fn only_children(n: Node, allowed: &[&str]) -> Result<(), String> {
    for c in n.children().filter(|c| c.is_element()) {
        let editor_only = (n.has_tag_name("viewController") || n.has_tag_name("navigationController"))
            && (EDITOR_ONLY.contains(&c.tag_name().name()) || c.has_tag_name("nil") && c.attribute("key").is_some_and(|k| k.starts_with("simulated")));
        if !allowed.contains(&c.tag_name().name()) && !editor_only {
            return fail(format!("<{}> inside <{}> is not supported by this ibtool", c.tag_name().name(), n.tag_name().name()));
        }
    }
    Ok(())
}

/// Refuses attributes other than `allowed`, and values other than those
/// given (a `None` value accepts any).
fn only_attributes(n: Node, allowed: &[(&str, Option<&str>)]) -> Result<(), String> {
    // An object's name in Interface Builder's outline; not compiled.
    for a in n.attributes().filter(|a| a.name() != "userLabel") {
        match allowed.iter().find(|(name, _)| *name == a.name()) {
            Some((_, Some(v))) if *v != a.value() => {
                return fail(format!("{}=\"{}\" on <{}> is not supported by this ibtool", a.name(), a.value(), n.tag_name().name()))
            }
            Some(_) => {}
            None => return fail(format!("the {} attribute of <{}> is not supported by this ibtool", a.name(), n.tag_name().name())),
        }
    }
    Ok(())
}

fn number(s: &str) -> Result<f64, String> {
    s.parse::<f64>().map_err(|_| format!("not a number: {s}"))
}

/// A CGRect, CGPoint or UIEdgeInsets archived as NIB data: a 7 (doubles)
/// followed by the components in little-endian order.
fn doubles(v: &[f64]) -> Value {
    let mut d = vec![7u8];
    for x in v {
        d.extend_from_slice(&x.to_le_bytes());
    }
    Value::Data(d)
}

/// The shortest decimal that round-trips, as NSColor archives components.
/// `x` rounded to 10 significant digits, as `%.10g` would print it.
fn significant_10(x: f64) -> f64 {
    format!("{x:.9e}").parse().unwrap()
}

/// `x` as C's `%.3g` prints it: 3 significant digits, trailing zeros
/// dropped, in exponent form (two exponent digits at least) below 1e-4.
fn general_3(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let sci = format!("{x:.2e}");
    let (mantissa, exponent) = sci.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let trim = |s: String| if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if exponent < -4 || exponent >= 3 {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim(mantissa.to_string()), exponent.abs())
    } else {
        trim(format!("{x:.*}", (2 - exponent) as usize))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Platform {
    Ios,
    Tvos,
}

/// A system colour's components as Xcode archives them, whatever the
/// document's copy says: a white and alpha, or sRGB and alpha.
enum SystemColor {
    Gray([f64; 2]),
    Rgb([f64; 4]),
}

/// iOS's system colours, as Xcode 27.0's ibtool archives them (measured).
const SYSTEM_COLORS: &[(&str, SystemColor)] = &[
    ("darkTextColor", SystemColor::Gray([0.0, 1.0])),
    ("groupTableViewBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("labelColor", SystemColor::Gray([0.0, 1.0])),
    ("lightTextColor", SystemColor::Gray([1.0, 0.6])),
    ("linkColor", SystemColor::Rgb([0.0, 0.4784313725, 1.0, 1.0])),
    ("opaqueSeparatorColor", SystemColor::Rgb([0.7764705882, 0.7764705882, 0.7843137255, 1.0])),
    ("placeholderTextColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.2980392157])),
    ("quaternaryLabelColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.1764705882])),
    ("quaternarySystemFillColor", SystemColor::Rgb([0.4549019608, 0.4549019608, 0.5019607843, 0.08])),
    ("secondaryLabelColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.6])),
    ("secondarySystemBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("secondarySystemFillColor", SystemColor::Rgb([0.4705882353, 0.4705882353, 0.5019607843, 0.16])),
    ("secondarySystemGroupedBackgroundColor", SystemColor::Gray([1.0, 1.0])),
    ("separatorColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.12])),
    ("systemBackgroundColor", SystemColor::Gray([1.0, 1.0])),
    ("systemBlueColor", SystemColor::Rgb([0.0, 0.5333333333, 1.0, 1.0])),
    ("systemBrownColor", SystemColor::Rgb([0.6745098039, 0.4980392157, 0.368627451, 1.0])),
    ("systemCyanColor", SystemColor::Rgb([0.0, 0.7529411765, 0.9098039216, 1.0])),
    ("systemFillColor", SystemColor::Rgb([0.4705882353, 0.4705882353, 0.5019607843, 0.2])),
    ("systemGray2Color", SystemColor::Rgb([0.6823529412, 0.6823529412, 0.6980392157, 1.0])),
    ("systemGray3Color", SystemColor::Rgb([0.7803921569, 0.7803921569, 0.8, 1.0])),
    ("systemGray4Color", SystemColor::Rgb([0.8196078431, 0.8196078431, 0.8392156863, 1.0])),
    ("systemGray5Color", SystemColor::Rgb([0.8980392157, 0.8980392157, 0.9176470588, 1.0])),
    ("systemGray6Color", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("systemGrayColor", SystemColor::Rgb([0.5568627451, 0.5568627451, 0.5764705882, 1.0])),
    ("systemGreenColor", SystemColor::Rgb([0.2039215686, 0.7803921569, 0.3490196078, 1.0])),
    ("systemGroupedBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("systemIndigoColor", SystemColor::Rgb([0.3803921569, 0.3333333333, 0.9607843137, 1.0])),
    ("systemMintColor", SystemColor::Rgb([0.0, 0.7843137255, 0.7019607843, 1.0])),
    ("systemOrangeColor", SystemColor::Rgb([1.0, 0.5529411765, 0.1568627451, 1.0])),
    ("systemPinkColor", SystemColor::Rgb([1.0, 0.1764705882, 0.3333333333, 1.0])),
    ("systemPurpleColor", SystemColor::Rgb([0.7960784314, 0.1882352941, 0.8784313725, 1.0])),
    ("systemRedColor", SystemColor::Rgb([1.0, 0.2196078431, 0.2352941176, 1.0])),
    ("systemTealColor", SystemColor::Rgb([0.0, 0.7647058824, 0.8156862745, 1.0])),
    ("systemYellowColor", SystemColor::Rgb([1.0, 0.8, 0.0, 1.0])),
    ("tertiaryLabelColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.2980392157])),
    ("tertiarySystemBackgroundColor", SystemColor::Gray([1.0, 1.0])),
    ("tertiarySystemFillColor", SystemColor::Rgb([0.462745098, 0.462745098, 0.5019607843, 0.12])),
    ("tertiarySystemGroupedBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("tintColor", SystemColor::Rgb([0.0, 0.5333333333, 1.0, 1.0])),
];

/// Colour components as Interface Builder archives them: each rounded to 10
/// significant digits, as a float followed by the double when the float
/// isn't exact, then the components as text (`%.3g`, alpha only when not 1).
fn color_components(names: &[(&'static str, &'static str)], values: &[f64]) -> (Vec<(&'static str, Value)>, Vec<u8>) {
    let values: Vec<f64> = values.iter().map(|&x| significant_10(x)).collect();
    let mut keys = Vec::new();
    for (&(key, double_key), &x) in names.iter().zip(&values) {
        keys.push((key, Value::Float(x as f32)));
        if (x as f32) as f64 != x {
            keys.push((double_key, Value::Double(x)));
        }
    }
    let (alpha, rest) = values.split_last().unwrap();
    let mut text: Vec<String> = rest.iter().map(|&x| general_3(x)).collect();
    if *alpha != 1.0 {
        text.push(general_3(*alpha));
    }
    (keys, text.join(" ").into_bytes())
}

const RGB_KEYS: [(&str, &str); 4] = [("UIRed", "UIRed-Double"), ("UIGreen", "UIGreen-Double"), ("UIBlue", "UIBlue-Double"), ("UIAlpha", "UIAlpha-Double")];
const GRAY_KEYS: [(&str, &str); 2] = [("UIWhite", "UIWhite-Double"), ("UIAlpha", "UIAlpha-Double")];

fn autoresizing_mask(n: Node) -> Result<i64, String> {
    let mut m = 0;
    for (name, bit) in [("flexibleMinX", 1), ("widthSizable", 2), ("flexibleMaxX", 4), ("flexibleMinY", 8), ("heightSizable", 16), ("flexibleMaxY", 32)] {
        if n.attribute(name) == Some("YES") {
            m |= bit;
        }
    }
    Ok(m)
}

/// NSLayoutAttribute.
fn layout_attribute(name: &str) -> Result<i64, String> {
    Ok(match name {
        "left" => 1,
        "right" => 2,
        "top" => 3,
        "bottom" => 4,
        "leading" => 5,
        "trailing" => 6,
        "width" => 7,
        "height" => 8,
        "centerX" => 9,
        "centerY" => 10,
        other => return fail(format!("constraint attribute {other} is not supported by this ibtool")),
    })
}

/// A nib being built: object names, and the strings and numbers it shares.
/// Xcode's nibs hold one object per distinct string or number, encoded
/// where it is first used.
struct Nib {
    platform: Platform,
    /// Whether the storyboard is in `Base.lproj`, where Xcode makes the
    /// strings a translation replaces localizable.
    localizable: bool,
    /// The document's `colorMatched`: without it, colours' components are
    /// taken as they are rather than converted from their colour space.
    color_matched: bool,
    shared: HashMap<String, String>,
    next: usize,
}

impl Nib {
    fn new(platform: Platform, color_matched: bool, localizable: bool) -> Nib {
        Nib { platform, color_matched, localizable, shared: HashMap::new(), next: 0 }
    }

    fn fresh(&mut self) -> String {
        self.next += 1;
        format!("#{}", self.next)
    }

    fn shared(&mut self, key: String, make: impl FnOnce() -> Object) -> Value {
        if let Some(name) = self.shared.get(&key) {
            return reference(name.clone());
        }
        let name = self.fresh();
        self.shared.insert(key, name.clone());
        Value::New(make().named(name))
    }

    /// The object shared under `key`, if there is one yet.
    fn existing(&self, key: &str) -> Option<Value> {
        self.shared.get(key).map(|name| reference(name.clone()))
    }

    fn string(&mut self, s: &str) -> Value {
        self.shared(format!("s:{s}"), || Object::new("NSString", vec![("NS.bytes", Value::Data(s.as_bytes().to_vec()))]))
    }

    fn double_number(&mut self, d: f64) -> Value {
        self.shared(format!("d:{d}"), || Object::new("NSNumber", vec![("NS.dblval", Value::Double(d))]))
    }

    /// A named image, as the nib refers to it: one placeholder per name.
    fn image(&mut self, name: &str) -> Value {
        if let Some(v) = self.existing(&format!("image:{name}")) {
            return v;
        }
        let resource = self.string(name);
        self.shared(format!("image:{name}"), || Object::new("UIImageNibPlaceholder", vec![
            ("UIImageWidth", Value::Float(1.0)),
            ("UIImageHeight", Value::Float(1.0)),
            ("UIResourceName", resource),
            ("UISystemSymbolResourceName", Value::Nil),
            ("UISymbolImageConfiguration", Value::Nil),
            ("UIResourceCatalogName", Value::Nil),
        ]))
    }

    /// A string a translation replaces, the `property` of the object `id`:
    /// in `Base.lproj` an `NSLocalizableString` keyed `<id>.<property>`
    /// with the string as its development value. An empty string is never
    /// localized.
    fn text(&mut self, id: &str, property: &str, s: &str) -> Value {
        if !self.localizable || s.is_empty() {
            return self.string(s);
        }
        let key = self.string(&format!("{id}.{property}"));
        let dev = self.string(s);
        Value::New(Object::new("NSLocalizableString", vec![
            ("NS.bytes", Value::Data(s.as_bytes().to_vec())),
            ("NSKey", key),
            ("NSDev", dev),
        ]))
    }

    /// The nib's one empty array, encoded where first used.
    fn empty_array(&mut self) -> Value {
        self.shared("empty array".into(), || Object::new("NSArray", vec![("NSInlinedValue", Value::Bool(true))]))
    }

    fn int_number(&mut self, i: i64) -> Value {
        self.shared(format!("i:{i}"), || Object::new("NSNumber", vec![("NS.intval", Value::Int(i))]))
    }

    fn array(&mut self, class: &'static str, items: Vec<Value>) -> Value {
        let mut values = vec![("NSInlinedValue", Value::Bool(true))];
        values.extend(items.into_iter().map(|v| ("UINibEncoderEmptyKey", v)));
        Value::New(Object::new(class, values))
    }

    fn system_color(&mut self, name: &str, color: &SystemColor) -> Value {
        let name_value = self.string(name);
        let (count, components, text_key, space) = match color {
            SystemColor::Gray(c) => (2, color_components(&GRAY_KEYS, c), "NSWhite", 4),
            SystemColor::Rgb(c) => (4, color_components(&RGB_KEYS, c), "NSRGB", 2),
        };
        self.shared(format!("color:{name}"), || {
            let mut keys = vec![("UISystemColorName", name_value), ("UIColorComponentCount", Value::Int(count))];
            keys.extend(components.0);
            keys.push((text_key, Value::Data(components.1)));
            keys.push(("NSColorSpace", Value::Int(space)));
            Object::new("UIColor", keys).with_fallback("NSColor")
        })
    }

    /// UIViewLargeContentStoredProperties, which tvOS's views don't have.
    fn large_content(&self, values: &mut Vec<(&'static str, Value)>) {
        if self.platform == Platform::Ios {
            values.push(("UIViewLargeContentStoredProperties", Value::Nil));
        }
    }
}

fn color(nib: &mut Nib, n: Node) -> Result<Value, String> {
    // A system colour is Xcode's own whatever the document says: its
    // definition in the resources, or Xcode 11's fallback name and
    // components beside it.
    if let Some(name) = n.attribute("systemColor") {
        only_attributes(n, &[
            ("key", None), ("systemColor", None), ("cocoaTouchSystemColor", None), ("red", None), ("green", None),
            ("blue", None), ("white", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", None),
        ])?;
        if nib.platform != Platform::Ios {
            return fail(format!("system color {name}: system colors outside iOS are not supported by this ibtool"));
        }
        let sc = &SYSTEM_COLORS.iter().find(|(c, _)| *c == name).ok_or_else(|| format!("system color {name} is not supported by this ibtool"))?.1;
        return Ok(nib.system_color(name, sc));
    }
    // Xcode 11's form names the system colour (and an older fallback), the
    // older form only a UIKit class colour; neither is declared in the
    // document. Both are archived as that system colour.
    if let Some(name) = n.attribute("xcode11CocoaTouchSystemColor").or_else(|| n.attribute("cocoaTouchSystemColor")) {
        only_attributes(n, &[("key", None), ("xcode11CocoaTouchSystemColor", None), ("cocoaTouchSystemColor", None)])?;
        if nib.platform != Platform::Ios {
            return fail(format!("system color {name}: system colors outside iOS are not supported by this ibtool"));
        }
        let sc = &SYSTEM_COLORS.iter().find(|(c, _)| *c == name).ok_or_else(|| format!("system color {name} is not supported by this ibtool"))?.1;
        return Ok(nib.system_color(name, sc));
    }
    if !nib.color_matched {
        return unmatched_color(nib, n);
    }
    // Equal colours are one object, compared at 10 significant digits. A
    // grey is archived as given (no conversion), named when it's exactly
    // black, white or clear.
    // Calibrated white is converted to the gamma 2.2 grey: only black and
    // white are established, white coming out a float's step above 1
    // unless it's opaque (and so `whiteColor`).
    let calibrated = n.attribute("colorSpace") == Some("calibratedWhite") || n.attribute("customColorSpace") == Some("calibratedWhite");
    if calibrated {
        only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", Some("calibratedWhite"))])?;
        let white = number(attr(n, "white")?)?;
        let alpha = significant_10(number(attr(n, "alpha")?)?);
        if white != 0.0 && white != 1.0 {
            return fail(format!("calibrated white {white} is converted by Xcode in a way not established for this ibtool"));
        }
        if white == 1.0 && alpha != 1.0 {
            let key = format!("color:calibrated-white:{alpha:?}");
            if let Some(v) = nib.existing(&key) {
                return Ok(v);
            }
            let (alpha_keys, alpha_text) = color_components(&GRAY_KEYS[1..], &[alpha]);
            let mut keys = vec![("UIColorComponentCount", Value::Int(2)), ("UIWhite", Value::Float(1.0000001))];
            keys.extend(alpha_keys);
            let mut text = b"1 ".to_vec();
            text.extend(alpha_text);
            keys.push(("NSWhite", Value::Data(text)));
            keys.push(("NSColorSpace", Value::Int(4)));
            return Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")));
        }
    }
    if calibrated || n.attribute("customColorSpace") == Some("genericGamma22GrayColorSpace") {
        if !calibrated {
            only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", None)])?;
        }
        let c = [number(attr(n, "white")?)?, number(attr(n, "alpha")?)?].map(significant_10);
        let name = match c {
            [0.0, 1.0] => Some("blackColor"),
            [1.0, 1.0] => Some("whiteColor"),
            [0.0, 0.0] => Some("clearColor"),
            _ => None,
        };
        if let Some(name) = name {
            return Ok(nib.system_color(name, &SystemColor::Gray(c)));
        }
        if let Some(v) = nib.existing(&format!("color:gray:{c:?}")) {
            return Ok(v);
        }
        let (components, text) = color_components(&GRAY_KEYS, &c);
        let mut keys = vec![("UIColorComponentCount", Value::Int(2))];
        keys.extend(components);
        keys.push(("NSWhite", Value::Data(text)));
        keys.push(("NSColorSpace", Value::Int(4)));
        return Ok(nib.shared(format!("color:gray:{c:?}"), || Object::new("UIColor", keys).with_fallback("NSColor")));
    }
    only_attributes(n, &[("key", None), ("red", None), ("green", None), ("blue", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", Some("sRGB"))])?;
    let c = [number(attr(n, "red")?)?, number(attr(n, "green")?)?, number(attr(n, "blue")?)?, number(attr(n, "alpha")?)?].map(significant_10);
    let key = format!("color:rgb:{c:?}");
    if let Some(v) = nib.existing(&key) {
        return Ok(v);
    }
    let (components, text) = color_components(&RGB_KEYS, &c);
    let mut keys = vec![("UIColorComponentCount", Value::Int(4))];
    keys.extend(components);
    keys.push(("NSRGB", Value::Data(text)));
    keys.push(("NSColorSpace", Value::Int(2)));
    Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")))
}

/// A colour in a document without `colorMatched`: archived as sRGB with
/// the document's components when they are calibrated RGB or white, which
/// aren't converted; colours in other spaces are, so only their neutral
/// black and white (which convert to themselves) are established.
fn unmatched_color(nib: &mut Nib, n: Node) -> Result<Value, String> {
    let space = match (n.attribute("colorSpace"), n.attribute("customColorSpace")) {
        (Some("custom"), Some(s)) => s,
        (Some(s), None) => s,
        _ => return fail("<color> without a colour space is not supported by this ibtool"),
    };
    let c = match space {
        "calibratedRGB" | "sRGB" => {
            only_attributes(n, &[("key", None), ("red", None), ("green", None), ("blue", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", None)])?;
            [number(attr(n, "red")?)?, number(attr(n, "green")?)?, number(attr(n, "blue")?)?, number(attr(n, "alpha")?)?]
        }
        "calibratedWhite" | "genericGamma22GrayColorSpace" => {
            only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", None)])?;
            // The grey becomes red, green and blue as a float.
            let w = number(attr(n, "white")?)? as f32 as f64;
            [w, w, w, number(attr(n, "alpha")?)?]
        }
        other => return fail(format!("colour space {other} without colorMatched is not supported by this ibtool")),
    };
    let calibrated = space.starts_with("calibrated");
    if !calibrated && !(c[0] == c[1] && c[1] == c[2] && (c[0] == 0.0 || c[0] == 1.0)) {
        return fail(format!("{space} colours other than black and white are converted without colorMatched, which is not supported by this ibtool"));
    }
    let c = c.map(significant_10);
    let key = format!("color:rgb:{c:?}");
    if let Some(v) = nib.existing(&key) {
        return Ok(v);
    }
    let (components, text) = color_components(&RGB_KEYS, &c);
    let mut keys = vec![("UIColorComponentCount", Value::Int(4))];
    keys.extend(components);
    keys.push(("NSRGB", Value::Data(text)));
    keys.push(("NSColorSpace", Value::Int(2)));
    Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")))
}

/// A view controller's navigation item: its title, also as an attributed
/// string without attributes, no items, and the large title mode.
fn navigation_item(nib: &mut Nib, n: Node) -> Result<Value, String> {
    only_attributes(n, &[("key", Some("navigationItem")), ("title", None), ("largeTitleDisplayMode", None), ("id", None)])?;
    if n.has_children() {
        return fail("navigation items with bar button items or other children are not supported by this ibtool");
    }
    let mut values = Vec::new();
    // An empty title is left out.
    if let Some(title) = n.attribute("title").filter(|t| !t.is_empty()) {
        // The attributed title holds the title's own object.
        let (first, string) = if nib.localizable {
            let Value::New(o) = nib.text(attr(n, "id")?, "title", title) else { unreachable!() };
            (Value::New(o.named("navigation title")), reference("navigation title"))
        } else {
            (nib.string(title), nib.string(title))
        };
        values.push(("UITitle", first));
        let attributes = nib.array("NSDictionary", Vec::new());
        values.push(("UIAttributedTitle", Value::New(Object::new("NSAttributedString", vec![("NSString", string), ("NSAttributes", attributes)]))));
    }
    values.push(("UICenterItemGroups", nib.empty_array()));
    let mode = match n.attribute("largeTitleDisplayMode") {
        None | Some("automatic") => 0,
        Some("always") => 1,
        Some("never") => 2,
        Some(other) => return fail(format!("largeTitleDisplayMode {other} is not supported by this ibtool")),
    };
    values.push(("UILargeTitleDisplayMode", Value::Int(mode)));
    Ok(Value::New(Object::new("UINavigationItem", values).named("navigation item")))
}

/// A text style's font as Xcode archives it: name, traits and the size
/// recorded for NSFont, per platform.
fn text_style(platform: Platform, style: &str) -> Result<(&'static str, i64, f64), String> {
    Ok(match (platform, style) {
        (Platform::Ios, "UICTFontTextStyleHeadline") => (".SFUI-Semibold", 2, 14.0),
        (Platform::Ios, "UICTFontTextStyleCallout") => (".SFUI-Regular", 0, 13.0),
        (Platform::Tvos, "UICTFontTextStyleTitle1") => (".SFUI-Medium", 0, 76.0),
        _ => return fail(format!("the {style} text style is not supported by this ibtool here")),
    })
}

/// The class a custom class names at run time: a Swift class in a module is
/// named as the Swift runtime names it (`_TtC`, then the module and class,
/// each after its length in bytes; the standard library is `s`). A module
/// the target provides is ibtool's `--module`, when there is one.
fn runtime_class_name(n: Node, module: Option<&str>, default: &str) -> Result<String, String> {
    let Some(class) = n.attribute("customClass") else {
        if n.attribute("customModule").is_some() || n.attribute("customModuleProvider").is_some_and(|p| !p.is_empty()) {
            return fail("a custom module without a custom class is not supported by this ibtool");
        }
        return Ok(default.into());
    };
    let module = match (n.attribute("customModuleProvider"), module) {
        (Some("target"), Some(m)) => Some(m),
        (None | Some("" | "target"), _) => n.attribute("customModule"),
        (Some(p), _) => return fail(format!("customModuleProvider={p:?} is not supported by this ibtool")),
    };
    Ok(match module {
        None | Some("") => class.to_string(),
        Some("Swift") => format!("_TtCs{}{class}", class.len()),
        Some(m) if m.starts_with("__") => return fail(format!("custom module {m} is not supported by this ibtool")),
        Some(m) => format!("_TtC{}{m}{}{class}", m.len(), class.len()),
    })
}

/// A label's minimum font size and scale factor, when it adjusts its font
/// size to fit (the default) and has either. A scale factor's size is
/// rounded to the nearest quarter point, halves up; a size's factor is the
/// size over the font's.
fn minimum_font_size(n: Node) -> Result<Option<(f32, f32)>, String> {
    if n.attribute("adjustsFontSizeToFit").is_some() {
        return Ok(None);
    }
    let (factor, size) = (n.attribute("minimumScaleFactor"), n.attribute("minimumFontSize"));
    if factor.is_none() && size.map_or(true, |s| number(s) == Ok(0.0)) {
        return Ok(None);
    }
    let font_size = child(n, "fontDescription", "fontDescription")
        .and_then(|f| f.attribute("pointSize"))
        .ok_or("a minimum font size without the font's point size is not supported by this ibtool")?;
    let font_size = number(font_size)?;
    Ok(Some(match (factor, size) {
        (Some(factor), _) => {
            let factor = number(factor)? as f32;
            (((factor as f64 * font_size * 4.0).round() / 4.0) as f32, factor)
        }
        (None, Some(size)) => {
            let size = number(size)? as f32;
            (size, (size as f64 / font_size) as f32)
        }
        (None, None) => unreachable!(),
    }))
}

/// A font: `<fontDescription>`, or a control's default system font. Equal
/// fonts are one object.
fn font(nib: &mut Nib, n: Option<Node>, default_size: Option<f64>) -> Result<Value, String> {
    let (style, name, size) = match n {
        Some(n) => {
            if let Some(style) = n.attribute("style") {
                only_attributes(n, &[("key", None), ("style", None)])?;
                (Some(style), None, 0.0)
            } else if let Some(name) = n.attribute("name") {
                only_attributes(n, &[("key", None), ("name", None), ("family", None), ("pointSize", None)])?;
                (None, Some(name), number(attr(n, "pointSize")?)?)
            } else {
                only_attributes(n, &[("key", None), ("type", Some("system")), ("pointSize", None)])?;
                (None, None, number(attr(n, "pointSize")?)?)
            }
        }
        None => (None, None, default_size.ok_or("a font is required")?),
    };
    // Xcode archives a font it finds under its own name.
    let name = name.map(|name| fonts::ALIASES.iter().find(|(a, _)| *a == name).map_or(name, |(_, to)| to));
    let key = format!("font:{style:?}:{name:?}:{size}");
    if let Some(v) = nib.existing(&key) {
        return Ok(v);
    }
    let mut values = Vec::new();
    let (font_name, traits, ns_size, system) = match (style, name) {
        (Some(style), _) => {
            let (name, traits, ns_size) = text_style(nib.platform, style)?;
            let name_value = nib.string(name);
            values.push(("UIFontName", name_value));
            values.push(("UIIBTextStyle", nib.string(style)));
            (name, traits, ns_size, true)
        }
        (None, Some(name)) => {
            // One Xcode doesn't find is archived differently (see ibtool.md).
            let traits = match fonts::FONTS.binary_search_by(|(n, _)| n.cmp(&name)) {
                Ok(i) => fonts::FONTS[i].1,
                Err(_) => return fail(format!("font {name} is not one Xcode's ibtool finds for iOS, which is not supported by this ibtool")),
            };
            values.push(("UIFontName", nib.string(name)));
            let name_key = nib.string("NSFontNameAttribute");
            let name_value = nib.string(name);
            let size_key = nib.string("NSFontSizeAttribute");
            let size_value = nib.double_number(size);
            let attributes = nib.array("NSDictionary", vec![name_key, name_value, size_key, size_value]);
            values.push(("UIFontDescriptor", Value::New(Object::new("UIFontDescriptor", vec![
                ("UIFontDescriptorAttributes", attributes),
                ("UIFontDescriptorOptions", Value::Int(0x8000_0000)),
            ]))));
            (name, traits, size, false)
        }
        (None, None) => {
            let name = ".SFUI-Regular";
            values.push(("UIFontName", nib.string(name)));
            let usage_key = nib.string("NSCTFontUIUsageAttribute");
            let usage = nib.string("CTFontRegularUsage");
            let size_key = nib.string("NSFontSizeAttribute");
            let size_value = nib.double_number(size);
            let attributes = nib.array("NSDictionary", vec![usage_key, usage, size_key, size_value]);
            values.push(("UIFontDescriptor", Value::New(Object::new("UIFontDescriptor", vec![
                ("UIFontDescriptorAttributes", attributes),
                ("UIFontDescriptorOptions", Value::Int(2147517444)),
            ]))));
            (name, 0, size, true)
        }
    };
    values.push(("UIFontPointSize", Value::Double(size)));
    values.push(("UIFontTextStyleForScaling", Value::Nil));
    values.push(("UIFontPointSizeForScaling", Value::Double(0.0)));
    values.push(("UIFontMaximumPointSizeAfterScaling", Value::Double(0.0)));
    values.push(("UIFontTraits", Value::Int(traits)));
    values.push(("UISystemFont", Value::Bool(system)));
    values.push(("NSName", nib.string(font_name)));
    values.push(("NSSize", Value::Double(ns_size)));
    Ok(nib.shared(key, || Object::new("UIFont", values).with_fallback("NSFont")))
}

struct Frame {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn frame(n: Node) -> Result<Frame, String> {
    let r = child(n, "rect", "frame").ok_or_else(|| format!("<{}> without a frame", n.tag_name().name()))?;
    Ok(Frame { x: number(attr(r, "x")?)?, y: number(attr(r, "y")?)?, w: number(attr(r, "width")?)?, h: number(attr(r, "height")?)? })
}

/// A subview's frame. Without one, or with an all-zero one, a label or a
/// plain view is 1000x1000 at the origin and a button has no size.
fn subview_frame(n: Node) -> Result<Frame, String> {
    if let Some(f) = child(n, "rect", "frame").map(|_| frame(n)).transpose()? {
        if f.x != 0.0 || f.y != 0.0 || f.w != 0.0 || f.h != 0.0 {
            return Ok(f);
        }
    }
    match n.tag_name().name() {
        "label" | "view" => Ok(Frame { x: 0.0, y: 0.0, w: 1000.0, h: 1000.0 }),
        "button" => Ok(Frame { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }),
        other => fail(format!("<{other}> without a frame")),
    }
}

/// Bounds and centre, each left out when zero.
fn geometry(values: &mut Vec<(&'static str, Value)>, f: &Frame) {
    if f.w != 0.0 || f.h != 0.0 {
        values.push(("UIBounds", doubles(&[0.0, 0.0, f.w, f.h])));
    }
    let center = [f.x + f.w / 2.0, f.y + f.h / 2.0];
    if center != [0.0, 0.0] {
        values.push(("UICenter", doubles(&center)));
    }
}

fn view_name(id: &str) -> String {
    format!("view {id}")
}

/// What compiling a scene's view collects besides the view itself.
struct Scene<'a> {
    vc_id: &'a str,
    /// ibtool's `--module`.
    module: Option<&'a str>,
    /// Subviews in depth-first preorder, with their superview's name (None
    /// for the root view's), for the nib's objects.
    subviews: Vec<(Option<String>, String)>,
    /// The superview of the subviews being compiled.
    parent: Option<String>,
    /// Control actions: (selector, source, event mask), in depth-first order.
    actions: Vec<(String, String, i64)>,
    /// The constraints subviews hold, with the holder's name, for the nib's
    /// objects.
    own_constraints: Vec<(String, String)>,
    /// Ids of the views some constraint is on.
    constrained: Vec<&'a str>,
    /// Ids of the legacy top and bottom layout guides.
    guides: Vec<(&'a str, &'static str)>,
}

fn hugging(nib: &mut Nib, n: Node) -> Result<Value, String> {
    let h = attr(n, "horizontalHuggingPriority")?;
    let v = attr(n, "verticalHuggingPriority")?;
    Ok(nib.string(&format!("{{{h}, {v}}}")))
}

fn text_alignment(s: &str) -> Result<i64, String> {
    Ok(match s {
        "center" => 1,
        "natural" => 4,
        other => return fail(format!("textAlignment {other} is not supported by this ibtool")),
    })
}

/// Whether a view is archived as not translating its autoresizing mask:
/// when the document says so and a constraint is on it.
/// A label's or button's autoresizing mask, which Xcode leaves out when
/// the view doesn't translate it into constraints.
fn ignored_autoresizing_mask(n: Node) -> Result<(), String> {
    if children(n, "autoresizingMask").next().is_some() && n.attribute("translatesAutoresizingMaskIntoConstraints").is_none() {
        return fail(format!("<autoresizingMask> inside a <{}> that translates it is not supported by this ibtool", n.tag_name().name()));
    }
    Ok(())
}

fn does_not_translate(scene: &Scene, n: Node) -> bool {
    n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() && n.attribute("id").is_some_and(|id| scene.constrained.contains(&id))
}

/// The ids of the views the constraints in `view` are on: their items, the
/// views whose own constraints name no first item, and a stack view's
/// arranged subviews, which it constrains.
fn constrained_views<'a>(view: Node<'a, '_>) -> Vec<&'a str> {
    let mut ids = Vec::new();
    for stack in view.descendants().filter(|d| d.has_tag_name("stackView")) {
        for s in children(stack, "subviews") {
            ids.extend(s.children().filter(|c| c.is_element()).filter_map(|c| c.attribute("id")));
        }
    }
    for c in view.descendants().filter(|d| d.has_tag_name("constraint")) {
        let owner = c.parent().and_then(|p| p.parent()).and_then(|v| v.attribute("id"));
        ids.extend(c.attribute("firstItem").or(owner));
        ids.extend(c.attribute("secondItem"));
    }
    ids
}

/// The constraints a subview holds: on its own size, width before height
/// whatever the document's order and each dimension's constant before its
/// aspect ratio; then, for a container, those between it
/// and its subviews or among them, ordered as a root view's are.
fn own_constraints(nib: &mut Nib, scene: &mut Scene, n: Node, name: &str) -> Result<Option<Value>, String> {
    let Some(cs) = children(n, "constraints").next() else { return Ok(None) };
    only_children(cs, &["constraint"])?;
    let id = attr(n, "id")?;
    let direct: Vec<Node> = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    let mut sizes = Vec::new();
    let mut others: Vec<((f64, f64), (&str, u8, (usize, usize)), Node, &str, &str)> = Vec::new();
    for c in children(cs, "constraint") {
        only_attributes(c, &[("firstItem", None), ("firstAttribute", None), ("secondItem", None), ("secondAttribute", None), ("multiplier", None), ("constant", None), ("id", None)])?;
        let first = c.attribute("firstItem").unwrap_or(id);
        let a = attr(c, "firstAttribute")?;
        let Some(second) = c.attribute("secondItem") else {
            if first != id || c.attribute("secondAttribute").is_some() || c.attribute("multiplier").is_some() {
                return fail("a subview's constraint without a second item on another view is not supported by this ibtool");
            }
            let rank = 2 * ["width", "height"].iter().position(|x| *x == a).ok_or_else(|| format!("a subview's own constraint on {a} is not supported by this ibtool"))?;
            if sizes.iter().any(|(r, _)| *r == rank) {
                return fail("two constraints on a subview's same dimension are not supported by this ibtool");
            }
            sizes.push((rank, c));
            continue;
        };
        if first == second {
            // An aspect ratio: after the constant on its dimension.
            let rank = match (first == id, a, attr(c, "secondAttribute")?) {
                (true, "width", "height") => 1,
                (true, "height", "width") => 3,
                _ => return fail("a constraint between an item and itself other than its own aspect ratio is not supported by this ibtool"),
            };
            if sizes.iter().any(|(r, _)| *r == rank) {
                return fail("two aspect ratios on a subview's same dimension are not supported by this ibtool");
            }
            sizes.push((rank, c));
            continue;
        }
        let key = constraint_key(id, first, a, second, attr(c, "secondAttribute")?)?;
        if others.iter().any(|(_, k, ..)| *k == key) {
            return fail("two constraints Xcode orders unstably (the same items and attributes) are not supported by this ibtool");
        }
        let v = direct.iter().find(|v| v.attribute("id") == Some(key.0)).ok_or_else(|| format!("a subview's constraints on {} are not supported by this ibtool", key.0))?;
        for item in [first, second] {
            if item != id && !direct.iter().any(|v| v.attribute("id") == Some(item)) {
                return fail(format!("a subview's constraints on {item} are not supported by this ibtool"));
            }
        }
        let f = subview_frame(*v)?;
        others.push(((f.x * f.x + f.y * f.y, f.x), key, c, first, second));
    }
    sizes.sort_by_key(|f| f.0);
    others.sort_by(|a, b| a.0 .0.total_cmp(&b.0 .0).then(a.0 .1.total_cmp(&b.0 .1)).then(a.1.cmp(&b.1)));
    let mut values = Vec::new();
    let item = |i: &str| reference(view_name(i));
    let mut made = Vec::new();
    for (rank, c) in sizes {
        let constant = c.attribute("constant").map(number).transpose()?.unwrap_or(0.0);
        let first = layout_attribute(attr(c, "firstAttribute")?)?;
        made.push(if rank % 2 == 0 {
            constraint(reference(name.to_string()), first, None, constant, None)
        } else {
            let second = Some((reference(name.to_string()), layout_attribute(attr(c, "secondAttribute")?)?));
            multiplied(constraint(reference(name.to_string()), first, second, constant, None), multiplier(c)?)
        });
    }
    for (_, _, c, first, second) in others {
        let constant = c.attribute("constant").map(number).transpose()?.unwrap_or(0.0);
        made.push(multiplied(constraint(item(first), layout_attribute(attr(c, "firstAttribute")?)?, Some((item(second), layout_attribute(attr(c, "secondAttribute")?)?)), constant, None), multiplier(c)?));
    }
    for m in made {
        let constraint_name = nib.fresh();
        scene.own_constraints.push((name.to_string(), constraint_name.clone()));
        match m {
            Value::New(o) => values.push(Value::New(o.named(constraint_name))),
            _ => unreachable!(),
        }
    }
    Ok(Some(nib.array("NSMutableArray", values)))
}

fn subview(nib: &mut Nib, scene: &mut Scene, n: Node) -> Result<Value, String> {
    let id = attr(n, "id")?;
    let name = view_name(id);
    scene.subviews.push((scene.parent.clone(), name.clone()));
    let f = subview_frame(n)?;
    let mut values = Vec::new();
    geometry(&mut values, &f);
    let object = match n.tag_name().name() {
        "label" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("userInteractionEnabled", Some("NO")), ("contentMode", Some("left")),
                ("fixedFrame", Some("YES")), ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None), ("text", None), ("textAlignment", None),
                ("lineBreakMode", None), ("numberOfLines", None), ("baselineAdjustment", Some("alignBaselines")),
                ("adjustsFontSizeToFit", Some("NO")), ("minimumFontSize", None), ("minimumScaleFactor", None),
                ("adjustsLetterSpacingToFitWidth", Some("YES")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "autoresizingMask", "fontDescription", "nil", "color", "constraints"])?;
            ignored_autoresizing_mask(n)?;
            if child(n, "nil", "highlightedColor").is_none() {
                return fail("labels with a highlighted colour are not supported by this ibtool");
            }
            // No text colour at all is a declared labelColor; a nil one is
            // the default.
            let (text_color, declared_color) = match (child(n, "color", "textColor"), child(n, "nil", "textColor")) {
                (Some(c), None) => (Some(c), true),
                (None, Some(_)) => (None, false),
                (None, None) => (None, true),
                _ => return fail("a label needs one text colour or nil"),
            };
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIUserInteractionDisabled", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            values.push(("UIContentMode", Value::Int(7)));
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewContentHuggingPriority", hugging(nib, n)?));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIText", nib.text(id, "text", attr(n, "text")?)));
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), None)?));
            // A declared text colour, even labelColor, stays as it is when
            // the trait collection changes.
            let text_color = match text_color {
                Some(c) => color(nib, c)?,
                None => nib.system_color("labelColor", &SystemColor::Gray([0.0, 1.0])),
            };
            values.push(("UITextColor", text_color));
            values.push(("UIShadowOffset", doubles(&[0.0, -1.0])));
            if let Some(lines) = n.attribute("numberOfLines") {
                values.push(("UINumberOfLines", Value::Int(lines.parse().map_err(|_| format!("not a number: {lines}"))?)));
            }
            // Tail truncation, the default, is left out.
            let line_break = match attr(n, "lineBreakMode")? {
                "wordWrap" => Some(0),
                "characterWrap" => Some(1),
                "clip" => Some(2),
                "headTruncation" => Some(3),
                "tailTruncation" => None,
                "middleTruncation" => Some(5),
                other => return fail(format!("lineBreakMode=\"{other}\" on <label> is not supported by this ibtool")),
            };
            let minimum = minimum_font_size(n)?;
            if n.attribute("adjustsFontSizeToFit").is_none() {
                values.push(("UIAdjustsFontSizeToFit", Value::Bool(true)));
            }
            if let Some((size, _)) = minimum {
                values.push(("UIMinimumFontSize", Value::Float(size)));
            }
            if let Some(mode) = line_break {
                values.push(("UILineBreakMode", Value::Int(mode)));
            }
            values.push(("UITextAlignment", Value::Int(text_alignment(attr(n, "textAlignment")?)?)));
            if let Some((_, scale)) = minimum {
                values.push(("UIMinimumScaleFactor", Value::Float(scale)));
            }
            if n.attribute("adjustsLetterSpacingToFitWidth").is_some() {
                values.push(("UIAdjustsLetterSpacingToFit", Value::Bool(true)));
            }
            values.push(("UIDisableUpdateTextColorOnTraitCollectionChange", Value::Bool(declared_color)));
            Object::new("UILabel", values)
        }
        "textField" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("contentMode", Some("scaleToFill")),
                ("contentHorizontalAlignment", Some("left")), ("contentVerticalAlignment", Some("center")), ("borderStyle", Some("roundedRect")),
                ("textAlignment", Some("natural")), ("minimumFontSize", None), ("clearButtonMode", None), ("placeholder", None), ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
                ("id", None),
            ])?;
            let clear_button_mode = match n.attribute("clearButtonMode") {
                None => None,
                Some("whileEditing") => Some(1),
                Some("unlessEditing") => Some(2),
                Some("always") => Some(3),
                Some(other) => return fail(format!("clearButtonMode=\"{other}\" on <textField> is not supported by this ibtool")),
            };
            only_children(n, &["rect", "nil", "color", "fontDescription", "textInputTraits", "constraints"])?;
            let traits = children(n, "textInputTraits").next().ok_or("text field without text input traits")?;
            if traits.attributes().len() != 1 || traits.has_children() {
                return fail("text input traits are not supported by this ibtool");
            }
            // No text colour at all is the same as a nil one.
            let text_color = match (child(n, "color", "textColor"), child(n, "nil", "textColor")) {
                (Some(c), None) => Some(c),
                (None, _) => None,
                _ => return fail("a text field needs one text colour or nil"),
            };
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIContentHorizontalAlignment", Value::Int(1)));
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), None)?));
            if let Some(placeholder) = n.attribute("placeholder") {
                values.push(("UIPlaceholder", nib.text(id, "placeholder", placeholder)));
            }
            values.push(("UITextAlignment", Value::Int(4)));
            let text_color = match text_color {
                Some(c) => color(nib, c)?,
                None => nib.system_color("labelColor", &SystemColor::Gray([0.0, 1.0])),
            };
            values.push(("UITextColor", text_color));
            values.push(("UIBorderStyle", Value::Int(3)));
            values.push(("UIAdjustsFontSizeToFit", Value::Bool(true)));
            values.push(("UIMinimumFontSize", Value::Float(number(attr(n, "minimumFontSize")?)? as f32)));
            if let Some(mode) = clear_button_mode {
                values.push(("UIClearButtonMode", Value::Int(mode)));
            }
            values.push(("UIClearButtonOffset", doubles(&[3.0, 1.0])));
            values.push(("UIDisableTextColorUpdateOnTraitCollectionChange", Value::Bool(true)));
            values.push(("UIRoundedRectBackgroundCornerRadius", Value::Double(0.0)));
            Object::new("UITextField", values)
        }
        "button" => {
            // `fixedFrame` is Interface Builder's alone, and a system button
            // is archived as a rounded-rect one; a custom one has no type.
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("fixedFrame", Some("YES")), ("contentHorizontalAlignment", Some("center")),
                ("contentVerticalAlignment", Some("center")), ("buttonType", None), ("lineBreakMode", Some("middleTruncation")),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            let button_type = match n.attribute("buttonType") {
                Some("roundedRect" | "system") => Some(1),
                None => None,
                Some(other) => return fail(format!("buttonType=\"{other}\" on <button> is not supported by this ibtool")),
            };
            only_children(n, &["rect", "autoresizingMask", "state", "connections", "fontDescription", "color", "constraints"])?;
            ignored_autoresizing_mask(n)?;
            let states: Vec<Node> = children(n, "state").collect();
            if states.len() != 1 || states[0].attribute("key") != Some("normal") {
                return fail("buttons other than one with a normal state are not supported by this ibtool");
            }
            only_attributes(states[0], &[("key", None), ("title", None)])?;
            only_children(states[0], &["color"])?;
            for c in children(states[0], "color") {
                if !matches!(c.attribute("key"), Some("titleColor" | "titleShadowColor")) {
                    return fail("button state colours other than title and title shadow colours are not supported by this ibtool");
                }
            }
            for c in children(n, "color") {
                if !matches!(c.attribute("key"), Some("backgroundColor" | "tintColor")) {
                    return fail("button colours other than background and tint colours are not supported by this ibtool");
                }
            }
            for c in children(n, "connections") {
                only_children(c, &["action"])?;
                for a in children(c, "action") {
                    only_attributes(a, &[("selector", None), ("destination", None), ("eventType", Some("touchUpInside")), ("id", None)])?;
                    if attr(a, "destination")? != scene.vc_id {
                        return fail("actions that don't target the scene's view controller are not supported by this ibtool");
                    }
                    scene.actions.push((attr(a, "selector")?.to_string(), name.clone(), 64));
                }
            }
            if let Some(c) = child(n, "color", "backgroundColor") {
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            if n.attribute("opaque").is_none() {
                values.push(("UIOpaque", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if let Some(c) = child(n, "color", "tintColor") {
                values.push(("UITintColor", color(nib, c)?));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            let state = nib.int_number(0);
            let mut content = Vec::new();
            if let Some(title) = states[0].attribute("title") {
                content.push(("UITitle", nib.text(id, "normalTitle", title)));
            }
            if let Some(c) = child(states[0], "color", "titleColor") {
                content.push(("UITitleColor", color(nib, c)?));
            }
            if let Some(c) = child(states[0], "color", "titleShadowColor") {
                content.push(("UIShadowColor", color(nib, c)?));
            }
            let content = Value::New(Object::new("UIButtonContent", content));
            values.push(("UIButtonStatefulContent", nib.array("NSMutableDictionary", vec![state, content])));
            values.push(("UIAdjustsImageWhenHighlighted", Value::Bool(true)));
            values.push(("UIAdjustsImageWhenDisabled", Value::Bool(true)));
            if let Some(t) = button_type {
                values.push(("UIButtonType", Value::Int(t)));
            }
            // A custom button's default font is larger.
            let default_size = if button_type.is_some() { 15.0 } else { 18.0 };
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), Some(default_size))?));
            values.push(("UIButtonConfiguration", Value::Nil));
            Object::new("UIButton", values)
        }
        "stackView" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("axis", Some("vertical")), ("spacing", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "subviews"])?;
            let mut arranged = Vec::new();
            for s in children(n, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    let outer = scene.parent.replace(name.clone());
                    let v = subview(nib, scene, c);
                    scene.parent = outer;
                    arranged.push(v?);
                }
            }
            let refs = arranged_names(&arranged);
            values.push(("UISubviews", nib.array("NSMutableArray", arranged)));
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            values.push(("UIViewLayoutMargins", doubles(&[0.0, 0.0, 0.0, 0.0])));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIStackViewArrangedSubviews", nib.array("NSArray", refs)));
            values.push(("UIStackViewAxis", Value::Int(1)));
            values.push(("UIStackViewDistribution", Value::Int(0)));
            values.push(("UIStackViewAlignment", Value::Int(0)));
            values.push(("UIStackViewSpacing", Value::Double(number(attr(n, "spacing")?)?)));
            values.push(("UIStackViewBaselineRelative", Value::Bool(false)));
            values.push(("UIStackViewLayoutMarginsRelative", Value::Bool(false)));
            Object::new("UIStackView", values)
        }
        "view" => {
            only_attributes(n, &[
                ("contentMode", Some("scaleToFill")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
                ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
            ])?;
            only_children(n, &["rect", "autoresizingMask", "color", "subviews", "constraints"])?;
            let mut inner = Vec::new();
            for s in children(n, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    let outer = scene.parent.replace(name.clone());
                    let v = subview(nib, scene, c);
                    scene.parent = outer;
                    inner.push(v?);
                }
            }
            if !inner.is_empty() {
                values.push(("UISubviews", nib.array("NSMutableArray", inner)));
            }
            for c in children(n, "color") {
                if c.attribute("key") != Some("backgroundColor") {
                    return fail("view colours other than a background colour are not supported by this ibtool");
                }
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIOpaque", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            let mask = match children(n, "autoresizingMask").next() {
                Some(m) => autoresizing_mask(m)?,
                None => 36,
            };
            values.push(("UIAutoresizingMask", Value::Int(mask)));
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            // A custom class is swapped in for the view when it loads.
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, "UIView")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UIView")));
                Object::new("UIClassSwapper", values)
            } else {
                if n.attribute("customModule").is_some() || n.attribute("customModuleProvider").is_some_and(|p| !p.is_empty()) {
                    return fail("a custom module without a custom class is not supported by this ibtool");
                }
                Object::new("UIView", values)
            }
        }
        "imageView" => {
            only_attributes(n, &[
                ("clipsSubviews", Some("YES")), ("userInteractionEnabled", Some("NO")), ("contentMode", None),
                ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None), ("image", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "color", "constraints"])?;
            if n.attribute("userInteractionEnabled").is_none() {
                return fail("image views with user interaction are not supported by this ibtool");
            }
            let content_mode = match attr(n, "contentMode")? {
                "scaleToFill" => 0,
                "scaleAspectFit" => 1,
                "scaleAspectFill" => 2,
                "center" => 4,
                other => return fail(format!("contentMode {other} on <imageView> is not supported by this ibtool")),
            };
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIOpaque", Value::Bool(true)));
            values.push(("UIUserInteractionDisabled", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if content_mode != 0 {
                values.push(("UIContentMode", Value::Int(content_mode)));
            }
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            for c in children(n, "color") {
                if c.attribute("key") != Some("tintColor") {
                    return fail("image view colours other than a tint colour are not supported by this ibtool");
                }
                values.push(("UITintColor", color(nib, c)?));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewContentHuggingPriority", hugging(nib, n)?));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if let Some(image) = n.attribute("image") {
                values.push(("UIImage", nib.image(image)));
            }
            Object::new("UIImageView", values)
        }
        other => return fail(format!("<{other}> is not supported by this ibtool")),
    };
    Ok(Value::New(object.named(name)))
}

/// References to the views just built, in order.
fn arranged_names(views: &[Value]) -> Vec<Value> {
    views
        .iter()
        .map(|v| match v {
            Value::New(o) => reference(o.name.clone().expect("subviews are named")),
            _ => unreachable!("subviews are new objects"),
        })
        .collect()
}

/// The size Xcode lays a root view out at for a `<device>`, whatever the
/// view's frame says, or None for ids it doesn't know, where the frame is
/// used: measured per id and orientation (docs/clean-room/ibtool.md). Ids
/// not measured are refused.
fn device_size(platform: Platform, id: &str, orientation: &str) -> Result<Option<(f64, f64)>, String> {
    const PORTRAIT: [(&str, f64, f64); 17] = [
        ("retina4_0", 320.0, 568.0), ("retina4_7", 375.0, 667.0), ("retina5_5", 414.0, 736.0), ("retina5_9", 375.0, 812.0),
        ("retina6_0", 390.0, 844.0), ("retina6_1", 414.0, 896.0), ("retina6_12", 393.0, 852.0), ("retina6_3", 402.0, 874.0),
        ("retina6_5", 414.0, 896.0), ("retina6_7", 430.0, 932.0), ("retina6_9", 440.0, 956.0), ("ipad7_9", 744.0, 1133.0),
        ("ipad9_7", 768.0, 1024.0), ("ipad10_2", 810.0, 1080.0), ("ipad10_9rounded", 820.0, 1180.0),
        ("ipad11_0rounded", 834.0, 1210.0), ("ipad12_9rounded", 1024.0, 1366.0),
    ];
    // Laid out the same whichever the orientation.
    const FIXED: [&str; 2] = ["retina3_5", "retina6_72"];
    // Not known to Xcode's ibtool: the view's frame is used.
    const UNKNOWN: [&str; 5] = ["retina5_4", "ipad10_5", "ipad12_9", "ipad8_3rounded", "ipad13_0rounded"];
    if orientation != "portrait" && orientation != "landscape" {
        return fail(format!("device orientation {orientation} is not supported by this ibtool"));
    }
    if platform == Platform::Tvos {
        return match (id, orientation) {
            ("appleTV", "landscape") => Ok(Some((1920.0, 1080.0))),
            ("appleTV4K", _) => Ok(None),
            _ => fail(format!("device {id} ({orientation}) is not supported by this ibtool")),
        };
    }
    if FIXED.contains(&id) {
        return Ok(Some((393.0, 852.0)));
    }
    if UNKNOWN.contains(&id) {
        return Ok(None);
    }
    let &(_, w, h) = PORTRAIT.iter().find(|(d, ..)| *d == id).ok_or_else(|| format!("device {id} is not supported by this ibtool"))?;
    Ok(Some(if orientation == "landscape" { (h, w) } else { (w, h) }))
}

/// Where Xcode puts a root view's constraint (docs/clean-room/ibtool.md):
/// grouped by the item other than the root view, or the first item between
/// siblings, the groups by where their item is (below); in a group,
/// constraints with the root view first, by their two attributes in one
/// order, then those between siblings, by first and second attribute in
/// another. Equal keys are refused: Xcode's order for them isn't stable.
fn constraint_key<'a>(root: &str, first: &'a str, a1: &str, second: &'a str, a2: &str) -> Result<(&'a str, u8, (usize, usize)), String> {
    const WITH_ROOT: [&str; 8] = ["centerX", "centerY", "trailing", "leading", "width", "bottom", "top", "height"];
    const SIBLINGS: [&str; 8] = ["centerX", "trailing", "leading", "centerY", "bottom", "top", "width", "height"];
    let rank = |order: &[&str; 8], a: &str| order.iter().position(|o| *o == a).ok_or_else(|| format!("constraints on {a} are not supported by this ibtool"));
    let axis = |a: &str| match a {
        "leading" | "trailing" | "centerX" => 0,
        "top" | "bottom" | "centerY" => 1,
        _ => 2,
    };
    if a1 != a2 && (axis(a1) != axis(a2) || axis(a1) == 2) {
        return fail(format!("constraints between {a1} and {a2} are not supported by this ibtool"));
    }
    let (group, kind, attributes) = if first == root || second == root {
        let (r1, r2) = (rank(&WITH_ROOT, a1)?, rank(&WITH_ROOT, a2)?);
        (if first == root { second } else { first }, 0, (r1.min(r2), r1.max(r2)))
    } else {
        (first, 1, (rank(&SIBLINGS, a1)?, rank(&SIBLINGS, a2)?))
    };
    Ok((group, kind, attributes))
}

fn constraint(first: Value, first_attr: i64, second: Option<(Value, i64)>, constant: f64, priority: Option<i64>) -> Value {
    let mut values = vec![("NSFirstItem", first), ("NSFirstAttributeV2", Value::Int(first_attr)), ("NSFirstAttribute", Value::Int(first_attr))];
    if let Some((item, a)) = second {
        values.push(("NSSecondItem", item));
        values.push(("NSSecondAttributeV2", Value::Int(a)));
        values.push(("NSSecondAttribute", Value::Int(a)));
    }
    if constant != 0.0 {
        values.push(("NSConstantV2", Value::Double(constant)));
    }
    values.push(("NSShouldBeArchived", Value::Bool(true)));
    if constant != 0.0 {
        values.push(("NSConstant", Value::Double(constant)));
    }
    if let Some(p) = priority {
        values.push(("NSPriority", Value::Int(p)));
    }
    Value::New(Object::new("NSLayoutConstraint", values))
}

/// The root view's safe-area layout guide, as archived where a constraint
/// first uses it: owned by the view, with the system constraints that pin
/// its edges to the view's.
fn safe_area_guide(nib: &mut Nib, root: &str) -> Value {
    let identifier = nib.string("UIViewSafeAreaLayoutGuide");
    let me = || reference("safe area guide");
    let view = || reference(root.to_string());
    // NSLayoutAttribute: left 1, right 2, top 3, bottom 4.
    let pins = [("top", me(), 3, view()), ("left", me(), 1, view()), ("bottom", view(), 4, me()), ("right", view(), 2, me())];
    let mut system = Vec::new();
    for (edge, first, a, second) in pins {
        let name = nib.string(&format!("UIViewSafeAreaLayoutGuide-{edge}"));
        system.push(Value::New(Object::new("NSLayoutConstraint", vec![
            ("NSFirstItem", first),
            ("NSFirstAttributeV2", Value::Int(a)),
            ("NSFirstAttribute", Value::Int(a)),
            ("NSSecondItem", second),
            ("NSSecondAttributeV2", Value::Int(a)),
            ("NSSecondAttribute", Value::Int(a)),
            ("NSShouldBeArchived", Value::Bool(false)),
            ("NSLayoutIdentifier", name),
        ])));
    }
    let system = nib.array("NSArray", system);
    Value::New(Object::new("UILayoutGuide", vec![
        ("UILayoutGuideOwningView", view()),
        ("UILayoutGuideIdentifier", identifier),
        ("UILayoutGuideOwningViewIsLocked", Value::Bool(true)),
        ("UILayoutGuideShouldBeArchived", Value::Bool(true)),
        ("UILayoutGuideAllowsNegativeDimensions", Value::Bool(true)),
        ("UILayoutGuideSystemConstraints", system),
    ]).named("safe area guide"))
}

/// A constraint's multiplier, as Xcode archives it: a float's value (a
/// ratio `a:b` divided), or None for 1.
fn multiplier(c: Node) -> Result<Option<f64>, String> {
    let Some(m) = c.attribute("multiplier") else { return Ok(None) };
    let value = match m.split_once(':') {
        Some((a, b)) => number(a)? / number(b)?,
        None => number(m)?,
    };
    Ok(if value == 1.0 { None } else { Some(value as f32 as f64) })
}

/// `constraint` with `NSMultiplier`, after the first attribute.
fn multiplied(c: Value, m: Option<f64>) -> Value {
    match (c, m) {
        (Value::New(mut o), Some(m)) => {
            let at = o.values.iter().position(|(k, _)| *k == "NSFirstAttribute").unwrap() + 1;
            o.values.insert(at, ("NSMultiplier", Value::Double(m)));
            Value::New(o)
        }
        (c, _) => c,
    }
}

/// A view controller's legacy top or bottom layout guide: a hidden view with
/// a width constraint UIKit replaces, and the constraints that pin it.
fn legacy_guide(nib: &mut Nib, root: &str, kind: &'static str) -> (Value, Vec<String>) {
    let name = format!("guide {kind}");
    let mut values = vec![
        ("UIDeepDrawRect", Value::Bool(true)),
        ("UIOpaque", Value::Bool(true)),
        ("UIHidden", Value::Bool(true)),
        ("UIAutoresizeSubviews", Value::Bool(true)),
        ("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)),
    ];
    let support = match constraint(reference(name.clone()), 7, None, 0.0, Some(999)) {
        Value::New(o) => Value::New(Object { class: "_UILayoutSupportConstraint", ..o }.with_fallback("NSLayoutConstraint")),
        _ => unreachable!(),
    };
    values.push(("UIViewAutolayoutConstraints", nib.array("NSMutableArray", vec![support])));
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    let identifier = if kind == "top" { "_UIViewControllerTop" } else { "_UIViewControllerBottom" };
    values.push(("_UILayoutGuideIdentifier", nib.string(identifier)));
    let me = || reference(name.clone());
    let view = || reference(root.to_string());
    let pins = if kind == "top" {
        vec![constraint(me(), 5, Some((view(), 5)), 0.0, None), constraint(me(), 3, Some((view(), 3)), 0.0, None), constraint(me(), 8, None, 0.0, None)]
    } else {
        vec![constraint(me(), 5, Some((view(), 5)), 0.0, None), constraint(me(), 8, None, 0.0, None), constraint(view(), 4, Some((me(), 4)), 0.0, None)]
    };
    let mut names = Vec::new();
    let named: Vec<Value> = pins
        .into_iter()
        .map(|p| match p {
            Value::New(o) => {
                let n = nib.fresh();
                names.push(n.clone());
                Value::New(o.named(n))
            }
            _ => unreachable!(),
        })
        .collect();
    values.push(("_UILayoutGuideConstraintsToRemove", nib.array("NSArray", named)));
    let guide = Object::new("_UILayoutGuide", values).with_fallback("UIView").named(name);
    (Value::New(guide), names)
}

fn proxy(nib: &mut Nib, id: &str, name: &str) -> Value {
    let identifier = nib.string(id);
    Value::New(Object::new("UIProxyObject", vec![("UIProxiedObjectIdentifier", identifier)]).named(name))
}

fn outlet(nib: &mut Nib, label: &str, source: &str, destination: &str) -> Value {
    let label = nib.string(label);
    Value::New(Object::new("UIRuntimeOutletConnection", vec![("UILabel", label), ("UISource", reference(source)), ("UIDestination", reference(destination))]))
}

fn root_object(nib: &mut Nib, top: Vec<Value>, objects: Vec<Value>, connections: Vec<Value>) -> Object {
    let top = nib.array("NSArray", top);
    let objects = nib.array("NSArray", objects);
    let connections = nib.array("NSArray", connections);
    Object::new("NSObject", vec![
        ("UINibTopLevelObjectsKey", top),
        ("UINibObjectsKey", objects),
        ("UINibConnectionsKey", connections),
        ("UINibVisibleWindowsKey", nib.empty_array()),
        ("UINibAccessibilityConfigurationsKey", nib.empty_array()),
        ("UINibTraitStorageListsKey", nib.empty_array()),
        ("UINibKeyValuePairsKey", nib.empty_array()),
        ("IBUINibLNEVersionKey", Value::Int(1)),
    ])
}

/// What a navigation controller's nib needs of its root view controller,
/// which it embeds.
struct RootController<'a, 'i> {
    vc: Node<'a, 'i>,
    /// Whether it has a nib of its own too (initial or identified).
    loadable: bool,
    view_nib_name: String,
    class_name: String,
    upstream_used: bool,
}

/// The id of a navigation controller's root view controller, its one
/// relationship segue.
fn navigation_root<'a>(nav: Node<'a, '_>) -> Result<&'a str, String> {
    only_children(nav, &["navigationBar", "connections", "toolbarItems", "nil"])?;
    for c in children(nav, "toolbarItems") {
        if c.has_children() {
            return fail("navigation controllers with toolbar items are not supported by this ibtool");
        }
    }
    for c in children(nav, "nil") {
        if c.attribute("name") != Some("viewControllers") {
            return fail("<nil> inside <navigationController> is not supported by this ibtool");
        }
    }
    let connections = children(nav, "connections").next().ok_or("a navigation controller without a root view controller")?;
    only_children(connections, &["segue"])?;
    let segues: Vec<Node> = children(connections, "segue").collect();
    if segues.len() != 1 {
        return fail("navigation controllers with segues other than their root's are not supported by this ibtool");
    }
    only_attributes(segues[0], &[("destination", None), ("kind", Some("relationship")), ("relationship", Some("rootViewController")), ("id", None)])?;
    attr(segues[0], "destination")
}

/// A navigation controller's nib: the scenes' placeholders, the controller
/// swapped in for UINavigationController, its root view controller (as in
/// its own nib, with its navigation item, real or implied) and its bar.
fn navigation_controller_nib(
    platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, nav: Node, root: &RootController,
) -> Result<(String, Vec<u8>), String> {
    only_attributes(nav, &[
        ("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("storyboardIdentifier", None),
        ("useStoryboardIdentifierAsRestorationIdentifier", Some("YES")), ("title", None), ("automaticallyAdjustsScrollViewInsets", Some("NO")),
        ("sceneMemberID", Some("viewController")),
    ])?;
    let nav_id = attr(nav, "id")?;
    let bar = children(nav, "navigationBar").next().ok_or("a navigation controller without a navigation bar")?;
    only_attributes(bar, &[("key", Some("navigationBar")), ("contentMode", Some("scaleToFill")), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("id", None)])?;
    only_children(bar, &["rect", "autoresizingMask"])?;
    for m in children(bar, "autoresizingMask") {
        if autoresizing_mask(m)? != 0 || m.has_children() {
            return fail("navigation bars with an autoresizing mask are not supported by this ibtool");
        }
    }
    let f = frame(bar)?;
    if f.x != 0.0 {
        return fail("navigation bars not at x 0 are not supported by this ibtool");
    }
    if let Some(identifier) = nav.attribute("storyboardIdentifier") {
        if identifier.is_empty() || identifier.contains('/') || !identifier.is_ascii() {
            return fail(format!("storyboard identifier {identifier:?} is not supported by this ibtool"));
        }
    } else if nav.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
        return fail("useStoryboardIdentifierAsRestorationIdentifier without a storyboard identifier is not supported by this ibtool");
    }
    let vc = root.vc;
    let name = nav.attribute("storyboardIdentifier").map_or_else(|| format!("UINavigationController-{nav_id}"), str::to_string);
    let mut nib = Nib::new(platform, color_matched, localizable);
    // The root's scene's storyboard placeholder comes first.
    let root_storyboard = proxy(&mut nib, "UIStoryboardPlaceholder", "root storyboard");
    let owner = proxy(&mut nib, "IBFilesOwner", "owner");
    let responder = proxy(&mut nib, "IBFirstResponder", "responder");
    let storyboard = proxy(&mut nib, "UIStoryboardPlaceholder", "storyboard");

    // The root view controller.
    let explicit_item = child(vc, "navigationItem", "navigationItem");
    let item = match explicit_item {
        Some(i) => match navigation_item(&mut nib, i)? {
            Value::New(mut o) => {
                o.values.push(("UINavigationBar", reference("bar")));
                Value::New(o)
            }
            _ => unreachable!(),
        },
        None => Value::New(Object::new("UINavigationItem", vec![
            ("UILargeTitleDisplayMode", Value::Int(0)),
            ("UINavigationBar", reference("bar")),
        ]).named("implied navigation item")),
    };
    let external = if root.upstream_used { vec![nib.string("UpstreamPlaceholder-1"), reference("root")] } else { Vec::new() };
    let external = nib.array("NSDictionary", external);
    let nib_name = nib.string(&root.view_nib_name);
    let mut root_values = Vec::new();
    if let Some(title) = vc.attribute("title") {
        root_values.push(("UITitle", nib.text(attr(vc, "id")?, "title", title)));
    }
    root_values.extend([
        ("UINavigationItem", item),
        ("UIExternalObjectsTableForViewLoading", external),
        ("UINibName", nib_name),
    ]);
    if let Some(identifier) = vc.attribute("storyboardIdentifier") {
        root_values.push(("UIStoryboardIdentifier", nib.string(identifier)));
        if vc.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            root_values.push(("UIRestorationIdentifier", nib.string(identifier)));
        }
    }
    root_values.push(("UIParentViewController", reference("controller")));
    if vc.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
        root_values.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
    }
    // One with its own nib too is swapped in, as it is there.
    let root_controller = if vc.attribute("customClass").is_some() || root.loadable {
        root_values.push(("UIClassName", nib.string(&root.class_name)));
        root_values.push(("UIOriginalClassName", nib.string("UIViewController")));
        Object::new("UIClassSwapper", root_values)
    } else {
        Object::new("UIViewController", root_values)
    };
    let children_value = nib.array("NSMutableArray", vec![Value::New(root_controller.named("root"))]);

    // The bar, at the height Xcode gives it whatever the document says.
    let hugging = nib.string("{250, 250}");
    let mut bar_values = vec![
        ("UIBounds", doubles(&[0.0, 0.0, 0.0, 54.0])),
        ("UICenter", doubles(&[f.x, f.y + 27.0])),
        ("UIDeepDrawRect", Value::Bool(true)),
        ("UIAutoresizeSubviews", Value::Bool(true)),
        ("UIAutoresizingMask", Value::Int(2)),
        ("UIViewContentHuggingPriority", hugging),
        ("UIViewLayoutMargins", doubles(&[0.0, 0.0, 0.0, 0.0])),
        ("UIViewLayoutMarginsAreDirectional", Value::Bool(true)),
    ];
    if bar.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
        bar_values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
    }
    bar_values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut bar_values);
    let items = nib.array("NSMutableArray", vec![reference(if explicit_item.is_some() { "navigation item" } else { "implied navigation item" })]);
    bar_values.push(("UIIsLocked", Value::Bool(true)));
    bar_values.push(("UIItems", items));
    bar_values.push(("UIDelegate", reference("controller")));
    bar_values.push(("UIBarPosition", Value::Int(2)));
    bar_values.push(("UIBarTranslucence", Value::Int(1)));

    let mut swapper = Vec::new();
    if let Some(title) = nav.attribute("title") {
        swapper.push(("UITitle", nib.text(nav_id, "title", title)));
    }
    if let Some(identifier) = nav.attribute("storyboardIdentifier") {
        swapper.push(("UIStoryboardIdentifier", nib.string(identifier)));
        if nav.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            swapper.push(("UIRestorationIdentifier", nib.string(identifier)));
        }
    }
    swapper.push(("UIChildViewControllers", children_value));
    if nav.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
        swapper.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
    }
    swapper.push(("UIViewControllers", nib.array("NSMutableArray", vec![reference("root")])));
    swapper.push(("UINavigationBar", Value::New(Object::new("UINavigationBar", bar_values).named("bar"))));
    let class = runtime_class_name(nav, module, "UINavigationController")?;
    swapper.push(("UIClassName", nib.string(&class)));
    swapper.push(("UIOriginalClassName", nib.string("UINavigationController")));
    let swapper = Value::New(Object::new("UIClassSwapper", swapper).named("controller"));

    let mut objects = vec![reference("root storyboard"), reference("owner"), reference("responder"), reference("storyboard"), reference("controller"), reference("bar"), reference("root")];
    if explicit_item.is_some() {
        objects.push(reference("navigation item"));
    }
    let connections = vec![
        outlet(&mut nib, "storyboard", "controller", "storyboard"),
        outlet(&mut nib, "storyboard", "root", "root storyboard"),
        outlet(&mut nib, "sceneViewController", "owner", "controller"),
    ];
    let top = vec![root_storyboard, owner, responder, storyboard, swapper];
    Ok((name, nib::archive(root_object(&mut nib, top, objects, connections))))
}

pub fn compile(xml: &str, minimum_deployment_target: Option<&str>, module: Option<&str>, localizable: bool) -> Result<Compiled, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let platform = match root.attribute("type") {
        Some("com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB") => Platform::Ios,
        Some("com.apple.InterfaceBuilder.AppleTV.Storyboard") => Platform::Tvos,
        _ => return fail("not an iOS or tvOS storyboard"),
    };
    // Without a <device>, Interface Builder lays an iOS scene out on its
    // generic canvas below iOS 17, and at the view's frame from iOS 17 on;
    // Xcode's ibtool fails on a tvOS one.
    let device = children(root, "device").next();
    let has_device = device.is_some();
    let device_size = match device {
        Some(d) => device_size(platform, attr(d, "id")?, attr(d, "orientation")?)?,
        None => None,
    };
    let generic_canvas = if has_device {
        false
    } else if platform == Platform::Tvos {
        return fail("a tvOS storyboard without a device is not compiled by Xcode's ibtool");
    } else {
        let target = minimum_deployment_target.ok_or("a storyboard without a device needs --minimum-deployment-target for this ibtool")?;
        let major: u32 = target.split('.').next().and_then(|m| m.parse().ok()).ok_or_else(|| format!("not a deployment target: {target}"))?;
        major < 17
    };
    let color_matched = root.attribute("colorMatched") == Some("YES");
    let initial = root.attribute("initialViewController");
    let scenes = children(root, "scenes").next().ok_or("storyboard has no scenes")?;
    let scene_nodes: Vec<Node> = children(scenes, "scene").collect();
    // Navigation controllers, with their root view controllers' ids. One
    // that can be loaded (initial or identified) embeds its root
    // controller, whose view nib is then compiled even if nothing else
    // loads it.
    let mut navs = Vec::new();
    for (index, scene_node) in scene_nodes.iter().enumerate() {
        if let Some(nav) = scene_node.descendants().find(|d| d.has_tag_name("navigationController")) {
            let root_id = navigation_root(nav)?;
            let loadable = initial == Some(attr(nav, "id")?) || nav.attribute("storyboardIdentifier").is_some();
            navs.push((index, nav, root_id, loadable));
        }
    }
    // Each scene's files and Info.plist entry, by scene.
    let mut outputs: Vec<(usize, Vec<(String, Vec<u8>)>, Option<(String, bool, bool)>)> = Vec::new();
    let mut roots: HashMap<&str, RootController> = HashMap::new();
    for (index, scene_node) in scene_nodes.iter().enumerate() {
        let scene_node = *scene_node;
        only_children(scene_node, &["objects", "point"])?;
        let objects_node = children(scene_node, "objects").next().ok_or("scene without objects")?;
        only_children(objects_node, &["viewController", "navigationController", "placeholder", "exit"])?;
        // An exit nothing unwinds to is Interface Builder's alone.
        for exit in children(objects_node, "exit") {
            only_attributes(exit, &[("id", None), ("userLabel", Some("Exit")), ("sceneMemberID", Some("exit"))])?;
            let id = attr(exit, "id")?;
            if scene_node.document().descendants().any(|d| d != exit && d.attributes().any(|a| a.value() == id)) {
                return fail("an exit something unwinds to is not supported by this ibtool");
            }
        }
        if children(objects_node, "navigationController").next().is_some() {
            continue;
        }
        let vc = children(objects_node, "viewController").next().ok_or("scene without a view controller")?;
        only_attributes(vc, &[
            ("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None),
            ("storyboardIdentifier", None), ("useStoryboardIdentifierAsRestorationIdentifier", Some("YES")), ("title", None),
            ("automaticallyAdjustsScrollViewInsets", Some("NO")), ("userLabel", None), ("sceneMemberID", Some("viewController")),
        ])?;
        let class_name = runtime_class_name(vc, module, "UIViewController")?;
        only_children(vc, &["view", "layoutGuides", "connections", "navigationItem", "size"])?;
        if children(vc, "size").any(|s| s.attribute("key") != Some("freeformSize")) {
            return fail("sizes other than a freeform size inside <viewController> are not supported by this ibtool");
        }
        let vc_id = attr(vc, "id")?;
        let view = children(vc, "view").next().ok_or("view controller without a view")?;
        only_children(view, &["rect", "autoresizingMask", "viewLayoutGuide", "color", "subviews", "constraints"])?;
        only_attributes(view, &[("key", Some("view")), ("contentMode", Some("scaleToFill")), ("id", None)])?;
        let view_id = attr(view, "id")?;
        let root_name = view_name(view_id);

        let mut scene = Scene { vc_id, module, own_constraints: Vec::new(), constrained: constrained_views(view), subviews: Vec::new(), parent: None, actions: Vec::new(), guides: Vec::new() };
        for g in children(vc, "layoutGuides") {
            only_children(g, &["viewControllerLayoutGuide"])?;
            for guide in children(g, "viewControllerLayoutGuide") {
                let kind = match attr(guide, "type")? {
                    "top" => "top",
                    "bottom" => "bottom",
                    other => return fail(format!("layout guide type {other} is not supported by this ibtool")),
                };
                scene.guides.push((attr(guide, "id")?, kind));
            }
        }
        if scene.guides.iter().map(|g| g.1).collect::<Vec<_>>() != ["top", "bottom"] && !scene.guides.is_empty() {
            return fail("layout guides other than a top and a bottom one are not supported by this ibtool");
        }
        let safe_area = children(view, "viewLayoutGuide").find(|g| g.attribute("key") == Some("safeArea"));
        let has_safe_area = safe_area.is_some();
        if has_safe_area == !scene.guides.is_empty() {
            return fail("a view needs either a safe-area layout guide or legacy top and bottom layout guides for this ibtool");
        }

        // Outlets from the view controller into its view.
        let mut vc_outlets = Vec::new();
        for c in children(vc, "connections") {
            only_children(c, &["outlet"])?;
            for o in children(c, "outlet") {
                only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
                vc_outlets.push((attr(o, "property")?.to_string(), view_name(attr(o, "destination")?)));
            }
        }

        let mut nib = Nib::new(platform, color_matched, localizable);
        // A controller's freeform size (`<size key="freeformSize">`, the
        // simulated metrics' own size) replaces the device's or the generic
        // canvas's, but not the frame a device-less view from iOS 17 keeps.
        let freeform = match child(vc, "size", "freeformSize") {
            Some(s) => Some((number(attr(s, "width")?)?, number(attr(s, "height")?)?)),
            None => None,
        };
        // The generic canvas is size classes' 600x600; without them, the
        // default device's 393x852.
        let traits = root.attribute("useTraitCollections") == Some("YES");
        let f = match (generic_canvas, has_device, device_size, freeform) {
            (true, _, _, None) if traits => Frame { x: 0.0, y: 0.0, w: 600.0, h: 600.0 },
            (true, _, _, None) => Frame { x: 0.0, y: 0.0, w: 393.0, h: 852.0 },
            (true, _, _, Some((w, h))) if traits => Frame { x: 0.0, y: 0.0, w, h },
            (true, _, _, Some(_)) => return fail("a freeform size without trait collections is not supported by this ibtool"),
            (false, true, Some(_), Some((w, h))) => Frame { x: 0.0, y: 0.0, w, h },
            (false, true, Some((w, h)), None) => Frame { x: 0.0, y: 0.0, w, h },
            (false, true, None, Some(_)) => return fail("a freeform size with this device is not supported by this ibtool"),
            (false, _, _, _) => frame(view)?,
        };
        // Laid out at another size than its frame's, the root view resizes
        // its autoresizing subviews: only those whose masks keep their frames
        // (fixed left and top margins, size) are established.
        let document = frame(view)?;
        if (f.w, f.h) != (document.w, document.h) {
            for s in children(view, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())) {
                if s.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
                    continue;
                }
                let mask = match children(s, "autoresizingMask").next() {
                    Some(m) => autoresizing_mask(m)?,
                    None => 36,
                };
                if mask & (1 | 2 | 8 | 16) != 0 {
                    return fail("an autoresizing subview of a view laid out at another size than its frame is not supported by this ibtool");
                }
            }
        }
        let mask = match children(view, "autoresizingMask").next() {
            Some(m) => autoresizing_mask(m)?,
            None => 0,
        };
        let background = child(view, "color", "backgroundColor").ok_or("view without a background color")?;

        // The view's nib, in encoding order.
        let owner = proxy(&mut nib, "IBFilesOwner", "owner");
        let responder = proxy(&mut nib, "IBFirstResponder", "responder");
        let upstream_used = !vc_outlets.is_empty() || view.descendants().any(|d| d.has_tag_name("action"));
        let mut top = vec![owner, responder];
        let mut objects = vec![reference("owner"), reference("responder")];
        if upstream_used {
            top.push(proxy(&mut nib, "UpstreamPlaceholder-1", "upstream"));
            objects.push(reference("upstream"));
        }

        let mut values = Vec::new();
        geometry(&mut values, &f);
        let mut subview_values = Vec::new();
        for s in children(view, "subviews") {
            for c in s.children().filter(|c| c.is_element()) {
                subview_values.push(subview(&mut nib, &mut scene, c)?);
            }
        }
        let mut guide_constraints = Vec::new();
        let guide_kinds: Vec<&'static str> = scene.guides.iter().map(|g| g.1).collect();
        for kind in guide_kinds {
            let (guide, names) = legacy_guide(&mut nib, &root_name, kind);
            subview_values.push(guide);
            guide_constraints.extend(names);
        }
        if !subview_values.is_empty() {
            values.push(("UISubviews", nib.array("NSMutableArray", subview_values)));
        }
        values.push(("UIBackgroundColor", color(&mut nib, background)?));
        values.push(("UIDeepDrawRect", Value::Bool(true)));
        values.push(("UIOpaque", Value::Bool(true)));
        values.push(("UIAutoresizeSubviews", Value::Bool(true)));
        values.push(("UIAutoresizingMask", Value::Int(mask)));

        // The view's constraints: the guides' first, then the document's in
        // the order Xcode gives them, by first attribute.
        let safe_area_id = safe_area.map(|g| attr(g, "id")).transpose()?;
        let mut known: Vec<&str> = vec![view_id];
        known.extend(view.descendants().filter(|d| d.is_element() && d.parent().is_some_and(|p| p.has_tag_name("subviews"))).filter_map(|d| d.attribute("id")));
        let item = |id: &str| -> Result<Value, String> {
            if let Some((_, kind)) = scene.guides.iter().find(|g| g.0 == id) {
                return Ok(reference(format!("guide {kind}")));
            }
            if !known.contains(&id) {
                return fail(format!("constraint item {id} is not supported by this ibtool"));
            }
            Ok(reference(view_name(id)))
        };
        // A group's place: its item's frame origin by distance from the
        // root view's, then x, then id; the safe-area guide's is the origin.
        let direct: Vec<Node> = children(view, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
        let place = |id: &str| -> Result<(f64, f64), String> {
            if Some(id) == safe_area_id {
                return Ok((0.0, 0.0));
            }
            let v = direct.iter().find(|v| v.attribute("id") == Some(id)).ok_or_else(|| format!("constraints between the root view and a nested view ({id}) are not supported by this ibtool"))?;
            let f = subview_frame(*v)?;
            Ok((f.x * f.x + f.y * f.y, f.x))
        };
        let mut xml_constraints: Vec<((f64, f64), (&str, u8, (usize, usize)), Node, &str, Option<&str>)> = Vec::new();
        for cs in children(view, "constraints") {
            for c in children(cs, "constraint") {
                only_attributes(c, &[("firstItem", None), ("firstAttribute", None), ("secondItem", None), ("secondAttribute", None), ("multiplier", None), ("constant", None), ("id", None)])?;
                let first = c.attribute("firstItem").unwrap_or(view_id);
                let second = c.attribute("secondItem").unwrap_or(view_id);
                if c.attribute("secondItem").is_none() && c.attribute("secondAttribute").is_none() {
                    return fail("a root view constraint without a second item is not supported by this ibtool");
                }
                if first == second {
                    return fail("a constraint between an item and itself is not supported by this ibtool");
                }
                // Where a legacy guide's own constraints go isn't established;
                // one as the second item of a sibling constraint is.
                let legacy = |id: &str| scene.guides.iter().any(|g| g.0 == id);
                if legacy(first) || (legacy(second) && first == view_id) {
                    return fail("constraints from a legacy layout guide, or between one and the root view, are not supported by this ibtool");
                }
                for id in [first, second] {
                    if id != view_id && Some(id) != safe_area_id && !known.contains(&id) && !legacy(id) {
                        return fail(format!("constraints on {id} are not supported by this ibtool"));
                    }
                }
                let key = constraint_key(view_id, first, attr(c, "firstAttribute")?, second, attr(c, "secondAttribute")?)?;
                if xml_constraints.iter().any(|(_, k, ..)| *k == key) {
                    return fail("two constraints Xcode orders unstably (the same items and attributes) are not supported by this ibtool");
                }
                xml_constraints.push((place(key.0)?, key, c, first, Some(second)));
            }
        }
        xml_constraints.sort_by(|a, b| a.0 .0.total_cmp(&b.0 .0).then(a.0 .1.total_cmp(&b.0 .1)).then(a.1.cmp(&b.1)));
        let mut constraint_values: Vec<Value> = guide_constraints.iter().map(|n| reference(n.clone())).collect();
        let mut constraint_names = Vec::new();
        // The safe-area guide is archived where a constraint first uses it.
        let mut safe_area_used = false;
        let mut resolve = |nib: &mut Nib, id: &str| -> Result<Value, String> {
            if Some(id) != safe_area_id {
                return item(id);
            }
            if safe_area_used {
                return Ok(reference("safe area guide"));
            }
            safe_area_used = true;
            Ok(safe_area_guide(nib, &root_name))
        };
        for (_, _, c, first, second) in &xml_constraints {
            let first = resolve(&mut nib, first)?;
            let second = match second {
                Some(s) => Some((resolve(&mut nib, s)?, layout_attribute(attr(*c, "secondAttribute")?)?)),
                None => None,
            };
            let constant = c.attribute("constant").map(number).transpose()?.unwrap_or(0.0);
            let name = nib.fresh();
            constraint_names.push(name.clone());
            match multiplied(constraint(first, layout_attribute(attr(*c, "firstAttribute")?)?, second, constant, None), multiplier(*c)?) {
                Value::New(o) => constraint_values.push(Value::New(o.named(name))),
                _ => unreachable!(),
            }
        }
        if !constraint_values.is_empty() {
            values.push(("UIViewAutolayoutConstraints", nib.array("NSMutableArray", constraint_values)));
        }
        if safe_area_used {
            values.push(("UIViewLayoutGuides", nib.array("NSMutableArray", vec![reference("safe area guide")])));
        }
        values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
        nib.large_content(&mut values);
        top.push(Value::New(Object::new("UIView", values).named(root_name.clone())));

        objects.push(reference(root_name.clone()));
        // The safe-area guide follows the root view.
        if has_safe_area {
            let identifier = nib.string("");
            objects.push(Value::New(Object::new("UILayoutGuide", vec![
                ("UILayoutGuideOwningView", Value::Nil),
                ("UILayoutGuideIdentifier", identifier),
                ("UILayoutGuideOwningViewIsLocked", Value::Bool(false)),
                ("UILayoutGuideShouldBeArchived", Value::Bool(false)),
                ("UILayoutGuideAllowsNegativeDimensions", Value::Bool(false)),
            ]).named("safe area")));
        }
        objects.extend(constraint_names.iter().map(|n| reference(n.clone())));
        objects.extend(scene.guides.iter().map(|g| reference(format!("guide {}", g.1))));
        // Then breadth first: each view in turn, from a queue, lists the
        // constraints it holds and then its subviews, which join the queue.
        let mut queue: std::collections::VecDeque<Option<&String>> = [None].into();
        while let Some(v) = queue.pop_front() {
            if let Some(v) = v {
                objects.extend(scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| reference(c.1.clone())));
            }
            for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == v) {
                objects.push(reference(s.clone()));
                queue.push_back(Some(s));
            }
        }

        // Outlets, `view` among them, then actions, each in byte order of
        // their names; actions with the same selector in document order.
        let mut connections = Vec::new();
        let mut outlets: Vec<(&str, &str, &str)> = vc_outlets.iter().map(|(l, d)| (l.as_str(), "upstream", d.as_str())).collect();
        outlets.push(("view", "owner", &root_name));
        outlets.sort_by(|a, b| a.0.cmp(b.0));
        for (label, source, destination) in outlets {
            connections.push(outlet(&mut nib, label, source, destination));
        }
        scene.actions.sort_by(|a, b| a.0.cmp(&b.0));
        for (selector, source, mask) in &scene.actions {
            let label = nib.string(selector);
            connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
                ("UILabel", label),
                ("UISource", reference(source.clone())),
                ("UIDestination", reference("upstream")),
                ("UIEventMask", Value::Int(*mask)),
            ])));
        }
        let view_root = root_object(&mut nib, top, objects, connections);

        // The view controller's nib: the scene's placeholders and a class
        // swapper standing in for the controller, which loads the view nib.
        if let Some(identifier) = vc.attribute("storyboardIdentifier") {
            // Xcode can't write a name with a slash, and writes a non-ASCII
            // one decomposed.
            if identifier.is_empty() || identifier.contains('/') || !identifier.is_ascii() {
                return fail(format!("storyboard identifier {identifier:?} is not supported by this ibtool"));
            }
        }
        // A controller with a storyboard identifier is loaded by it.
        let vc_nib_name = vc.attribute("storyboardIdentifier").map_or_else(|| format!("UIViewController-{vc_id}"), str::to_string);
        let view_nib_name = format!("{vc_id}-view-{view_id}");
        let mut vnib = Nib::new(platform, color_matched, localizable);
        let owner = proxy(&mut vnib, "IBFilesOwner", "owner");
        let responder = proxy(&mut vnib, "IBFirstResponder", "responder");
        let storyboard = proxy(&mut vnib, "UIStoryboardPlaceholder", "storyboard");
        let external = if upstream_used { vec![vnib.string("UpstreamPlaceholder-1"), reference("controller")] } else { Vec::new() };
        let external = vnib.array("NSDictionary", external);
        let nib_name = vnib.string(&view_nib_name);
        let mut swapper = Vec::new();
        if let Some(title) = vc.attribute("title") {
            swapper.push(("UITitle", vnib.text(vc_id, "title", title)));
        }
        if let Some(item) = child(vc, "navigationItem", "navigationItem") {
            swapper.push(("UINavigationItem", navigation_item(&mut vnib, item)?));
        }
        swapper.push(("UIExternalObjectsTableForViewLoading", external));
        swapper.push(("UINibName", nib_name));
        if let Some(identifier) = vc.attribute("storyboardIdentifier") {
            swapper.push(("UIStoryboardIdentifier", vnib.string(identifier)));
            if vc.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
                swapper.push(("UIRestorationIdentifier", vnib.string(identifier)));
            }
        } else if vc.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            return fail("useStoryboardIdentifierAsRestorationIdentifier without a storyboard identifier is not supported by this ibtool");
        }
        if vc.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
            swapper.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
        }
        swapper.push(("UIClassName", vnib.string(&class_name)));
        swapper.push(("UIOriginalClassName", vnib.string("UIViewController")));
        let swapper = Value::New(Object::new("UIClassSwapper", swapper).named("controller"));
        let connections = vec![outlet(&mut vnib, "storyboard", "controller", "storyboard"), outlet(&mut vnib, "sceneViewController", "owner", "controller")];
        let vc_root = root_object(
            &mut vnib,
            vec![owner, responder, storyboard, swapper],
            {
                let mut objects = vec![reference("owner"), reference("responder"), reference("storyboard"), reference("controller")];
                if child(vc, "navigationItem", "navigationItem").is_some() {
                    objects.push(reference("navigation item"));
                }
                objects
            },
            connections,
        );
        // A scene neither initial nor identified can't be loaded, and
        // Xcode leaves it out, unless a navigation controller embeds it.
        let is_initial = initial == Some(vc_id);
        let identified = vc.attribute("storyboardIdentifier").is_some();
        let embedded = navs.iter().any(|n| n.2 == vc_id && n.3);
        if embedded {
            roots.insert(vc_id, RootController { vc, loadable: is_initial || identified, view_nib_name: view_nib_name.clone(), class_name: class_name.clone(), upstream_used });
        }
        if !is_initial && !identified && !embedded {
            continue;
        }
        let mut scene_files = vec![(format!("{view_nib_name}.nib"), nib::archive(view_root))];
        let mut entry = None;
        if is_initial || identified {
            scene_files.push((format!("{vc_nib_name}.nib"), nib::archive(vc_root)));
            entry = Some((vc_nib_name.clone(), is_initial, identified));
        }
        outputs.push((index, scene_files, entry));
    }
    for &(index, nav, root_id, loadable) in &navs {
        if !loadable {
            return fail("a navigation controller neither initial nor identified is not supported by this ibtool");
        }
        let root = roots.get(root_id).ok_or("a navigation controller's root view controller is not in a supported scene")?;
        let (name, bytes) = navigation_controller_nib(platform, color_matched, localizable, module, nav, root)?;
        let entry = (name.clone(), initial == Some(attr(nav, "id")?), nav.attribute("storyboardIdentifier").is_some());
        outputs.push((index, vec![(format!("{name}.nib"), bytes)], Some(entry)));
    }
    outputs.sort_by_key(|o| o.0);
    let mut files = Vec::new();
    let mut identifiers = Vec::new();
    for (_, scene_files, entry) in outputs {
        files.extend(scene_files);
        identifiers.extend(entry);
    }
    // Xcode renames nibs whose names differ only in case and keeps one of
    // equal identifiers.
    let mut names: Vec<String> = files.iter().map(|(n, _)| n.to_lowercase()).collect();
    names.sort();
    if names.windows(2).any(|w| w[0] == w[1]) {
        return fail("storyboard identifiers that are equal ignoring case are not supported by this ibtool");
    }
    let mut info = Vec::new();
    // The entry point is the same object as its identifier's key when the
    // document names it; a generated name is made twice.
    let mut shared_entry = None;
    if initial.is_some() {
        let entry = identifiers.iter().find(|i| i.1).ok_or("the initial view controller is not in a supported scene")?;
        info.push(("UIStoryboardDesignatedEntryPointIdentifier", cf::Value::string(entry.0.clone())));
        if entry.2 {
            shared_entry = Some(entry.0.clone());
        }
    }
    info.push(("UIStoryboardVersion", cf::Value::Integer(1)));
    info.push(("UIViewControllerIdentifiersToNibNames", cf::Value::dict(identifiers.iter().map(|(n, ..)| (n.clone(), cf::Value::string(n.clone()))))));
    let info = cf::Value::dict(info);
    let share = |_: &cf::Value, at: &cf::binary::Position| {
        use cf::binary::Step;
        let Some(entry) = &shared_entry else { return false };
        match at {
            [Step::Value(k)] => k == "UIStoryboardDesignatedEntryPointIdentifier",
            [Step::Value(k), Step::Key(id)] => k == "UIViewControllerIdentifiersToNibNames" && id == entry,
            _ => false,
        }
    };
    files.push(("Info.plist".into(), cf::binary::to_bytes_with(&info, &share, cf::binary::Dictionaries::Grown)));
    Ok(Compiled { files })
}
