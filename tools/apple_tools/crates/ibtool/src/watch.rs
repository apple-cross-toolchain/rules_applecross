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

/// A notification's alert label is `alert` rather than an outlet's
/// `property`, inserted first (seven probes, no key set's order in doubt).
const LABEL_ORDER: [&str; 10] = ["alert", "fontSize2", "color", "fontSize", "text", "property", "type", "fontScale", "alignment", "numberOfLines"];
const GROUP_ORDER: [&str; 9] = ["height", "layout", "type", "spacing", "property", "verticalAlignment", "width", "alignment", "items"];
/// The orders Xcode's groups, buttons and images take, each reproducing
/// every key set of their probes (about 50 groups', 20 buttons', 20
/// images'), the last ones chosen where the earlier left two orders
/// possible, so that no key set's order is in doubt.
const BUTTON_ORDER: [&str; 9] = ["property", "action", "type", "title", "verticalAlignment", "enabled", "height", "width", "alignment"];
const IMAGE_ORDER: [&str; 9] = ["property", "type", "image", "verticalAlignment", "duration", "contentMode", "height", "width", "alignment"];

/// What compiling a WatchKit item needs beside the item: its controller's
/// outlets, whether a notification's alert label is an outlet, and
/// whether text is localizable (`Base.lproj`).
struct Context<'o> {
    outlets: &'o [(String, String)],
    localizable: bool,
}

/// Text an item shows: in `Base.lproj`, unless empty, its development text
/// and localization key `<id>.<key>` (probes, rules_apple's Buttons
/// example).
fn text(cx: &Context, id: &str, key: &str, value: &str) -> Value {
    if !cx.localizable || value.is_empty() {
        return Value::string(value);
    }
    Value::Dictionary(vec![
        ("fallbackString".to_string(), Value::string(value)),
        ("localizationKey".to_string(), Value::string(format!("{id}.{key}"))),
    ])
}

