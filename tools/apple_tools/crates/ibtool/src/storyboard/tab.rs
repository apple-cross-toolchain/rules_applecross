//! Tab bar controllers (docs/clean-room/ibtool.md, "Tab bar controllers"):
//! one nib holding the controller, its tab bar and its children, view
//! controllers and navigation controllers as they are in their own nibs.

use super::*;

/// The ids of a tab bar controller's children: its `viewControllers`
/// relationships, in order.
pub(super) fn tab_children<'a>(tab: Node<'a, '_>) -> Result<Vec<&'a str>, String> {
    only_children(tab, &["tabBar", "connections", "nil", "toolbarItems"])?;
    for c in children(tab, "toolbarItems") {
        if c.has_children() {
            return fail("tab bar controllers with toolbar items are not supported by this ibtool");
        }
    }
    // Interface Builder's simulated metrics change nothing compiled.
    for c in children(tab, "nil") {
        if !matches!(c.attribute("key"), Some("viewControllers" | "simulatedBottomBarMetrics")) {
            return fail("<nil> inside <tabBarController> is not supported by this ibtool");
        }
    }
    let mut ids = Vec::new();
    for connections in children(tab, "connections") {
        only_children(connections, &["segue"])?;
        for s in children(connections, "segue") {
            only_attributes(s, &[("destination", None), ("kind", Some("relationship")), ("relationship", Some("viewControllers")), ("id", None)])?;
            ids.push(attr(s, "destination")?);
        }
    }
    Ok(ids)
}

/// A child's tab bar item: the document's, or the one Xcode implies (enabled
/// and spring loaded), and whether it is the document's.
pub(super) fn tab_bar_item(nib: &mut Nib, controller: Node, name: &str) -> Result<(Value, bool), String> {
    let Some(item) = child(controller, "tabBarItem", "tabBarItem") else {
        return Ok((Value::New(Object::new("UITabBarItem", vec![("UIEnabled", Value::Bool(true)), ("UISpringLoaded", Value::Bool(true))]).named(name)), false));
    };
    only_attributes(
        item,
        &[("key", Some("tabBarItem")), ("title", None), ("image", None), ("selectedImage", None), ("systemItem", None), ("tag", None), ("id", None)],
    )?;
    if item.has_children() {
        return fail("a tab bar item's insets or other children are not supported by this ibtool");
    }
    let mut values = Vec::new();
    if let Some(system) = item.attribute("systemItem") {
        // UITabBarSystemItem, as UIKit's header lists it.
        const SYSTEM_ITEMS: [&str; 12] =
            ["more", "favorites", "featured", "topRated", "recents", "contacts", "history", "bookmarks", "search", "downloads", "mostRecent", "mostViewed"];
        let index = SYSTEM_ITEMS.iter().position(|s| *s == system).ok_or_else(|| format!("tab bar system item {system} is not supported by this ibtool"))?;
        if item.attribute("title").is_some() || item.attribute("image").is_some() || item.attribute("selectedImage").is_some() {
            return fail("a system tab bar item with a title or images is not supported by this ibtool");
        }
        values.push(("UISystemItem", Value::Int(index as i64)));
    }
    values.push(("UIEnabled", Value::Bool(true)));
    if let Some(tag) = item.attribute("tag") {
        values.push(("UITag", Value::Int(tag.parse().map_err(|_| format!("not a tag: {tag}"))?)));
    }
    if let Some(title) = item.attribute("title") {
        values.push(("UITitle", nib.text(attr(item, "id")?, "title", title)));
    }
    if let Some(image) = item.attribute("image") {
        values.push(("UIImage", nib.image(image)));
    }
    if let Some(image) = item.attribute("selectedImage") {
        values.push(("UISelectedTemplateImage", nib.image(image)));
    }
    Ok((Value::New(Object::new("UITabBarItem", values).named(name)), true))
}

