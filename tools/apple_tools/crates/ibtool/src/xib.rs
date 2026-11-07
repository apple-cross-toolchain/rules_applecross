//! iOS xibs, as `ibtool --compile X.nib … X.xib` compiles them; see
//! docs/clean-room/ibtool.md.
//!
//! Xcode's nib of a xib holding one view is the view nib it compiles for a
//! storyboard scene with that view (`<vc>-view-<view>.nib`), but for the
//! File's Owner: the xib's owner is the nib's own `IBFilesOwner` proxy, where
//! a scene's controller is an `UpstreamPlaceholder` of its own, and the
//! owner's `view` outlet is there only when the xib has it. So the xib is
//! rewritten as that one-scene storyboard, its owner the scene's controller,
//! and compiled by the storyboard compiler, which leaves the placeholder out.

use roxmltree::{Document, Node};

/// The id the scene's controller takes; a xib's own ids are Interface
/// Builder's `abc-de-fgh` or the placeholders' negative numbers.
const CONTROLLER: &str = "xib-files-owner";

/// The views that may be top-level beside a xib's view, compiled as its
/// subviews are (the views a storyboard compiles, as `subview` does).
pub(crate) const BESIDE_VIEWS: [&str; 9] = ["view", "label", "button", "textField", "imageView", "tableView", "collectionView", "webView", "wkWebView"];

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

/// The nib of an iOS xib.
pub fn compile(xml: &str, minimum_deployment_target: Option<&str>, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    compile_with(xml, minimum_deployment_target, module, localizable, false)
}

/// The id of the placeholder view a lone table, collection or web view is
/// compiled beside, which the nib leaves out.
const PLACEHOLDER: &str = "xib-placeholder-view";

