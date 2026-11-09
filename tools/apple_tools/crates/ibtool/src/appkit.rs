//! AppKit xibs (macOS, `com.apple.InterfaceBuilder3.Cocoa.XIB`), compiled
//! to the nibs Xcode's ibtool makes of them: see docs/clean-room/ibtool.md
//! (AppKit).
//!
//! A nib's root is an `NSIBObjectData`: the File's Owner as its root, the
//! windows visible at launch, the connections, each object's parent, and
//! each object's and connection's id. Every object is named and defined
//! where the encoder first meets it, as Xcode's are.

use crate::nib::{self, reference, Object, Value};
use crate::storyboard::{attr, child, children, fail, number, only_attributes, only_children, runtime_class_name};
use roxmltree::{Document, Node};
use std::collections::HashMap;

/// A nib being built: the strings and numbers it shares.
struct Nib {
    shared: HashMap<String, String>,
    next: usize,
    /// Whether the document is in `Base.lproj`, where its titles are
    /// localizable.
    localizable: bool,
}

impl Nib {
    fn shared(&mut self, key: String, make: impl FnOnce() -> Object) -> Value {
        if let Some(name) = self.shared.get(&key) {
            return reference(name.clone());
        }
        self.next += 1;
        let name = format!("{}{}", nib::SHARED, self.next);
        self.shared.insert(key, name.clone());
        Value::New(make().named(name))
    }

    fn string(&mut self, s: &str) -> Value {
        self.shared(format!("s:{s}"), || Object::new("NSString", vec![("NS.bytes", Value::Data(s.as_bytes().to_vec()))]))
    }

    /// A title or other text an object `id` shows: in `Base.lproj` an
    /// `NSLocalizableString` keyed `<id>.<key>`, its development text the
    /// shared string; empty, a plain one.
    fn text(&mut self, id: &str, key: &str, s: &str) -> Value {
        if !self.localizable || s.is_empty() {
            return self.string(s);
        }
        let key = self.string(&format!("{id}.{key}"));
        let dev = self.string(s);
        Value::New(Object::new(
            "NSLocalizableString",
            vec![("NS.bytes", Value::Data(s.as_bytes().to_vec())), ("NSKey", key), ("NSDev", dev)],
        ))
    }

    fn int_number(&mut self, i: i64) -> Value {
        self.shared(format!("n:{i}"), || Object::new("NSNumber", vec![("NS.intval", Value::Int(i))]))
    }

    fn empty_array(&mut self) -> Value {
        self.shared("empty array".into(), || Object::new("NSArray", vec![("NSInlinedValue", Value::Bool(true))]))
    }

    fn array(&mut self, class: &'static str, items: Vec<Value>) -> Value {
        let mut values = vec![("NSInlinedValue", Value::Bool(true))];
        values.extend(items.into_iter().map(|v| ("UINibEncoderEmptyKey", v)));
        Value::New(Object::new(class, values))
    }
}

/// The name an archived document object goes by: shared, so that it is
/// defined where the encoder first meets it.
fn object_name(id: &str) -> String {
    format!("{}o:{id}", nib::SHARED)
}

/// A number as NSStringFromRect and its kin write it.
fn g(x: f64) -> String {
    cf::xml::format_real(x)
}