/// The tab bar: at the height Xcode gives it whatever the document says.
fn tab_bar(nib: &mut Nib, tab: Node, items: Vec<Value>) -> Result<Value, String> {
    let bar = children(tab, "tabBar").next().ok_or("a tab bar controller without a tab bar")?;
    only_attributes(
        bar,
        &[
            ("key", Some("tabBar")),
            ("contentMode", Some("scaleToFill")),
            ("insetsLayoutMarginsFromSafeArea", Some("NO")),
            ("translucent", Some("NO")),
            ("id", None),
        ],
    )?;
    only_children(bar, &["rect", "autoresizingMask", "color"])?;
    for m in children(bar, "autoresizingMask") {
        if autoresizing_mask(m)? != 0 || m.has_children() {
            return fail("tab bars with an autoresizing mask are not supported by this ibtool");
        }
    }
    if children(bar, "color").any(|c| c.attribute("key") != Some("backgroundColor")) {
        return fail("tab bar colours other than the background are not supported by this ibtool");
    }
    if frame(bar)?.x != 0.0 {
        return fail("tab bars not at x 0 are not supported by this ibtool");
    }
    let mut values = vec![("UIBounds", doubles(&[0.0, 0.0, 0.0, 54.0])), ("UICenter", doubles(&[0.0, 27.0]))];
    if let Some(c) = child(bar, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", color(nib, c)?));
    }
    let hugging = nib.string("{250, 250}");
    values.extend([
        ("UIDeepDrawRect", Value::Bool(true)),
        ("UIOpaque", Value::Bool(true)),
        ("UIAutoresizeSubviews", Value::Bool(true)),
        ("UIAutoresizingMask", Value::Int(10)),
        ("UIViewContentHuggingPriority", hugging),
    ]);
    if bar.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
        values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIDelegate", reference("controller")));
    values.push(("UIItems", nib.array("NSArray", items)));
    values.push(("UIBarTranslucence", Value::Int(if bar.attribute("translucent").is_some() { 2 } else { 1 })));
    Ok(Value::New(Object::new("UITabBar", values).named("tab bar")))
}