/// An item's horizontal alignment: left out when left, 3 without one, as
/// a label's (probes).
fn alignment(n: Node, entries: &mut Vec<(&'static str, Value)>) {
    match n.attribute("alignment") {
        Some("left") => {}
        Some(a) => entries.push(("alignment", Value::string(a))),
        None => entries.push(("alignment", Value::Integer(3))),
    }
}

/// An outlet to an item: a notification's alert label is `alert`, others
/// `property`.
fn outlet(cx: &Context, id: &str, entries: &mut Vec<(&'static str, Value)>) {
    for (p, d) in cx.outlets {
        if d == id {
            if p == "notificationAlertLabel" {
                entries.push(("alert", Value::Boolean(true)));
            } else {
                entries.push(("property", Value::string(p.clone())));
            }
        }
    }
}

/// WatchKit's names for the text styles verified here.
fn text_style(style: &str) -> Result<&'static str, String> {
    match style {
        "UICTFontTextStyleCaption2" => Ok("caption2"),
        other => fail(format!("the {other} text style is not supported by this ibtool here")),
    }
}

fn item(n: Node, cx: &Context) -> Result<Value, String> {
    let id = attr(n, "id")?;
    match n.tag_name().name() {
        "group" => {
            let mut entries = vec![("type", Value::string("group"))];
            for a in n.attributes() {
                match a.name() {
                    "width" | "height" | "spacing" => entries.push((
                        match a.name() {
                            "width" => "width",
                            "height" => "height",
                            _ => "spacing",
                        },
                        Value::Real(number(a.value())?),
                    )),
                    "verticalAlignment" => entries.push(("verticalAlignment", Value::string(a.value()))),
                    "layout" => entries.push(("layout", Value::string(a.value()))),
                    "alignment" | "id" => {}
                    other => return fail(format!("the {other} attribute of a group is not supported by this ibtool")),
                }
            }
            alignment(n, &mut entries);
            let mut items = Vec::new();
            for c in elements(n) {
                if !c.has_tag_name("items") {
                    return fail(format!("<{}> inside a group is not supported by this ibtool", c.tag_name().name()));
                }
                for i in elements(c) {
                    items.push(item(i, cx)?);
                }
            }
            entries.push(("items", Value::Array(items)));
            outlet(cx, id, &mut entries);
            dict(&GROUP_ORDER, entries)
        }
        "label" => {
            let mut entries = vec![("type", Value::string("label"))];
            for a in n.attributes() {
                match a.name() {
                    // An explicit left is left out (probes).
                    "alignment" if a.value() == "left" => {}
                    "alignment" => entries.push(("alignment", Value::string(a.value()))),
                    "text" => entries.push(("text", text(cx, id, "text", a.value()))),
                    // One line, the default, is left out (probes).
                    "numberOfLines" if a.value() == "1" => {}
                    "numberOfLines" => entries.push(("numberOfLines", Value::Integer(a.value().parse().map_err(|_| format!("not a number: {}", a.value()))?))),
                    "id" => {}
                    other => return fail(format!("the {other} attribute of a label is not supported by this ibtool")),
                }
            }
            // A label without an alignment is recorded as 3, an integer
            // (40 probes).
            if n.attribute("alignment").is_none() {
                entries.push(("alignment", Value::Integer(3)));
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
            outlet(cx, id, &mut entries);
            dict(&LABEL_ORDER, entries)
        }
        // A button (about 20 probes): its title, size, alignments, enabled
        // state and action, its connections only an action.
        "button" => {
            let mut entries = vec![("type", Value::string("button"))];
            for a in n.attributes() {
                match a.name() {
                    "title" => entries.push(("title", text(cx, id, "title", a.value()))),
                    "width" | "height" => entries.push((if a.name() == "width" { "width" } else { "height" }, Value::Real(number(a.value())?))),
                    "verticalAlignment" => entries.push(("verticalAlignment", Value::string(a.value()))),
                    "enabled" if a.value() == "NO" => entries.push(("enabled", Value::Boolean(false))),
                    "alignment" | "id" => {}
                    other => return fail(format!("the {other} attribute of a button is not supported by this ibtool")),
                }
            }
            alignment(n, &mut entries);
            for c in elements(n) {
                if !c.has_tag_name("connections") {
                    return fail(format!("<{}> inside a button is not supported by this ibtool", c.tag_name().name()));
                }
                for k in elements(c) {
                    if !k.has_tag_name("action") {
                        return fail("a button's connections other than an action are not supported by this ibtool");
                    }
                    entries.push(("action", Value::string(attr(k, "selector")?)));
                }
            }
            outlet(cx, id, &mut entries);
            dict(&BUTTON_ORDER, entries)
        }
        // An image (about 20 probes): its image, size, alignments and an
        // aspect-fit content mode (1), and `duration` false.
        "imageView" => {
            let mut entries = vec![("type", Value::string("image")), ("duration", Value::Boolean(false))];
            for a in n.attributes() {
                match a.name() {
                    "image" => entries.push(("image", Value::string(a.value()))),
                    "width" | "height" => entries.push((if a.name() == "width" { "width" } else { "height" }, Value::Real(number(a.value())?))),
                    "verticalAlignment" => entries.push(("verticalAlignment", Value::string(a.value()))),
                    "contentMode" if a.value() == "scaleAspectFit" => entries.push(("contentMode", Value::Integer(1))),
                    "alignment" | "id" => {}
                    other => return fail(format!("the {other} attribute of an image is not supported by this ibtool")),
                }
            }
            alignment(n, &mut entries);
            if let Some(c) = elements(n).next() {
                return fail(format!("<{}> inside an image is not supported by this ibtool", c.tag_name().name()));
            }
            outlet(cx, id, &mut entries);
            dict(&IMAGE_ORDER, entries)
        }
        other => fail(format!("<{other}> is not supported by this ibtool")),
    }
}

/// Whether ibtool's string is a tagged pointer (objc4's NSTaggedPointerString):
/// up to 7 ASCII characters, 8 or 9 from its six-bit alphabet, 10 or 11 from
/// the five-bit one. Equal tagged strings are one object.
fn tagged(s: &str) -> bool {
    const SIX: &str = "eilotrm.apdnsIc ufkMShjTRxgC4013bDNvwyUL2O856P-B79AFKEWV_zGJ/HYX";
    let n = s.chars().count();
    match n {
        0..=7 => s.is_ascii(),
        8 | 9 => s.chars().all(|c| SIX.contains(c)),
        10 | 11 => s.chars().all(|c| SIX[..32].contains(c)),
        _ => false,
    }
}

/// Where Xcode shares an equal string or number, which its writer does by
/// identity (about 15 probes, the corpus's six storyboards):
/// - a controller's identifier, one object as its key and in the root; its
///   generated `controller-<id>` name, formatted for each, never;
/// - keys, and the type `image`, constants;
/// - other strings (from the document, or the other types) when they are
///   tagged pointers, never otherwise;
/// - numbers and booleans throughout.
fn scope(v: &Value, at: &cf::binary::Position) -> Option<String> {
    use cf::binary::Step;
    let Value::String(s) = v else { return Some(String::new()) };
    let name = matches!(at, [Step::Value(c), Step::Key(_)] if c == "controllers") || matches!(at.first(), Some(Step::Value(r)) if r == "root");
    if name {
        return if s.starts_with("controller-") {
            None
        } else if tagged(s) {
            Some("tagged".into())
        } else {
            Some("identifier".into())
        };
    }
    let category = matches!(at, [Step::Value(c), Step::Key(_)] if c == "categories");
    match at.last() {
        Some(Step::Key(_)) if !category => Some("constant".into()),
        Some(Step::Value(k)) if k == "type" && s == "image" => Some("constant".into()),
        _ => tagged(s).then(|| "tagged".into()),
    }
}

/// A controller's items, its outlets and actions to them taken in, and its
/// own connections: outlets, and the relationship segues `kinds` allows
/// (relationship, destination).
fn controller_items(controller: Node, cx_outlets: &mut Vec<(String, String)>, segues: &mut Vec<(String, String)>, localizable: bool, kinds: &[&str]) -> Result<Vec<Value>, String> {
    let mut items = Vec::new();
    for c in elements(controller) {
        match c.tag_name().name() {
            "items" => items = elements(c).collect(),
            "connections" => {
                for o in elements(c) {
                    match o.tag_name().name() {
                        "outlet" => cx_outlets.push((attr(o, "property")?.to_string(), attr(o, "destination")?.to_string())),
                        "segue" if o.attribute("kind") == Some("relationship") && kinds.contains(&attr(o, "relationship")?) => {
                            segues.push((attr(o, "relationship")?.to_string(), attr(o, "destination")?.to_string()));
                        }
                        _ => return fail("connections other than outlets are not supported by this ibtool"),
                    }
                }
            }
            "notificationCategory" if kinds.contains(&"dynamicNotificationInterface") => {}
            other => return fail(format!("<{other}> inside a controller is not supported by this ibtool")),
        }
    }
    let cx = Context { outlets: cx_outlets, localizable };
    items.into_iter().map(|i| item(i, &cx)).collect()
}

pub fn compile(xml: &str, module: Option<&str>, localizable: bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let initial = attr(root, "initialViewController")?;
    let scenes = elements(root).find(|c| c.has_tag_name("scenes")).ok_or("storyboard has no scenes")?;
    let members: Vec<Node> = elements(scenes)
        .map(|scene| elements(scene).find(|c| c.has_tag_name("objects")).ok_or("scene without objects"))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(elements)
        .collect();
    // The controllers notifications show dynamically are listed with their
    // notification, not among the controllers (probes).
    let dynamic: Vec<&str> = doc
        .descendants()
        .filter(|d| d.has_tag_name("segue") && d.attribute("relationship") == Some("dynamicNotificationInterface"))
        .filter_map(|d| d.attribute("destination"))
        .collect();
    // A controller is named by its identifier, else `controller-<id>`
    // (rules_apple's Buttons example, a corpus storyboard).
    let name = |c: Node| -> Result<String, String> { Ok(c.attribute("identifier").map(str::to_string).unwrap_or(format!("controller-{}", attr(c, "id")?))) };
    let class = |c: Node| -> Result<String, String> {
        if c.attribute("customClass").is_none() {
            return fail("controllers without a custom class are not supported by this ibtool");
        }
        // Its class as the runtime names it: a Swift class of a module
        // (the target's, given by --module) mangled (a corpus storyboard).
        crate::storyboard::runtime_class_name(c, module, "")
    };
    let mut controllers = Vec::new();
    let mut pages: Vec<(String, String)> = Vec::new();
    let mut categories = Vec::new();
    for controller in &members {
        let id = attr(*controller, "id")?;
        match controller.tag_name().name() {
            // A SwiftUI hosting controller is listed as a controller of its
            // class (a corpus storyboard).
            // A controller nothing loads (not initial, unnamed, no page
            // leads to it) is left out (probes).
            "controller" | "hostingController" if !dynamic.contains(&id) && id != initial && controller.attribute("identifier").is_none() && !doc.descendants().any(|d| d.has_tag_name("segue") && d.attribute("relationship") == Some("nextPage") && d.attribute("destination") == Some(id)) => {
                if elements(*controller).next().is_some() {
                    return fail("a controller nothing loads, with items or connections, is not supported by this ibtool");
                }
            }
            "controller" | "hostingController" if !dynamic.contains(&id) => {
                let mut outlets = Vec::new();
                let mut segues = Vec::new();
                let items = controller_items(*controller, &mut outlets, &mut segues, localizable, &["nextPage"])?;
                let mut values = vec![("controllerClass", Value::string(class(*controller)?))];
                // A controller without items has none listed (a probe).
                if !items.is_empty() {
                    values.push(("items", Value::Array(items)));
                }
                controllers.push((name(*controller)?, dict(&["controllerClass", "items"], values)?));
                for (_, destination) in segues {
                    pages.push((id.to_string(), destination));
                }
            }
            "controller" => {
                if controller.attribute("identifier").is_some() || elements(*controller).next().is_some() {
                    return fail("a dynamic notification controller with an identifier, items or connections is not supported by this ibtool");
                }
            }
            // A static notification: its category's static items (and class),
            // and its dynamic controller's class (probes, rules_apple's
            // Buttons example). Without a category Xcode writes nothing.
            "notificationController" => {
                let category = elements(*controller)
                    .find(|c| c.has_tag_name("notificationCategory"))
                    .ok_or("a notification controller without a category is not supported by this ibtool")?;
                if category.attributes().any(|a| !["key", "identifier", "id"].contains(&a.name())) || elements(category).next().is_some() {
                    return fail("a notification category other than an identifier is not supported by this ibtool");
                }
                if controller.attributes().any(|a| !["id", "customClass", "customModule", "customModuleProvider"].contains(&a.name())) {
                    return fail("a notification controller's attributes other than its class are not supported by this ibtool");
                }
                let mut outlets = Vec::new();
                let mut segues = Vec::new();
                let items = controller_items(*controller, &mut outlets, &mut segues, localizable, &["dynamicNotificationInterface"])?;
                let mut static_values = Vec::new();
                if !items.is_empty() {
                    static_values.push(("items", Value::Array(items)));
                }
                if controller.attribute("customClass").is_some() {
                    static_values.push(("controllerClass", Value::string(class(*controller)?)));
                }
                let mut values = Vec::new();
                if let [(_, destination)] = segues.as_slice() {
                    let target = members.iter().find(|m| m.attribute("id") == Some(destination.as_str())).ok_or("a dynamic notification's controller")?;
                    values.push(("dynamic", Value::dict([("controllerClass", Value::string(class(*target)?))])));
                } else if !segues.is_empty() {
                    return fail("a notification with more than one dynamic interface is not supported by this ibtool");
                }
                values.push(("static", dict(&["items", "controllerClass"], static_values)?));
                categories.push((attr(category, "identifier")?.to_string(), dict(&["dynamic", "static"], values)?));
            }
            other => return fail(format!("<{other}> in a WatchKit scene is not supported by this ibtool")),
        }
    }
    let initial_node = members.iter().find(|m| m.attribute("id") == Some(initial)).ok_or("the initial controller")?;
    // Pages: the root is the chain of nextPage segues from the initial
    // controller (a corpus storyboard), which must hold them all.
    let root = if pages.is_empty() {
        Value::string(name(*initial_node)?)
    } else {
        let mut chain = vec![name(*initial_node)?];
        let mut at = initial.to_string();
        let mut seen = 0;
        while let Some((_, next)) = pages.iter().find(|(from, _)| *from == at) {
            let node = members.iter().find(|m| m.attribute("id") == Some(next.as_str())).ok_or("a page's controller")?;
            chain.push(name(*node)?);
            at = next.clone();
            seen += 1;
            if seen > pages.len() {
                return fail("pages in a cycle are not supported by this ibtool");
            }
        }
        if seen != pages.len() {
            return fail("pages other than one chain from the initial controller are not supported by this ibtool");
        }
        Value::Array(chain.into_iter().map(Value::string).collect())
    };
    let plist = Value::dict([("controllers", Value::Dictionary(controllers)), ("root", root)]);
    let mut out = vec![("Interface.plist".into(), cf::binary::to_bytes_scoped(&plist, &scope, cf::binary::Dictionaries::Grown))];
    if !categories.is_empty() {
        let plist = Value::dict([("categories", Value::Dictionary(categories))]);
        out.push(("Interface-notification.plist".into(), cf::binary::to_bytes_scoped(&plist, &scope, cf::binary::Dictionaries::Grown)));
    }
    Ok(out)
}

/// Linking writes a compiled interface out of its .storyboardc again as read
/// back: each dictionary created whole, from its keys in the order written.
pub fn link(compiled: &[u8]) -> Result<Vec<u8>, String> {
    let plist = Value::parse(compiled)?;
    // Read back, its objects keep the identities compiling gave them, but
    // for constants that are read back as tagged strings (a probe's type
    // `image` and an outlet `image`).
    let link_scope = |v: &Value, at: &cf::binary::Position| match (scope(v, at), v) {
        (Some(s), Value::String(text)) if s == "constant" && tagged(text) => Some("tagged".to_string()),
        (other, _) => other,
    };
    Ok(cf::binary::to_bytes_scoped(&plist, &link_scope, cf::binary::Dictionaries::Created))
}