struct Frame {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn frame(n: Node, key: &str) -> Result<Frame, String> {
    let r = child(n, "rect", key).ok_or_else(|| format!("<{}> without a {key} rect", n.tag_name().name()))?;
    only_attributes(r, &[("key", None), ("x", None), ("y", None), ("width", None), ("height", None)])?;
    Ok(Frame {
        x: number(attr(r, "x")?)?,
        y: number(attr(r, "y")?)?,
        w: number(attr(r, "width")?)?,
        h: number(attr(r, "height")?)?,
    })
}

/// What compiling the document collects beside the objects themselves.
struct Collected {
    /// (object, parent) in the order the nib lists them.
    parents: Vec<(String, String)>,
    /// The windows visible at launch.
    visible: Vec<String>,
    /// The ids of the objects whose connections the nib holds.
    connection_sources: Vec<String>,
}

/// An autoresizing mask's bits, as NSView's: flexible left margin 1, width
/// 2, right margin 4, bottom margin 8, height 16, top margin 32.
fn autoresizing_mask(n: Node) -> Result<i64, String> {
    let Some(m) = child(n, "autoresizingMask", "autoresizingMask") else {
        return fail(format!("<{}> without an autoresizing mask is not supported by this ibtool", n.tag_name().name()));
    };
    only_attributes(
        m,
        &[
            ("key", None),
            ("flexibleMinX", Some("YES")),
            ("widthSizable", Some("YES")),
            ("flexibleMaxX", Some("YES")),
            ("flexibleMinY", Some("YES")),
            ("heightSizable", Some("YES")),
            ("flexibleMaxY", Some("YES")),
        ],
    )?;
    let mut mask = 0;
    for (name, bit) in [
        ("flexibleMinX", 1),
        ("widthSizable", 2),
        ("flexibleMaxX", 4),
        ("flexibleMinY", 8),
        ("heightSizable", 16),
        ("flexibleMaxY", 32),
    ] {
        if m.attribute(name).is_some() {
            mask |= bit;
        }
    }
    Ok(mask)
}

/// A view and its subviews: an `NSView`, or for a top-level custom view an
/// `NSClassSwapper` for its class.
fn view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str, next_responder: Option<&str>, swapped: bool) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("key", None),
            ("id", None),
            ("wantsLayer", Some("YES")),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "subviews"])?;
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    let mut values = Vec::new();
    let class = runtime_class_name(n, module, "NSView")?;
    if swapped {
        values.push(("NSClassName", nib.string(&class)));
        values.push(("NSOriginalClassName", nib.string("NSView")));
    } else if class != "NSView" {
        return fail("a custom class on a view other than a top-level one is not supported by this ibtool");
    }
    values.push(("NSNextResponder", next_responder.map_or(Value::Nil, reference)));
    values.push(("NSNibTouchBar", Value::Nil));
    values.push(("NSvFlags", Value::Int(256 | autoresizing_mask(n)?)));
    // Its subviews, then its frame (a template's text field; plain views
    // alone so far).
    let subviews: Vec<Node> = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    if !subviews.is_empty() {
        let mut items = Vec::new();
        for s in subviews {
            if !matches!(s.tag_name().name(), "view" | "customView") {
                return fail(format!("a subview <{}> is not supported by this ibtool", s.tag_name().name()));
            }
            items.push(view(nib, c, s, module, &name, Some(&name), false)?);
        }
        values.push(("NSSubviews", nib.array("NSMutableArray", items)));
    }
    let f = frame(n, "frame")?;
    if f.x == 0.0 && f.y == 0.0 {
        values.push(("NSFrameSize", nib.string(&format!("{{{}, {}}}", g(f.w), g(f.h)))));
    } else {
        values.push(("NSFrame", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(f.x), g(f.y), g(f.w), g(f.h)))));
    }
    if next_responder.is_some() {
        values.push(("NSSuperview", reference(parent.to_string())));
    }
    if n.attribute("wantsLayer").is_some() {
        values.push(("NSViewIsLayerTreeHost", Value::Bool(true)));
    }
    values.push(("NSViewWantsBestResolutionOpenGLSurface", Value::Bool(true)));
    values.push(("IBNSSafeAreaLayoutGuide", Value::Nil));
    values.push(("IBNSLayoutMarginsGuide", Value::Nil));
    values.push(("IBNSClipsToBounds", Value::Int(0)));
    Ok(Value::New(Object::new(if swapped { "NSClassSwapper" } else { "NSView" }, values).named(name)))
}

