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
    /// Whether the document is a storyboard.
    storyboard: bool,
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
    /// The definitions of objects that the objects' list defines.
    definitions: HashMap<String, Value>,
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
            ("key", Some("window")),
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
    let content_view = match contents.as_slice() {
        [v] if v.attribute("key") == Some("contentView") => Some(*v),
        // A storyboard's window without one has a plain view of its
        // content's size (Xcode's app template).
        [] if nib.storyboard => None,
        _ => return fail("a window's views other than one content view are not supported by this ibtool"),
    };
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
    let content = match content_view {
        Some(v) => view(nib, c, v, module, &name, None, false)?,
        None => {
            let view_name = format!("{name} content view");
            c.parents.push((view_name.clone(), name.clone()));
            let values = vec![
                ("NSNextResponder", Value::Nil),
                ("NSNibTouchBar", Value::Nil),
                ("NSvFlags", Value::Int(256)),
                ("NSFrameSize", size(nib, (content.w, content.h), 0.0)),
                ("NSViewWantsBestResolutionOpenGLSurface", Value::Bool(true)),
                ("IBNSSafeAreaLayoutGuide", Value::Nil),
                ("IBNSLayoutMarginsGuide", Value::Nil),
                ("IBNSClipsToBounds", Value::Int(0)),
            ];
            Value::New(Object::new("NSView", values).named(view_name))
        }
    };
    values.push(("NSWindowView", content));
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
    // A xib's window has full-screen content sizes; a storyboard's doesn't
    // (Xcode's app template).
    if !nib.storyboard {
        values.push(("NSMinFullScreenContentSize", nib.string("{0, 0}")));
        values.push(("NSMaxFullScreenContentSize", nib.string("{0, 0}")));
    }
    // A custom class's reference last (three corpus xibs).
    if n.attribute("customClass").is_some() {
        let class = class_values(nib, n, module, None)?.into_iter().find(|v| v.0 == "IBClassReference").map(|v| v.1).ok_or("a custom class's reference")?;
        values.push(("IBClassReference", class));
    }
    Ok(Value::New(Object::new("NSWindowTemplate", values).named(name)))
}