/// A tab bar controller's child: a view or table view controller, or a
/// navigation controller with its root.
pub(super) enum Child<'r, 'a, 'i> {
    Controller(&'r RootController<'a, 'i>),
    Navigation(Node<'a, 'i>, &'r RootController<'a, 'i>),
}

/// A tab bar controller's nib: a storyboard placeholder per controller it
/// holds (its children's and their roots', the last child's first), the
/// scene's own placeholders, and the controller with its children and tab
/// bar.
#[allow(clippy::too_many_arguments)]
pub(super) fn tab_bar_controller_nib(
    platform: Platform,
    color_matched: bool,
    localizable: bool,
    module: Option<&str>,
    tab: Node,
    kids: &[Child],
    swapped: bool,
    nib_names: &HashMap<String, String>,
) -> Result<(String, Vec<u8>), String> {
    only_attributes(
        tab,
        &[
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("storyboardIdentifier", None),
            ("useStoryboardIdentifierAsRestorationIdentifier", Some("YES")),
            ("title", None),
            ("automaticallyAdjustsScrollViewInsets", Some("NO")),
            ("userLabel", None),
            ("sceneMemberID", Some("viewController")),
        ],
    )?;
    let tab_id = attr(tab, "id")?;
    if let Some(identifier) = tab.attribute("storyboardIdentifier") {
        if identifier.is_empty() || identifier.contains('/') || !identifier.is_ascii() {
            return fail(format!("storyboard identifier {identifier:?} is not supported by this ibtool"));
        }
    } else if tab.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
        return fail("useStoryboardIdentifierAsRestorationIdentifier without a storyboard identifier is not supported by this ibtool");
    }
    let name = tab.attribute("storyboardIdentifier").map_or_else(|| format!("UITabBarController-{tab_id}"), str::to_string);
    let mut nib = Nib::new(platform, color_matched, localizable);

    // The controllers it holds in preorder (each child, then a navigation
    // controller's root) with their original classes; their scenes'
    // storyboard placeholders come in reverse.
    let mut held: Vec<(String, &str)> = Vec::new();
    for (i, k) in kids.iter().enumerate() {
        match k {
            Child::Controller(r) => held.push((format!("child {i}"), r.original_class)),
            Child::Navigation(_, r) => {
                held.push((format!("child {i}"), "UINavigationController"));
                held.push((format!("child {i} root"), r.original_class));
            }
        }
    }
    let mut top = Vec::new();
    for (held_name, _) in held.iter().rev() {
        top.push(proxy(&mut nib, "UIStoryboardPlaceholder", &format!("{held_name} storyboard")));
    }
    top.push(proxy(&mut nib, "IBFilesOwner", "owner"));
    top.push(proxy(&mut nib, "IBFirstResponder", "responder"));
    top.push(proxy(&mut nib, "UIStoryboardPlaceholder", "storyboard"));

    let mut values = Vec::new();
    if let Some(title) = tab.attribute("title") {
        values.push(("UITitle", nib.text(tab_id, "title", title)));
    }
    if let Some(identifier) = tab.attribute("storyboardIdentifier") {
        values.push(("UIStoryboardIdentifier", nib.string(identifier)));
        if tab.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            values.push(("UIRestorationIdentifier", nib.string(identifier)));
        }
    }
    // The objects below the children, breadth first: each child's parts
    // (a view controller's navigation item and tab bar item; a navigation
    // controller's bar, tab bar item and root), then the roots' items.
    let mut parts = Vec::new();
    let mut deeper = Vec::new();
    let mut child_values = Vec::new();
    let mut items = Vec::new();
    for (i, k) in kids.iter().enumerate() {
        let child_name = format!("child {i}");
        let item_name = format!("child {i} tab bar item");
        let no_segues = |r: &RootController| r.view_segues.is_empty() && r.own_segues.is_empty();
        match k {
            Child::Controller(r) => {
                if !no_segues(r) {
                    return fail("a tab bar controller's child with segues is not supported by this ibtool");
                }
                let (item, explicit) = tab_bar_item(&mut nib, r.vc, &item_name)?;
                let embedded = embedded_controller(&mut nib, r, &child_name, "controller", None, Some(item), nib_names)?;
                if let Some(n) = &embedded.navigation_item {
                    parts.push(reference(n.clone()));
                }
                if explicit {
                    parts.push(reference(item_name.clone()));
                }
                child_values.push(embedded.value);
            }
            Child::Navigation(nav, r) => {
                check_navigation_controller(*nav)?;
                if nav.attribute("storyboardIdentifier").is_some() {
                    return fail("a tab bar controller's navigation controller with a storyboard identifier is not supported by this ibtool");
                }
                if !no_segues(r) {
                    return fail("a tab bar controller's child with segues is not supported by this ibtool");
                }
                let root_name = format!("child {i} root");
                let (item, explicit) = tab_bar_item(&mut nib, *nav, &item_name)?;
                let embedded = embedded_controller(&mut nib, r, &root_name, &child_name, Some(&format!("{child_name} bar")), None, nib_names)?;
                let root_item = embedded.navigation_item.clone();
                child_values.push(navigation_controller(&mut nib, module, *nav, &child_name, embedded, &root_name, false, Some((item, "controller")))?);
                parts.push(reference(format!("{child_name} bar")));
                if explicit {
                    parts.push(reference(item_name.clone()));
                }
                parts.push(reference(root_name));
                if let Some(n) = root_item {
                    deeper.push(reference(n));
                }
            }
        }
        items.push(reference(item_name));
    }
    let child_refs = || (0..kids.len()).map(|i| reference(format!("child {i}"))).collect::<Vec<Value>>();
    if !kids.is_empty() {
        values.push(("UIChildViewControllers", nib.array("NSMutableArray", child_values)));
    }
    if tab.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
        values.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
    }
    values.push(("UITabBar", tab_bar(&mut nib, tab, items)?));
    values.push(("UICustomizableViewControllers", nib.array("NSMutableArray", child_refs())));
    values.push(("UIViewControllers", nib.array("NSMutableArray", child_refs())));
    let controller = if swapped || tab.attribute("customClass").is_some() {
        let class = runtime_class_name(tab, module, "UITabBarController")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UITabBarController")));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new("UITabBarController", values)
    };
    top.push(Value::New(controller.named("controller")));

    let mut objects: Vec<Value> = held.iter().rev().map(|(n, _)| reference(format!("{n} storyboard"))).collect();
    objects.extend([reference("owner"), reference("responder"), reference("storyboard"), reference("controller"), reference("tab bar")]);
    objects.extend((0..kids.len()).map(|i| reference(format!("child {i}"))));
    objects.extend(parts);
    objects.extend(deeper);
    // The controller's storyboard outlet, then the others in preorder,
    // stably by their controllers' original classes.
    let mut connections = vec![outlet(&mut nib, "storyboard", "controller", "storyboard")];
    let mut by_class = held.clone();
    by_class.sort_by(|a, b| a.1.cmp(b.1));
    for (held_name, _) in &by_class {
        connections.push(outlet(&mut nib, "storyboard", held_name, &format!("{held_name} storyboard")));
    }
    connections.push(outlet(&mut nib, "sceneViewController", "owner", "controller"));
    Ok((name, nib::archive(root_object(&mut nib, top, objects, connections))))
}