/// A window's position flags in `NSWTFlags` (bits 19 to 22), from its
/// initial position's struts: without one on an axis, or with both, both
/// of the axis's bits; with one, the other's (a left strut keeps 0x80000, a
/// right one 0x100000, a top one 0x200000, a bottom one 0x400000); with a
/// left and a bottom one alone, none. Springs change nothing (64 probes).
fn position_flags(n: Node) -> Result<i64, String> {
    let Some(m) = child(n, "windowPositionMask", "initialPositionMask") else {
        return Ok(0x78_0000);
    };
    only_attributes(
        m,
        &[
            ("key", None),
            ("leftStrut", Some("YES")),
            ("rightStrut", Some("YES")),
            ("topStrut", Some("YES")),
            ("bottomStrut", Some("YES")),
            ("horizontalStrut", Some("YES")),
            ("verticalStrut", Some("YES")),
        ],
    )?;
    let has = |s: &str| m.attribute(s).is_some();
    let x = match (has("leftStrut"), has("rightStrut")) {
        (true, false) => 0x8_0000,
        (false, true) => 0x10_0000,
        _ => 0x18_0000,
    };
    let y = match (has("topStrut"), has("bottomStrut")) {
        (true, false) => 0x20_0000,
        (false, true) => 0x40_0000,
        _ => 0x60_0000,
    };
    Ok(if (x, y) == (0x8_0000, 0x40_0000) { 0 } else { x | y })
}

