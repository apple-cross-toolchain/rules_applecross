//! WatchKit storyboards, compiled to the `Interface.plist` a watch app
//! carries; see docs/clean-room/ibtool.md.

use cf::Value;
use roxmltree::{Document, Node};

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

fn attr<'a>(n: Node<'a, '_>, name: &str) -> Result<&'a str, String> {
    n.attribute(name).ok_or_else(|| format!("<{}> has no {name} attribute", n.tag_name().name()))
}

fn elements<'a, 'i>(n: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    n.children().filter(|c| c.is_element())
}

fn number(s: &str) -> Result<f64, String> {
    s.parse::<f64>().map_err(|_| format!("not a number: {s}"))
}

/// A dictionary whose keys are inserted in `order`, which Xcode's own
/// insertion order reproduces for these key sets (the table's enumeration
/// order depends on it).
fn dict(order: &[&str], mut entries: Vec<(&'static str, Value)>) -> Result<Value, String> {
    let mut out = Vec::new();
    for key in order {
        if let Some(i) = entries.iter().position(|(k, _)| k == key) {
            let (k, v) = entries.remove(i);
            out.push((k.to_string(), v));
        }
    }
    if !entries.is_empty() {
        return fail(format!("{} is not supported by this ibtool here", entries[0].0));
    }
    Ok(Value::Dictionary(out))
}

/// Checks a set of keys against those verified for an item.
fn verified(keys: &[&str], sets: &[&[&str]], what: &str) -> Result<(), String> {
    let mut have: Vec<&str> = keys.to_vec();
    have.sort();
    for set in sets {
        let mut s = set.to_vec();
        s.sort();
        if s == have {
            return Ok(());
        }
    }
    fail(format!("a {what} with {} is not supported by this ibtool", keys.join(", ")))
}

const LABEL_ORDER: [&str; 9] = ["fontSize2", "color", "fontSize", "text", "property", "type", "fontScale", "numberOfLines", "alignment"];
const LABEL_SETS: [&[&str]; 2] = [
    &["type", "alignment", "text", "numberOfLines", "fontScale", "property"],
    &["type", "alignment", "text", "numberOfLines", "fontScale", "property", "color", "fontSize", "fontSize2"],
];
const GROUP_ORDER: [&str; 6] = ["type", "width", "alignment", "verticalAlignment", "layout", "items"];

/// WatchKit's names for the text styles verified here.
fn text_style(style: &str) -> Result<&'static str, String> {
    match style {
        "UICTFontTextStyleCaption2" => Ok("caption2"),
        other => fail(format!("the {other} text style is not supported by this ibtool here")),
    }
}