/// A menu: its title, items and, for a system menu, the name AppKit finds
/// it by. Its items, and their submenus, follow it in the objects' list.
fn menu(nib: &mut Nib, c: &mut Collected, n: Node, parent: &str) -> Result<Value, String> {
    only_attributes(n, &[("key", None), ("title", None), ("systemMenu", None), ("id", None)])?;
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
    // In a storyboard, an item opening a submenu is allowed in limited app
    // mode (Xcode's main menu templates' 20 such items; a xib's aren't).
    if nib.storyboard && children(n, "menu").next().is_some() {
        values.push(("NSAllowedForLimitedAppMode", Value::Bool(true)));
    }
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

/// A custom object. An AppKit class's is an NSCustomObject of its class, as
/// a placeholder's (Xcode's main menu templates' font manager); the app's
/// own classes are swapped in for NSObject, made with init when the
/// document says so (its app delegate; an installer pane template).
fn custom_object(nib: &mut Nib, c: &mut Collected, t: Node, module: Option<&str>, parent: &str, direct: bool) -> Result<Value, String> {
    only_attributes(t, &[("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("userLabel", None)])?;
    only_children(t, &["connections"])?;
    let class = t.attribute("customClass").ok_or("a custom object without a class is not supported by this ibtool")?;
    let name = object_name(attr(t, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    c.connection_sources.push(attr(t, "id")?.to_string());
    let appkit = APPKIT_OBJECTS.contains(&class);
    if !appkit && class.starts_with("NS") {
        return fail(format!("a custom object of class {class} is not supported by this ibtool"));
    }
    if appkit {
        let values = class_values(nib, t, module, None)?;
        return Ok(Value::New(Object::new("NSCustomObject", values).named(name)));
    }
    if !direct {
        return fail("a custom object made other than directly is not supported by this ibtool");
    }
    let values = vec![
        ("NSClassName", nib.string(&runtime_class_name(t, module, "NSObject")?)),
        ("NSOriginalClassName", nib.string("NSObject")),
        ("NSInitializeWithInit", Value::Bool(true)),
    ];
    Ok(Value::New(Object::new("NSClassSwapper", values).named(name)))
}

/// Connections: outlets, then actions (whose target is left out when it is
/// the first responder), each by their source's id (as bytes; Xcode's main
/// menu template's 87, and two corpus xibs), in document order for one
/// source.
fn connections(nib: &mut Nib, doc: &Document, sources: &[String], first_responder: &str) -> Result<Vec<(String, Value)>, String> {
    let by_id: HashMap<&str, Node> = doc.descendants().filter(|d| d.is_element()).filter_map(|d| d.attribute("id").map(|id| (id, d))).collect();
    let mut sorted: Vec<&str> = sources.iter().map(String::as_str).collect();
    sorted.sort();
    let mut out = Vec::new();
    for kind in ["outlet", "action"] {
        for id in &sorted {
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
                        let target = attr(o, "target")?;
                        if target != first_responder {
                            values.push(("NSDestination", reference(object_name(target))));
                        }
                        values.push(("NSLabel", nib.string(attr(o, "selector")?)));
                        Object::new("NSNibControlConnector", values)
                    };
                    let name = object_name(attr(o, "id")?);
                    out.push((name.clone(), Value::New(connector.named(name))));
                }
            }
        }
    }
    Ok(out)
}

/// The nib of a root object (the File's Owner, or a storyboard nib's
/// NSObject), its objects with their parents (each defined where the
/// encoder first meets it: those `c.definitions` holds stand in the
/// objects' list) and connections.
fn assemble(nib: &mut Nib, root_object: Object, root_name: &str, mut c: Collected, connections: Vec<(String, Value)>) -> Result<Vec<u8>, String> {
    let visible: Vec<Value> = c.visible.iter().map(|v| reference(v.clone())).collect();
    let visible = nib.array("NSMutableSet", visible);
    let mut oids = vec![reference(root_name.to_string())];
    let mut keys = Vec::new();
    let mut parents = Vec::new();
    for (object, parent) in &c.parents {
        oids.push(reference(object.clone()));
        parents.push(reference(parent.clone()));
        keys.push(c.definitions.remove(object).unwrap_or_else(|| reference(object.clone())));
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

fn new_nib(localizable: bool, storyboard: bool) -> Nib {
    Nib { shared: HashMap::new(), next: 0, localizable, storyboard }
}

fn new_collected(sources: Vec<String>) -> Collected {
    Collected { parents: Vec::new(), visible: Vec::new(), connection_sources: sources, definitions: HashMap::new() }
}

/// Compiles an AppKit xib to its nib.
pub fn compile_xib(xml: &str, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.attribute("targetRuntime") != Some("MacOSX.Cocoa") {
        return fail("an AppKit xib for another runtime than MacOSX.Cocoa is not supported by this ibtool");
    }
    only_children(root, &["dependencies", "objects", "resources"])?;
    let objects_node = children(root, "objects").next().ok_or("a xib without objects")?;
    let mut nib = new_nib(localizable, false);
    let mut c = new_collected(vec!["-2".into()]);
    let mut owner = None;
    let mut application = None;
    let mut tops = Vec::new();
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
    only_attributes(owner, &[("id", None), ("userLabel", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None)])?;
    only_children(owner, &["connections"])?;
    let owner_name = object_name("-2");
    let root_object = Object::new("NSCustomObject", class_values(&mut nib, owner, module, None)?).named(owner_name.clone());

    // The application placeholder: its class, NSApplication at run time.
    let application = application.ok_or("a xib without an Application placeholder")?;
    application_placeholder(&mut nib, &mut c, application, module, &owner_name)?;

    let direct = root.attribute("customObjectInstantitationMethod") == Some("direct");
    for t in &tops {
        let value = match t.tag_name().name() {
            "window" => {
                c.connection_sources.push(attr(*t, "id")?.to_string());
                window(&mut nib, &mut c, *t, module, &owner_name)?
            }
            "menu" => menu(&mut nib, &mut c, *t, &owner_name)?,
            "customObject" => custom_object(&mut nib, &mut c, *t, module, &owner_name, direct)?,
            _ => {
                c.connection_sources.push(attr(*t, "id")?.to_string());
                view(&mut nib, &mut c, *t, module, &owner_name, None, true)?
            }
        };
        c.definitions.insert(object_name(attr(*t, "id")?), value);
    }
    let connections = connections(&mut nib, &doc, &c.connection_sources.clone(), "-1")?;
    assemble(&mut nib, root_object, &owner_name, c, connections)
}

/// A xib's Application placeholder (`-3`): of class NSObject, it refers to
/// that class; of NSApplication, it names that alone (a template).
fn application_placeholder(nib: &mut Nib, c: &mut Collected, application: Node, module: Option<&str>, parent: &str) -> Result<(), String> {
    only_attributes(application, &[("id", None), ("userLabel", None), ("customClass", None)])?;
    only_children(application, &[])?;
    let name = object_name(attr(application, "id")?);
    let values = match application.attribute("customClass") {
        Some("NSObject") => class_values(nib, application, module, Some("NSApplication"))?,
        Some("NSApplication") => vec![("NSClassName", nib.string("NSApplication"))],
        _ => return fail("an Application placeholder of a class other than NSObject or NSApplication is not supported by this ibtool"),
    };
    c.parents.push((name.clone(), parent.to_string()));
    c.definitions.insert(name.clone(), Value::New(Object::new("NSCustomObject", values).named(name)));
    Ok(())
}

/// Compiles an AppKit storyboard: its files in the `.storyboardc`. The
/// application scene's main menu is `MainMenu.nib`; the Info.plist names it
/// whether or not there is one (Xcode's storyboard template).
pub fn compile_storyboard(xml: &str, module: Option<&str>, localizable: bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.attribute("targetRuntime") != Some("MacOSX.Cocoa") {
        return fail("an AppKit storyboard for another runtime than MacOSX.Cocoa is not supported by this ibtool");
    }
    only_children(root, &["dependencies", "scenes", "resources"])?;
    let mut files = Vec::new();
    let mut nib_names: Vec<(String, String)> = Vec::new();
    let mut uuids: Vec<(String, String)> = Vec::new();
    let mut entry = None;
    // The controllers windows' content segues lead to.
    let content_controllers: Vec<&str> = doc
        .descendants()
        .filter(|d| d.has_tag_name("segue") && d.attribute("relationship") == Some("window.shadowedContentViewController"))
        .filter_map(|d| d.attribute("destination"))
        .collect();
    for s in doc.descendants().filter(|d| d.has_tag_name("segue")) {
        if s.attribute("relationship") != Some("window.shadowedContentViewController") || s.attribute("kind") != Some("relationship") {
            return fail("segues other than a window's content relationship are not supported by this ibtool");
        }
    }
    for scenes in children(root, "scenes") {
        for scene in children(scenes, "scene") {
            only_attributes(scene, &[("sceneID", None)])?;
            only_children(scene, &["objects", "point"])?;
            canvas_location_only(scene)?;
            let objects = children(scene, "objects").next().ok_or("a scene without objects")?;
            let members: Vec<Node> = objects.children().filter(|c| c.is_element()).collect();
            let first_responder = members
                .iter()
                .find(|m| m.attribute("sceneMemberID") == Some("firstResponder"))
                .ok_or("a scene without a first responder")?;
            only_attributes(*first_responder, &[("id", None), ("userLabel", None), ("customClass", Some("NSResponder")), ("sceneMemberID", None)])?;
            only_children(*first_responder, &[])?;
            let controller = members.iter().find(|m| m.attribute("sceneMemberID") == Some("viewController")).ok_or("a scene without a controller")?;
            let others: Vec<Node> = members.iter().copied().filter(|m| m != first_responder && m != controller).collect();
            match controller.tag_name().name() {
                "windowController" => {
                    let id = attr(*controller, "id")?;
                    let identifier = format!("NSWindowController-{id}");
                    let (bytes, uuid) = window_controller_nib(&doc, xml, *controller, &others, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{identifier}.nib"), bytes));
                    nib_names.push((identifier.clone(), identifier.clone()));
                    uuids.push((identifier.clone(), uuid));
                    if root.attribute("initialViewController") == Some(id) {
                        entry = Some(identifier);
                    }
                }
                // A window's content controller: its view's nib (the window
                // controller's holds the controller).
                "viewController" if content_controllers.contains(&attr(*controller, "id")?) => {
                    if !others.is_empty() || controller.attribute("storyboardIdentifier").is_some() {
                        return fail("a content controller with an identifier or objects beside it is not supported by this ibtool");
                    }
                    let (name, bytes) = view_nib(&doc, *controller, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{name}.nib"), bytes));
                }
                "application" => {
                    let nib = main_menu(&doc, *controller, &others, attr(*first_responder, "id")?, module, localizable)?;
                    files.push(("MainMenu.nib".to_string(), nib));
                    nib_names.push(("MainMenu".into(), "MainMenu".into()));
                }
                // A controller nothing loads (no identifier, not initial,
                // no segue to it) compiles to nothing (Xcode's storyboard
                // template).
                _ if controller.attribute("storyboardIdentifier").is_none()
                    && root.attribute("initialViewController") != controller.attribute("id")
                    && !doc.descendants().any(|d| d.has_tag_name("segue") && d.attribute("destination") == controller.attribute("id")) =>
                {
                    if !others.is_empty() {
                        return fail("objects beside a controller nothing loads are not supported by this ibtool");
                    }
                }
                tag => return fail(format!("an AppKit storyboard's <{tag}> scene is not supported by this ibtool")),
            }
        }
    }
    if root.attribute("initialViewController").is_some() && entry.is_none() {
        return fail("an initial controller other than a window controller is not supported by this ibtool");
    }
    // Its keys in the order CoreFoundation's table gives them inserted so
    // (two templates' Info.plists, with and without an entry point).
    let mut info = vec![
        ("NSStoryboardMainMenu", cf::Value::string("MainMenu")),
        ("NSViewControllerIdentifiersToNibNames", cf::Value::dict(nib_names.into_iter().map(|(k, v)| (k, cf::Value::string(v))))),
    ];
    if let Some(e) = &entry {
        info.push(("NSStoryboardDesignatedEntryPointIdentifier", cf::Value::string(e.clone())));
    }
    if !uuids.is_empty() {
        info.push(("NSViewControllerIdentifiersToUUIDs", cf::Value::dict(uuids.into_iter().map(|(k, v)| (k, cf::Value::string(v))))));
    }
    info.push(("NSStoryboardVersion", cf::Value::Integer(1)));
    let info = cf::Value::dict(info);
    // The main menu's nib name is the string naming the main menu (a
    // template).
    let share = |_: &cf::Value, at: &cf::binary::Position| {
        use cf::binary::Step;
        match at {
            [Step::Value(k)] => k == "NSStoryboardMainMenu",
            [Step::Value(k), Step::Key(id)] => k == "NSViewControllerIdentifiersToNibNames" && id == "MainMenu",
            _ => false,
        }
    };
    files.push(("Info.plist".to_string(), cf::binary::to_bytes_with(&info, &share, cf::binary::Dictionaries::Grown)));
    Ok(files)
}

/// The application scene's nib: an NSObject root, the application (its
/// main menu, NSApplication at run time), the scene's objects, an
/// Application placeholder, then the menu.
fn main_menu(doc: &Document, application: Node, others: &[Node], first_responder: &str, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    only_attributes(application, &[("id", None), ("sceneMemberID", None)])?;
    only_children(application, &["menu", "connections"])?;
    let mut nib = new_nib(localizable, true);
    let app_id = attr(application, "id")?;
    let mut c = new_collected(vec![app_id.to_string()]);
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let app_name = object_name(app_id);
    c.parents.push((app_name.clone(), root_name.clone()));
    let direct = true;
    for o in others {
        if !o.has_tag_name("customObject") {
            return fail(format!("an application scene's <{}> is not supported by this ibtool", o.tag_name().name()));
        }
        let value = custom_object(&mut nib, &mut c, *o, module, &root_name, direct)?;
        c.definitions.insert(object_name(attr(*o, "id")?), value);
    }
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    let menus: Vec<Node> = children(application, "menu").collect();
    let [main] = menus.as_slice() else {
        return fail("an application without one main menu is not supported by this ibtool");
    };
    if main.attribute("key") != Some("mainMenu") {
        return fail("an application's menu other than its main menu is not supported by this ibtool");
    }
    let menu_value = menu(&mut nib, &mut c, *main, &root_name)?;
    let app = Object::new("NSCustomObject", vec![("IBMainMenu", menu_value), ("NSClassName", nib.string("NSApplication"))]).named(app_name.clone());
    c.definitions.insert(app_name, Value::New(app));
    let connections = connections(&mut nib, doc, &c.connection_sources.clone(), first_responder)?;
    assemble(&mut nib, root_object, &root_name, c, connections)
}

/// A UUID for a controller's `uniqueIdentifierForStoryboardCompilation`,
/// which Xcode makes anew on every run: this ibtool derives one from the
/// document and the controller's id, so that its output is reproducible.
fn uuid(xml: &str, id: &str) -> String {
    let mut h: u128 = 0x6c62272e07bb014262b821756295c58d;
    for b in xml.bytes().chain([0]).chain(id.bytes()) {
        h ^= b as u128;
        h = h.wrapping_mul(0x0000000001000000000000000000013b);
    }
    let hex = format!("{h:032X}");
    format!("{}-{}-4{}-8{}-{}", &hex[0..8], &hex[8..12], &hex[13..16], &hex[17..20], &hex[20..32])
}

/// A storyboard placeholder, as a controller's `storyboard` outlet names it.
fn storyboard_placeholder(nib: &mut Nib, c: &mut Collected, name: &str, parent: &str) {
    c.parents.push((name.to_string(), parent.to_string()));
    let identifier = nib.string("NSStoryboardPlaceholder");
    c.definitions.insert(name.to_string(), Value::New(Object::new("NSNibExternalObjectPlaceholder", vec![("NSExternalObjectPlaceholderIdentifier", identifier)]).named(name.to_string())));
}

fn outlet(name: String, source: &str, destination: &str, label: Value) -> (String, Value) {
    let connector = Object::new(
        "NSNibOutletConnector",
        vec![
            ("NSSource", reference(source.to_string())),
            ("NSDestination", reference(destination.to_string())),
            ("NSLabel", label),
            ("NSChildControllerCreationSelectorName", Value::Nil),
        ],
    );
    (name.clone(), Value::New(connector.named(name)))
}

/// A window controller's nib: an NSObject root (its scene controller), the
/// controller holding its window and the window's content controller, each
/// with a storyboard placeholder, and the content controller set on the
/// window as a runtime attribute. Xcode lists the connections in an order
/// that varies from run to run, and gives each controller a new UUID; this
/// ibtool keeps one order of each.
fn window_controller_nib(doc: &Document, xml: &str, wc: Node, others: &[Node], first_responder: &str, module: Option<&str>, localizable: bool) -> Result<(Vec<u8>, String), String> {
    only_attributes(wc, &[("id", None), ("sceneMemberID", None)])?;
    only_children(wc, &["window", "connections"])?;
    if !others.is_empty() {
        return fail("objects beside a window controller are not supported by this ibtool");
    }
    let mut nib = new_nib(localizable, true);
    let wc_id = attr(wc, "id")?;
    let window_node = children(wc, "window").next().ok_or("a window controller without a window")?;
    let segues: Vec<Node> = children(wc, "connections").flat_map(|l| l.children().filter(|c| c.is_element())).collect();
    let [segue] = segues.as_slice() else {
        return fail("a window controller without one content segue is not supported by this ibtool");
    };
    let vc = doc.descendants().find(|d| d.attribute("id") == segue.attribute("destination") && d.attribute("sceneMemberID") == Some("viewController")).ok_or("a content segue's destination")?;
    if !vc.has_tag_name("viewController") {
        return fail("a window's content controller other than a view controller is not supported by this ibtool");
    }
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let mut c = new_collected(vec![attr(window_node, "id")?.to_string()]);
    let wc_name = object_name(wc_id);
    let vc_name = object_name(attr(vc, "id")?);
    let vc_placeholder = format!("{vc_name} storyboard");
    let wc_placeholder = format!("{wc_name} storyboard");
    storyboard_placeholder(&mut nib, &mut c, &vc_placeholder, &root_name);
    c.parents.push((wc_name.clone(), root_name.clone()));
    let window_value = window(&mut nib, &mut c, window_node, module, &wc_name)?;
    let window_name = object_name(attr(window_node, "id")?);
    c.parents.push((vc_name.clone(), window_name.clone()));
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    storyboard_placeholder(&mut nib, &mut c, &wc_placeholder, &root_name);

    let (vc_value, _) = content_controller(&mut nib, xml, vc, module, &vc_name)?;
    let wc_uuid = uuid(xml, wc_id);
    let wc_object = Object::new(
        "NSWindowController",
        vec![
            ("IBWindowTemplate", window_value),
            ("IBWindowTemplateContentViewController", vc_value),
            ("showSeguePresentationStyle", Value::Int(0)),
            ("uniqueIdentifierForStoryboardCompilation", nib.string(&wc_uuid)),
        ],
    )
    .named(wc_name.clone());
    c.definitions.insert(wc_name.clone(), Value::New(wc_object));

    // The scene controller first, then the window's own connections, the
    // storyboard outlets, the window outlet and the content controller.
    let label = nib.string("sceneController");
    let mut connections = vec![outlet(format!("{wc_name} scene controller"), &root_name, &wc_name, label)];
    connections.extend(self::connections(&mut nib, doc, &c.connection_sources.clone(), first_responder)?);
    let label = nib.string("storyboard");
    connections.push(outlet(format!("{vc_name} storyboard outlet"), &vc_name, &vc_placeholder, label));
    let label = nib.string("storyboard");
    connections.push(outlet(format!("{wc_name} storyboard outlet"), &wc_name, &wc_placeholder, label));
    let label = nib.string("window");
    connections.push(outlet(format!("{wc_name} window outlet"), &wc_name, &window_name, label));
    let values = nib.array("NSArray", vec![reference(vc_name.clone())]);
    let key_path = nib.string("contentViewController");
    let key_paths = nib.array("NSArray", vec![key_path]);
    let runtime = format!("{wc_name} content controller");
    let connector = Object::new("NSIBUserDefinedRuntimeAttributesConnector", vec![("NSObject", reference(window_name)), ("NSValues", values), ("NSKeyPaths", key_paths)]).named(runtime.clone());
    connections.push((runtime, Value::New(connector)));
    Ok((assemble(&mut nib, root_object, &root_name, c, connections)?, wc_uuid))
}

/// A content view controller as its window controller's nib holds it: its
/// class swapped in for NSViewController, naming its view's nib.
fn content_controller(nib: &mut Nib, xml: &str, vc: Node, module: Option<&str>, name: &str) -> Result<(Value, String), String> {
    only_attributes(vc, &[("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("sceneMemberID", None)])?;
    if vc.attribute("customClass").is_none() {
        return fail("a content controller without a custom class is not supported by this ibtool");
    }
    let view = children(vc, "view").next().ok_or("a view controller without a view")?;
    let nib_name = format!("{}-view-{}", attr(vc, "id")?, attr(view, "id")?);
    let values = vec![
        ("NSClassName", nib.string(&runtime_class_name(vc, module, "NSViewController")?)),
        ("NSOriginalClassName", nib.string("NSViewController")),
        ("NSNibName", nib.string(&nib_name)),
        ("NSExternalObjectsTableForViewLoading", Value::New(Object::new("NSDictionary", vec![("NSInlinedValue", Value::Bool(true))]))),
        ("showSeguePresentationStyle", Value::Int(0)),
        ("uniqueIdentifierForStoryboardCompilation", nib.string(&uuid(xml, attr(vc, "id")?))),
    ];
    Ok((Value::New(Object::new("NSClassSwapper", values).named(name.to_string())), nib_name))
}

/// A view controller's view nib: an NSObject root (the controller) whose
/// `view` outlet is the view.
fn view_nib(doc: &Document, vc: Node, first_responder: &str, module: Option<&str>, localizable: bool) -> Result<(String, Vec<u8>), String> {
    only_children(vc, &["view", "connections"])?;
    if children(vc, "connections").next().is_some() {
        return fail("a view controller's connections are not supported by this ibtool");
    }
    let mut nib = new_nib(localizable, true);
    let view_node = children(vc, "view").next().ok_or("a view controller without a view")?;
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let mut c = new_collected(Vec::new());
    let view_name = object_name(attr(view_node, "id")?);
    let value = view(&mut nib, &mut c, view_node, module, &root_name, None, false)?;
    c.definitions.insert(view_name.clone(), value);
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    let label = nib.string("view");
    let mut connections = vec![outlet(format!("{view_name} view outlet"), &root_name, &view_name, label)];
    connections.extend(self::connections(&mut nib, doc, &c.connection_sources.clone(), first_responder)?);
    let name = format!("{}-view-{}", attr(vc, "id")?, attr(view_node, "id")?);
    Ok((name, assemble(&mut nib, root_object, &root_name, c, connections)?))
}