/// A window, as an `NSWindowTemplate`.
fn window(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("title", None),
            ("allowsToolTipsWhenApplicationIsInactive", Some("NO")),
            ("autorecalculatesKeyViewLoop", Some("NO")),
            ("releasedWhenClosed", Some("NO")),
            ("deferred", Some("NO")),
            ("frameAutosaveName", None),
            ("animationBehavior", Some("default")),
            ("visibleAtLaunch", Some("NO")),
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
        ],
    )?;
    only_children(n, &["windowStyleMask", "windowPositionMask", "rect", "value", "view", "point", "connections"])?;
    for p in children(n, "point") {
        if p.attribute("key") != Some("canvasLocation") {
            return fail("a window's point other than its canvas location is not supported by this ibtool");
        }
    }
    // Its minimum and maximum sizes: its content's as given, and its
    // frame's with the title bar's 32 points when titled (seven probes).
    let mut sizes = [None, None];
    for v in children(n, "value") {
        only_attributes(v, &[("key", None), ("type", Some("size")), ("width", None), ("height", None)])?;
        let at = match attr(v, "key")? {
            "minSize" => 0,
            "maxSize" => 1,
            other => return fail(format!("a window's {other} is not supported by this ibtool")),
        };
        sizes[at] = Some((number(attr(v, "width")?)?, number(attr(v, "height")?)?));
    }
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    if n.attribute("visibleAtLaunch").is_none() {
        c.visible.push(name.clone());
    }
    let style = child(n, "windowStyleMask", "styleMask").ok_or("a window without a style mask")?;
    only_attributes(
        style,
        &[
            ("key", None),
            ("titled", Some("YES")),
            ("closable", Some("YES")),
            ("miniaturizable", Some("YES")),
            ("resizable", Some("YES")),
        ],
    )?;
    let mut mask = 0;
    for (a, bit) in [("titled", 1), ("closable", 2), ("miniaturizable", 4), ("resizable", 8)] {
        if style.attribute(a).is_some() {
            mask |= bit;
        }
    }
    // Flags: deferred (0x20000000, the default), not released when closed
    // (0x40000000), recalculating the key view loop (0x800) and tooltips
    // while inactive (0x2000), both defaults, and the position's (28
    // probes).
    let mut flags = position_flags(n)?;
    if n.attribute("deferred").is_none() {
        flags |= 0x2000_0000;
    }
    if n.attribute("releasedWhenClosed").is_some() {
        flags |= 0x4000_0000;
    }
    if n.attribute("autorecalculatesKeyViewLoop").is_none() {
        flags |= 0x800;
    }
    if n.attribute("allowsToolTipsWhenApplicationIsInactive").is_none() {
        flags |= 0x2000;
    }
    // Its content rectangle, centred on the screen along an axis without a
    // strut (16 probes).
    let content = frame(n, "contentRect")?;
    let screen = frame(n, "screenRect")?;
    let struts = child(n, "windowPositionMask", "initialPositionMask");
    let has = |s: &str| struts.is_some_and(|m| m.attribute(s).is_some());
    let x = if has("leftStrut") || has("rightStrut") {
        content.x
    } else {
        screen.x + (screen.w - content.w) / 2.0
    };
    let y = if has("topStrut") || has("bottomStrut") {
        content.y
    } else {
        screen.y + (screen.h - content.h) / 2.0
    };
    let contents = children(n, "view").collect::<Vec<_>>();
    let [content_view] = contents.as_slice() else {
        return fail("a window without one content view is not supported by this ibtool");
    };
    if content_view.attribute("key") != Some("contentView") {
        return fail("a window's view other than its content view is not supported by this ibtool");
    }
    let titlebar = if mask & 1 != 0 { 32.0 } else { 0.0 };
    let size = |nib: &mut Nib, (w, h): (f64, f64), extra: f64| nib.string(&format!("{{{}, {}}}", g(w), g(h + extra)));
    let mut values = vec![
        ("NSWindowStyleMask", Value::Int(mask)),
        ("NSWindowBacking", Value::Int(2)),
        ("NSWindowRect", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(x), g(y), g(content.w), g(content.h)))),
        ("NSWTFlags", Value::Int(flags)),
        ("NSWindowTitle", nib.text(id, "title", n.attribute("title").unwrap_or(""))),
        ("NSWindowSubtitle", nib.string("")),
        ("NSWindowClass", nib.string(&runtime_class_name(n, module, "NSWindow")?)),
        ("NSViewClass", Value::Nil),
        ("NSUserInterfaceItemIdentifier", Value::Nil),
    ];
    if let Some(max) = sizes[1] {
        values.push(("NSWindowContentMaxSize", size(nib, max, 0.0)));
    }
    if let Some(min) = sizes[0] {
        values.push(("NSWindowContentMinSize", size(nib, min, 0.0)));
    }
    values.push(("NSWindowView", view(nib, c, *content_view, module, &name, None, false)?));
    values.push(("NSScreenRect", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(screen.x), g(screen.y), g(screen.w), g(screen.h)))));
    if let Some(min) = sizes[0] {
        values.push(("NSMinSize", size(nib, min, titlebar)));
    }
    values.push((
        "NSMaxSize",
        match sizes[1] {
            Some(max) => size(nib, max, titlebar),
            None => nib.string("{10000000000000, 10000000000000}"),
        },
    ));
    if let Some(name) = n.attribute("frameAutosaveName") {
        values.push(("NSFrameAutosaveName", nib.string(name)));
    }
    values.push(("NSWindowIsRestorable", Value::Bool(true)));
    values.push(("NSMinFullScreenContentSize", nib.string("{0, 0}")));
    values.push(("NSMaxFullScreenContentSize", nib.string("{0, 0}")));
    Ok(Value::New(Object::new("NSWindowTemplate", values).named(name)))
}

/// A menu: its title, items and, for a system menu, the name AppKit finds
/// it by. Its items, and their submenus, follow it in the objects' list.
fn menu(nib: &mut Nib, c: &mut Collected, n: Node, parent: &str) -> Result<Value, String> {
    only_attributes(n, &[("key", Some("submenu")), ("title", None), ("systemMenu", None), ("id", None)])?;
    only_children(n, &["items", "point"])?;
    canvas_location_only(n)?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    let mut items = Vec::new();
    for list in children(n, "items") {
        only_children(list, &["menuItem"])?;
        for i in children(list, "menuItem") {
            items.push(menu_item(nib, c, i, &name)?);
        }
    }
    let title = nib.text(attr(n, "id")?, "title", n.attribute("title").unwrap_or(""));
    let mut values = vec![("NSTitle", title), ("NSMenuItems", nib.array("NSMutableArray", items))];
    if let Some(system) = n.attribute("systemMenu") {
        let system = match system {
            "main" => "_NSMainMenu",
            "apple" => "_NSAppleMenu",
            "services" => "_NSServicesMenu",
            "recentDocuments" => "_NSRecentDocumentsMenu",
            "font" => "_NSFontMenu",
            "window" => "_NSWindowsMenu",
            "help" => "_NSHelpMenu",
            other => return fail(format!("system menu {other} is not supported by this ibtool")),
        };
        values.push(("NSName", nib.string(system)));
    }
    Ok(Value::New(Object::new("NSMenu", values).named(name)))
}