/// `placeholder`: whether the xib's view is the placeholder a lone view was
/// given, its owner's `view` outlet then the lone view's.
fn compile_with(xml: &str, minimum_deployment_target: Option<&str>, module: Option<&str>, localizable: bool, placeholder: bool) -> Result<Vec<u8>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    // An iPad xib compiles as any other (one probe).
    let kind = root.attribute("type");
    if !matches!(kind, Some("com.apple.InterfaceBuilder3.CocoaTouch.XIB" | "com.apple.InterfaceBuilder3.CocoaTouch.iPad.XIB")) {
        return fail("only iOS xibs are supported by this ibtool");
    }
    crate::storyboard::refuse_before_ios8(root)?;
    if doc.descendants().any(|d| d.attribute("id") == Some(CONTROLLER)) {
        return fail(format!("an object with id {CONTROLLER} is not supported by this ibtool"));
    }
    let objects = elements(root).find(|c| c.has_tag_name("objects")).ok_or("a xib in the archive format before Xcode 5 is not supported by this ibtool")?;
    let mut owner = None;
    let mut responder = None;
    let mut view = None;
    // Table view cells and views after a top-level view, compiled into its
    // nib.
    let mut cells: Vec<Node> = Vec::new();
    // Or one of those views alone, compiled beside a placeholder view the
    // nib then leaves out.
    let mut lone = None;
    for o in elements(objects) {
        match (o.tag_name().name(), o.attribute("placeholderIdentifier")) {
            ("placeholder", Some("IBFilesOwner")) if owner.is_none() => owner = Some(o),
            ("placeholder", Some("IBFirstResponder")) if responder.is_none() => {
                if elements(o).next().is_some() || o.attribute("customClass").is_some_and(|c| c != "UIResponder") {
                    return fail("a first responder with connections or a class of its own is not supported by this ibtool");
                }
                responder = Some(o.attribute("id").ok_or("a first responder without an id")?);
            }
            ("view" | "collectionReusableView" | "tableViewCell" | "collectionViewCell", _) if view.is_none() && lone.is_none() => view = Some(o),
            (tag, _) if view.is_none() && lone.is_none() && BESIDE_VIEWS.contains(&tag) => lone = Some(o),
            // Beside a lone view too: the placeholder view's then (below).
            (tag, _) if (view.is_some_and(|v| v.has_tag_name("view")) || lone.is_some()) && (tag == "tableViewCell" || tag == "customObject" || BESIDE_VIEWS.contains(&tag)) => cells.push(o),
            (tag, _) => return fail(format!("a xib's top-level {tag} beside its one view is not supported by this ibtool")),
        }
    }
    let owner = owner.ok_or("a xib without a File's Owner is not supported by this ibtool")?;
    // A lone view (a table view controller's template, say) is the root
    // beside a placeholder view, which the nib leaves out: its values,
    // objects and outlets are as beside a view (three probes).
    // An empty xib (Xcode's Empty template) is its placeholders alone, as
    // the placeholder view leaves them (one probe).
    if lone.is_some() || (view.is_none() && cells.is_empty()) {
        if placeholder {
            return fail("a xib's lone view beside a placeholder");
        }
        let stub = format!("<view contentMode=\"scaleToFill\" id=\"{PLACEHOLDER}\">\n<rect key=\"frame\" x=\"0.0\" y=\"0.0\" width=\"0.0\" height=\"0.0\"/>\n<autoresizingMask key=\"autoresizingMask\"/>\n</view>\n");
        let at = lone.map_or(objects.range().end - "</objects>".len(), |l| l.range().start);
        if !xml[..objects.range().end].ends_with("</objects>") {
            return fail("a xib's objects element");
        }
        let rewritten = format!("{}{stub}{}", &xml[..at], &xml[at..]);
        return compile_with(&rewritten, minimum_deployment_target, module, localizable, true);
    }
    let view = view.ok_or("a xib without a top-level view is not supported by this ibtool")?;

    // A cell is a prototype cell's nib, with its safe-area guide.
    if view.has_tag_name("tableViewCell") || view.has_tag_name("collectionViewCell") {
        // Its outlets are the cell nib's (`table::cell_nib`).
        let outlets_only = elements(owner).all(|c| c.has_tag_name("connections") && elements(c).all(|o| o.has_tag_name("outlet")));
        if elements(owner).next().is_some() && (!outlets_only || !view.has_tag_name("tableViewCell")) {
            return fail("a File's Owner's connections beside a cell are not supported by this ibtool");
        }
        let safe_area = |c: &Node| c.has_tag_name("viewLayoutGuide") && c.attribute("key") == Some("safeArea");
        let cell = without(xml, view, |c| canvas(c) || safe_area(c));
        let document = format!("{}{cell}{}", &xml[..view.range().start], &xml[view.range().end..]);
        let guide = elements(view).find(|c| safe_area(c)).map(|c| c.attribute("id").ok_or("a safe-area guide without an id")).transpose()?;
        return if view.has_tag_name("tableViewCell") {
            crate::storyboard::compile_xib_cell(&document, module, localizable, guide.is_some())
        } else {
            crate::storyboard::compile_xib_collection_cell(&document, module, localizable, guide)
        };
    }
    let owner_id = owner.attribute("id").ok_or("a File's Owner without an id")?;
    if view.attribute("key").is_some() {
        return fail("a keyed top-level view is not supported by this ibtool");
    }

    // The owner, as the scene's controller: its class and its outlets (its
    // `view` outlet among them when the xib has one), its view the xib's.
    let mut controller = format!("<viewController id=\"{CONTROLLER}\"");
    for (name, value) in owner.attributes().map(|a| (a.name(), a.value())) {
        match name {
            "placeholderIdentifier" | "id" | "userLabel" => {}
            "customClass" | "customModule" | "customModuleProvider" => controller.push_str(&format!(" {name}=\"{}\"", escape(value))),
            _ => return fail(format!("a File's Owner's {name} is not supported by this ibtool")),
        }
    }
    controller.push_str(" sceneMemberID=\"viewController\">\n");
    let view_id = view.attribute("id").ok_or("a view without an id")?;
    let mut outlets = String::new();
    for c in elements(owner) {
        if !c.has_tag_name("connections") {
            return fail(format!("a File's Owner's {} is not supported by this ibtool", c.tag_name().name()));
        }
        for o in elements(c) {
            if o.has_tag_name("outlet") && o.attribute("property") == Some("view") && o.attribute("destination") != Some(view_id) && !placeholder {
                return fail("a File's Owner's view outlet to anything but the top-level view is not supported by this ibtool");
            }
            outlets.push_str(&xml[o.range()]);
            outlets.push('\n');
        }
    }
    // Actions and outlets to the owner are the controller's.
    let retarget = |text: &str| text.replace(&format!("destination=\"{owner_id}\""), &format!("destination=\"{CONTROLLER}\""));
    let text = without(xml, view, canvas);
    let tag = format!("<{}", view.tag_name().name());
    controller.push_str(&retarget(&text.replacen(&tag, &format!("{tag} key=\"view\""), 1)));
    controller.push('\n');
    if !outlets.is_empty() {
        controller.push_str(&format!("<connections>\n{}</connections>\n", retarget(&outlets)));
    }
    controller.push_str("</viewController>\n");
    // Actions sent to the first responder are the scene's.
    let responder = responder.map_or(String::new(), |id| format!("<placeholder placeholderIdentifier=\"IBFirstResponder\" id=\"{}\" sceneMemberID=\"firstResponder\"/>\n", escape(id)));
    // Cells beside the view follow the controller, their canvas places
    // left out.
    let safe_area = |c: &Node| c.has_tag_name("viewLayoutGuide") && c.attribute("key") == Some("safeArea");
    let cell_text: String = cells.iter().map(|c| retarget(&without(xml, *c, |d| canvas(d) || safe_area(d))) + "\n").collect();
    let cell_guides = cells.iter().filter(|c| elements(**c).any(|d| safe_area(&d))).filter_map(|c| c.attribute("id").map(str::to_string)).collect();
    let scene = format!("<scenes>\n<scene sceneID=\"{CONTROLLER}-scene\">\n<objects>\n{controller}{responder}{cell_text}</objects>\n</scene>\n</scenes>");

    // The document's own attributes, as a storyboard's, with the scene as
    // its initial controller.
    let head_end = xml[root.range().start..].find('>').map(|i| root.range().start + i).ok_or("a xib's document element")?;
    let mut head = xml[root.range().start..head_end].trim_end_matches('/').to_string();
    head = head.replacen(&format!("type=\"{}\"", kind.unwrap_or_default()), "type=\"com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB\"", 1);
    if root.attribute("initialViewController").is_some() {
        return fail("a xib's initialViewController is not supported by this ibtool");
    }
    head.push_str(&format!(" initialViewController=\"{CONTROLLER}\""));
    let storyboard = format!("{}{head}{}{scene}{}", &xml[..root.range().start], &xml[head_end..objects.range().start], &xml[objects.range().end..]);

    // Its first responder's place among its view and cells, which the nib
    // keeps.
    let roots: Vec<Node> = elements(objects).filter(|o| *o == view || cells.contains(o)).collect();
    let responder_at = match elements(objects).position(|o| o.attribute("placeholderIdentifier") == Some("IBFirstResponder")) {
        Some(r) => roots.iter().filter(|o| elements(objects).position(|e| e == **o) < Some(r)).count(),
        None => 0,
    };
    let layout = crate::storyboard::XibLayout { responder_at, cell_guides, placeholder };
    let compiled = crate::storyboard::compile_xib(&storyboard, minimum_deployment_target, module, localizable, &layout)?;
    let name = format!("{CONTROLLER}-view-{view_id}.nib");
    compiled.files.into_iter().find(|f| f.0 == name).map(|f| f.1).ok_or_else(|| "the xib's view compiled to no nib".to_string())
}

/// Where a top-level object sits on Interface Builder's canvas, and the
/// bars and screen it's shown with there (`simulated…Metrics`), the
/// editor's alone.
fn canvas(c: &Node) -> bool {
    match c.attribute("key") {
        Some("canvasLocation") => c.has_tag_name("point"),
        Some(key) => key.starts_with("simulated") && key.ends_with("Metrics"),
        None => false,
    }
}

/// `n`'s text without its children `leave_out` takes.
fn without(xml: &str, n: Node, leave_out: impl Fn(&Node) -> bool) -> String {
    let mut text = String::new();
    let mut at = n.range().start;
    for c in elements(n).filter(|c| leave_out(c)) {
        text.push_str(&xml[at..c.range().start]);
        at = c.range().end;
    }
    text.push_str(&xml[at..n.range().end]);
    text
}

fn elements<'a, 'i>(n: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    n.children().filter(|c| c.is_element())
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}