fn item(n: Node, outlets: &[(String, String)]) -> Result<Value, String> {
    let id = attr(n, "id")?;
    let property = outlets.iter().find(|(_, d)| d == id).map(|(p, _)| Value::string(p.clone()));
    match n.tag_name().name() {
        "group" => {
            let mut entries = vec![("type", Value::string("group"))];
            for a in n.attributes() {
                match a.name() {
                    "width" => entries.push(("width", Value::Real(number(a.value())?))),
                    "alignment" => entries.push(("alignment", Value::string(a.value()))),
                    "verticalAlignment" => entries.push(("verticalAlignment", Value::string(a.value()))),
                    "layout" => entries.push(("layout", Value::string(a.value()))),
                    "id" => {}
                    other => return fail(format!("the {other} attribute of a group is not supported by this ibtool")),
                }
            }
            let mut items = Vec::new();
            for c in elements(n) {
                if !c.has_tag_name("items") {
                    return fail(format!("<{}> inside a group is not supported by this ibtool", c.tag_name().name()));
                }
                for i in elements(c) {
                    items.push(item(i, outlets)?);
                }
            }
            entries.push(("items", Value::Array(items)));
            if property.is_some() {
                return fail("outlets to groups are not supported by this ibtool");
            }
            let keys: Vec<&str> = entries.iter().map(|e| e.0).collect();
            verified(&keys, &[&GROUP_ORDER], "group")?;
            dict(&GROUP_ORDER, entries)
        }
        "label" => {
            let mut entries = vec![("type", Value::string("label"))];
            for a in n.attributes() {
                match a.name() {
                    "alignment" => entries.push(("alignment", Value::string(a.value()))),
                    "text" => entries.push(("text", Value::string(a.value()))),
                    "numberOfLines" => entries.push(("numberOfLines", Value::Integer(a.value().parse().map_err(|_| format!("not a number: {}", a.value()))?))),
                    "id" => {}
                    other => return fail(format!("the {other} attribute of a label is not supported by this ibtool")),
                }
            }
            for c in elements(n) {
                match (c.tag_name().name(), c.attribute("key")) {
                    ("color", Some("textColor")) => {
                        // A calibrated white of 2/3 is sRGB 0xB8 as WatchKit
                        // records it; other colours are not established.
                        if c.attribute("white") != Some("0.66666666666666663") || c.attribute("alpha") != Some("1") || c.attribute("colorSpace") != Some("calibratedWhite") {
                            return fail("label colours other than a calibrated white of 2/3 are not supported by this ibtool");
                        }
                        entries.push(("color", Value::string("B8")));
                    }
                    ("fontDescription", Some("font")) => {
                        let style = text_style(attr(c, "style")?)?;
                        entries.push(("fontSize", Value::string(style)));
                        entries.push(("fontSize2", Value::string(style)));
                    }
                    (other, _) => return fail(format!("<{other}> inside a label is not supported by this ibtool")),
                }
            }
            entries.push(("fontScale", Value::Real(1.0)));
            if let Some(p) = property {
                entries.push(("property", p));
            }
            let keys: Vec<&str> = entries.iter().map(|e| e.0).collect();
            verified(&keys, &LABEL_SETS, "label")?;
            dict(&LABEL_ORDER, entries)
        }
        other => fail(format!("<{other}> is not supported by this ibtool")),
    }
}

/// Strings Xcode writes once for each use; it shares every other equal
/// string and number.
fn shared(v: &Value, _: &cf::binary::Position) -> bool {
    !matches!(v, Value::String(s) if s.starts_with("controller-"))
}

pub fn compile(xml: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let initial = attr(root, "initialViewController")?;
    let scenes = elements(root).find(|c| c.has_tag_name("scenes")).ok_or("storyboard has no scenes")?;
    let mut controllers = Vec::new();
    for scene in elements(scenes) {
        let objects = elements(scene).find(|c| c.has_tag_name("objects")).ok_or("scene without objects")?;
        for controller in elements(objects) {
            if !controller.has_tag_name("controller") {
                return fail(format!("<{}> in a WatchKit scene is not supported by this ibtool", controller.tag_name().name()));
            }
            let id = attr(controller, "id")?;
            let mut outlets = Vec::new();
            let mut items = Vec::new();
            for c in elements(controller) {
                match c.tag_name().name() {
                    "items" => items = elements(c).collect(),
                    "connections" => {
                        for o in elements(c) {
                            if !o.has_tag_name("outlet") {
                                return fail("connections other than outlets are not supported by this ibtool");
                            }
                            outlets.push((attr(o, "property")?.to_string(), attr(o, "destination")?.to_string()));
                        }
                    }
                    other => return fail(format!("<{other}> inside a controller is not supported by this ibtool")),
                }
            }
            let items = items.into_iter().map(|i| item(i, &outlets)).collect::<Result<Vec<_>, _>>()?;
            let class = controller.attribute("customClass").ok_or("controllers without a custom class are not supported by this ibtool")?;
            let entry = dict(&["controllerClass", "items"], vec![("controllerClass", Value::string(class)), ("items", Value::Array(items))])?;
            controllers.push((format!("controller-{id}"), entry));
        }
    }
    let plist = Value::dict([
        ("controllers", Value::Dictionary(controllers)),
        ("root", Value::string(format!("controller-{initial}"))),
    ]);
    Ok(vec![("Interface.plist".into(), cf::binary::to_bytes_with(&plist, &shared, cf::binary::Dictionaries::Grown))])
}

/// Linking writes a compiled interface out of its .storyboardc again as read
/// back: each dictionary created whole, from its keys in the order written.
pub fn link(compiled: &[u8]) -> Result<Vec<u8>, String> {
    let plist = Value::parse(compiled)?;
    Ok(cf::binary::to_bytes_with(&plist, &shared, cf::binary::Dictionaries::Created))
}