/// A title: its attribute, or a `<string key="title">` (one with leading
/// white space, as Xcode's main menu template's writing directions).
fn title(n: Node) -> Result<String, String> {
    let element = children(n, "string").find(|s| s.attribute("key") == Some("title"));
    match (n.attribute("title"), element) {
        (Some(t), None) => Ok(t.to_string()),
        (None, Some(e)) => {
            only_attributes(e, &[("key", None)])?;
            Ok(e.text().unwrap_or("").to_string())
        }
        (None, None) => Ok(String::new()),
        (Some(_), Some(_)) => fail("a title both as an attribute and an element"),
    }
}

/// Refuses points but a canvas location, which is the editor's alone.
fn canvas_location_only(n: Node) -> Result<(), String> {
    for p in children(n, "point") {
        if p.attribute("key") != Some("canvasLocation") {
            return fail(format!("a point other than a canvas location in <{}> is not supported by this ibtool", n.tag_name().name()));
        }
    }
    Ok(())
}

/// An image AppKit provides, as a nib names it: `NSCustomResource`.
fn system_image(nib: &mut Nib, name: &str, size: &str) -> Value {
    let key = format!("resource:{name}");
    let class = nib.string("NSImage");
    let resource = nib.string(name);
    let size = nib.string(size);
    let size = Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", size)]));
    nib.shared(key, || {
        Object::new(
            "NSCustomResource",
            vec![
                ("NSClassName", class),
                ("NSResourceName", resource),
                ("IBNamespaceID", Value::Nil),
                ("IBDesignSize", size),
                ("IBDesignImageConfiguration", Value::Nil),
            ],
        )
    })
}

/// A key equivalent's modifiers, as NSEvent's flags: shift 1 << 17,
/// control 1 << 18, option 1 << 19, command 1 << 20; without a mask, command
/// for an item with a key equivalent.
fn modifier_mask(n: Node) -> Result<i64, String> {
    let Some(m) = child(n, "modifierMask", "keyEquivalentModifierMask") else {
        return Ok(if n.attribute("keyEquivalent").is_some_and(|k| !k.is_empty()) { 1 << 20 } else { 0 });
    };
    only_attributes(m, &[("key", None), ("shift", Some("YES")), ("control", Some("YES")), ("option", Some("YES")), ("command", Some("YES"))])?;
    let mut mask = 0;
    for (a, bit) in [("shift", 1 << 17), ("control", 1 << 18), ("option", 1 << 19), ("command", 1 << 20)] {
        if m.attribute(a).is_some() {
            mask |= bit;
        }
    }
    Ok(mask)
}

/// A menu item: its menu (by a conditional reference), title, key
/// equivalent and the system's check and mixed-state images; one with a
/// submenu opens it with `submenuAction:`, targeting it.
fn menu_item(nib: &mut Nib, c: &mut Collected, n: Node, menu_name: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("title", None),
            ("keyEquivalent", None),
            ("tag", None),
            ("isSeparatorItem", Some("YES")),
            ("enabled", Some("NO")),
            ("id", None),
        ],
    )?;
    only_children(n, &["modifierMask", "menu", "connections", "string"])?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), menu_name.to_string()));
    c.connection_sources.push(n.attribute("id").unwrap_or_default().to_string());
    let separator = n.attribute("isSeparatorItem").is_some();
    let mut values = vec![("NSMenu", Value::Conditional(menu_name.to_string()))];
    if separator || n.attribute("enabled").is_some() {
        values.push(("NSIsDisabled", Value::Bool(true)));
    }
    if separator {
        values.push(("NSIsSeparator", Value::Bool(true)));
    }
    values.push(("NSAllowsKeyEquivalentLocalization", Value::Bool(true)));
    values.push(("NSAllowsKeyEquivalentMirroring", Value::Bool(true)));
    let title = title(n)?;
    values.push(("NSTitle", nib.text(attr(n, "id")?, "title", &title)));
    values.push(("NSKeyEquiv", nib.string(n.attribute("keyEquivalent").unwrap_or(""))));
    let mask = modifier_mask(n)?;
    if mask != 0 {
        values.push(("NSKeyEquivModMask", Value::Int(mask)));
    }
    values.push(("NSMnemonicLoc", Value::Int(0x7fff_ffff)));
    values.push(("NSOnImage", system_image(nib, "NSMenuCheckmark", "{18, 16}")));
    values.push(("NSMixedImage", system_image(nib, "NSMenuMixedState", "{18, 4}")));
    if let Some(tag) = n.attribute("tag") {
        values.push(("NSTag", Value::Int(tag.parse().map_err(|_| format!("not a tag: {tag}"))?)));
    }
    let submenus: Vec<Node> = children(n, "menu").collect();
    if let [submenu] = submenus.as_slice() {
        let submenu_name = object_name(attr(*submenu, "id")?);
        values.push(("NSAction", nib.string("submenuAction:")));
        values.push(("NSTarget", menu(nib, c, *submenu, &name)?));
        values.push(("NSSubmenu", reference(submenu_name)));
    } else if submenus.len() > 1 {
        return fail("a menu item with several submenus");
    }
    values.push(("NSHiddenInRepresentation", Value::Bool(false)));
    Ok(Value::New(Object::new("NSMenuItem", values).named(name)))
}

/// A placeholder or custom object's class: `IBClassReference` and
/// `NSClassName` (the class the runtime looks up).
fn class_values(nib: &mut Nib, n: Node, module: Option<&str>, runtime: Option<&str>) -> Result<Vec<(&'static str, Value)>, String> {
    let Some(class) = n.attribute("customClass") else {
        return Ok(vec![("NSClassName", nib.string(runtime.unwrap_or("NSObject")))]);
    };
    let provider = n.attribute("customModuleProvider").filter(|p| !p.is_empty());
    let module_name = match (provider, module) {
        (Some(_), Some(m)) => Some(m.to_string()),
        _ => n.attribute("customModule").map(str::to_string),
    };
    let reference = Object::new(
        "IBClassReference",
        vec![
            ("IBClassName", nib.string(class)),
            ("IBModuleName", module_name.map_or(Value::Nil, |m| nib.string(&m))),
            ("IBModuleProvider", provider.map_or(Value::Nil, |p| nib.string(p))),
        ],
    );
    let runtime = match runtime {
        Some(r) => r.to_string(),
        None => runtime_class_name(n, module, "NSObject")?,
    };
    Ok(vec![("IBClassReference", Value::New(reference)), ("NSClassName", nib.string(&runtime))])
}

/// AppKit classes whose custom objects are `NSCustomObject`s.
const APPKIT_OBJECTS: [&str; 1] = ["NSFontManager"];

/// Compiles an AppKit xib to its nib.
pub fn compile_xib(xml: &str, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.attribute("targetRuntime") != Some("MacOSX.Cocoa") {
        return fail("an AppKit xib for another runtime than MacOSX.Cocoa is not supported by this ibtool");
    }
    only_children(root, &["dependencies", "objects", "resources"])?;
    let objects_node = children(root, "objects").next().ok_or("a xib without objects")?;
    let mut nib = Nib {
        shared: HashMap::new(),
        next: 0,
        localizable,
    };
    let mut c = Collected {
        parents: Vec::new(),
        visible: Vec::new(),
        connection_sources: vec!["-2".into()],
    };
    let mut owner = None;
    let mut application = None;
    let mut tops = Vec::new();
    let mut connections: Vec<(String, Value)> = Vec::new();
    for o in objects_node.children().filter(|c| c.is_element()) {
        match (o.tag_name().name(), o.attribute("id")) {
            ("customObject", Some("-2")) => owner = Some(o),
            ("customObject", Some("-1")) => {
                only_attributes(o, &[("id", None), ("userLabel", None), ("customClass", Some("FirstResponder"))])?;
                only_children(o, &[])?;
            }
            ("customObject", Some("-3")) => application = Some(o),
            ("customView" | "window" | "customObject" | "menu", _) => tops.push(o),
            (tag, _) => return fail(format!("a xib's top-level <{tag}> is not supported by this ibtool")),
        }
    }
    let owner = owner.ok_or("a xib without a File's Owner")?;
    only_attributes(
        owner,
        &[("id", None), ("userLabel", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None)],
    )?;
    only_children(owner, &["connections"])?;
    let owner_name = object_name("-2");
    let root_object = Object::new("NSCustomObject", class_values(&mut nib, owner, module, None)?).named(owner_name.clone());

    // The application placeholder: its class, NSApplication at run time.
    let application = application.ok_or("a xib without an Application placeholder")?;
    only_attributes(application, &[("id", None), ("userLabel", None), ("customClass", None)])?;
    only_children(application, &[])?;
    let application_name = object_name("-3");
    // Of class NSObject, it refers to that class; of NSApplication, it
    // names that alone (a template).
    let values = match application.attribute("customClass") {
        Some("NSObject") => class_values(&mut nib, application, module, Some("NSApplication"))?,
        Some("NSApplication") => vec![("NSClassName", nib.string("NSApplication"))],
        _ => return fail("an Application placeholder of a class other than NSObject or NSApplication is not supported by this ibtool"),
    };
    let application_object = Object::new("NSCustomObject", values).named(application_name.clone());
    c.parents.push((application_name.clone(), owner_name.clone()));

    let mut definitions = Vec::new();
    for t in &tops {
        if !t.has_tag_name("customObject") && !t.has_tag_name("menu") {
            c.connection_sources.push(attr(*t, "id")?.to_string());
        }
        let value = match t.tag_name().name() {
            "window" => window(&mut nib, &mut c, *t, module, &owner_name)?,
            "menu" => menu(&mut nib, &mut c, *t, &owner_name)?,
            // A custom object: its class swapped in for NSObject, made with
            // init as the document says (an installer pane template).
            "customObject" => {
                only_attributes(*t, &[("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("userLabel", None)])?;
                only_children(*t, &["connections"])?;
                if t.attribute("customClass").is_none() {
                    return fail("a custom object without a class is not supported by this ibtool");
                }
                let name = object_name(attr(*t, "id")?);
                c.parents.push((name.clone(), owner_name.clone()));
                c.connection_sources.push(attr(*t, "id")?.to_string());
                // An AppKit class's is an NSCustomObject of its class, as a
                // placeholder's (Xcode's main menu templates' font manager);
                // the app's own classes are swapped in for NSObject, made
                // with init when the document says so (its app delegate).
                let class = attr(*t, "customClass")?;
                let appkit = APPKIT_OBJECTS.contains(&class);
                if !appkit && class.starts_with("NS") {
                    return fail(format!("a custom object of class {class} is not supported by this ibtool"));
                }
                if !appkit && root.attribute("customObjectInstantitationMethod") != Some("direct") {
                    return fail("a custom object made other than directly is not supported by this ibtool");
                }
                if appkit {
                    let values = class_values(&mut nib, *t, module, None)?;
                    definitions.push(Value::New(Object::new("NSCustomObject", values).named(name)));
                    continue;
                }
                let values = vec![
                    ("NSClassName", nib.string(&runtime_class_name(*t, module, "NSObject")?)),
                    ("NSOriginalClassName", nib.string("NSObject")),
                    ("NSInitializeWithInit", Value::Bool(true)),
                ];
                Value::New(Object::new("NSClassSwapper", values).named(name))
            }
            _ => view(&mut nib, &mut c, *t, module, &owner_name, None, true)?,
        };
        definitions.push(value);
    }

    // Connections: outlets, then actions (whose target is left out when it
    // is the first responder), each by their source's id (as bytes; Xcode's
    // main menu template's 87, and two corpus xibs), in document order for
    // one source.
    let by_id: HashMap<&str, Node> = doc.descendants().filter(|d| d.is_element()).filter_map(|d| d.attribute("id").map(|id| (id, d))).collect();
    let mut sources: Vec<&str> = c.connection_sources.iter().map(String::as_str).collect();
    sources.sort();
    for kind in ["outlet", "action"] {
        for id in &sources {
            let id = *id;
            let source = by_id.get(id).ok_or("a connection's source")?;
            for list in children(*source, "connections") {
                only_children(list, &["outlet", "action"])?;
                for o in list.children().filter(|c| c.has_tag_name(kind)) {
                    let connector = if o.has_tag_name("outlet") {
                        only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
                        Object::new(
                            "NSNibOutletConnector",
                            vec![
                                ("NSSource", reference(object_name(id))),
                                ("NSDestination", reference(object_name(attr(o, "destination")?))),
                                ("NSLabel", nib.string(attr(o, "property")?)),
                                ("NSChildControllerCreationSelectorName", Value::Nil),
                            ],
                        )
                    } else {
                        only_attributes(o, &[("selector", None), ("target", None), ("id", None)])?;
                        let mut values = vec![("NSSource", reference(object_name(id)))];
                        match attr(o, "target")? {
                            "-1" => {}
                            target => values.push(("NSDestination", reference(object_name(target)))),
                        }
                        values.push(("NSLabel", nib.string(attr(o, "selector")?)));
                        Object::new("NSNibControlConnector", values)
                    };
                    let name = object_name(attr(o, "id")?);
                    connections.push((name.clone(), Value::New(connector.named(name))));
                }
            }
        }
    }

    // Every object is defined where the encoder first meets it (all are
    // shared): the definitions stand in the objects' list.
    let visible: Vec<Value> = c.visible.iter().map(|v| reference(v.clone())).collect();
    let visible = nib.array("NSMutableSet", visible);
    let mut oids = vec![reference(owner_name.clone())];
    let mut keys = Vec::new();
    let mut parents = Vec::new();
    let mut definitions = definitions.into_iter();
    let mut application_object = Some(application_object);
    for (object, parent) in &c.parents {
        oids.push(reference(object.clone()));
        parents.push(reference(parent.clone()));
        let definition = if *object == application_name {
            application_object.take().map(Value::New)
        } else if tops.iter().any(|t| t.attribute("id").map(object_name).as_ref() == Some(object)) {
            definitions.next()
        } else {
            None
        };
        keys.push(definition.unwrap_or_else(|| reference(object.clone())));
    }
    let mut connection_values = Vec::new();
    for (name, value) in connections {
        oids.push(reference(name));
        connection_values.push(value);
    }
    let numbers: Vec<Value> = (1..=oids.len() as i64).map(|i| nib.int_number(i)).collect();
    let data = Object::new(
        "NSIBObjectData",
        vec![
            ("NSRoot", Value::New(root_object)),
            ("NSVisibleWindows", visible),
            ("NSConnections", nib.array("NSMutableArray", connection_values)),
            ("NSObjectsKeys", nib.array("NSArray", keys)),
            ("NSObjectsValues", nib.array("NSArray", parents)),
            ("NSOidsKeys", nib.array("NSArray", oids)),
            ("NSOidsValues", nib.array("NSArray", numbers)),
            ("NSAccessibilityConnectors", nib.array("NSMutableArray", Vec::new())),
            ("NSAccessibilityOidsKeys", nib.empty_array()),
            ("NSAccessibilityOidsValues", nib.empty_array()),
        ],
    );
    nib::archive_appkit(Object::new("NSObject", vec![("IB.objectdata", Value::New(data)), ("IB.systemFontUpdateVersion", Value::Int(1))]))
}
