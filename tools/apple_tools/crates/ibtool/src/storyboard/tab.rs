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
        &[
            ("key", Some("tabBarItem")), ("title", None), ("image", None), ("selectedImage", None), ("largeContentSizeImage", None), ("systemItem", None), ("catalog", Some("system")),
            ("tag", None), ("id", None),
        ],
    )?;
    only_children(item, &["inset"])?;
    let insets = match children(item, "inset").collect::<Vec<_>>()[..] {
        [] => None,
        [i] if i.attribute("key") == Some("imageInsets") => {
            only_attributes(i, &[("key", None), ("minX", None), ("minY", None), ("maxX", None), ("maxY", None)])?;
            let edge = |a: &str| number(attr(i, a)?);
            Some(doubles(&[edge("minY")?, edge("minX")?, edge("maxY")?, edge("maxX")?]))
        }
        _ => return fail("a tab bar item's insets other than its image insets are not supported by this ibtool"),
    };
    let mut values = Vec::new();
    // Its large content size image comes first (two probes).
    if let Some(image) = item.attribute("largeContentSizeImage") {
        if item.attribute("systemItem").is_some() {
            return fail("a system tab bar item with a large content size image is not supported by this ibtool");
        }
        values.push(("_UIBarItemLargeContentSizeImageCodingKey", nib.image(image)));
    }
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
    // A system symbol as an image view's (one probe); beside other images
    // it is refused.
    if item.attribute("catalog").is_some() && (item.attribute("image").is_none() || item.attribute("selectedImage").is_some() || item.attribute("largeContentSizeImage").is_some()) {
        return fail("a tab bar item's system catalog beside a selected or large image, or without an image, is not supported by this ibtool");
    }
    if let Some(image) = item.attribute("image") {
        values.push(("UIImage", nib.image_in(image, item.attribute("catalog"))?));
    }
    if let Some(image) = item.attribute("selectedImage") {
        values.push(("UISelectedTemplateImage", nib.image(image)));
    }
    // Image insets, top, left, bottom and right, last.
    if let Some(insets) = insets {
        values.push(("UIImageInsets", insets));
    }
    Ok((Value::New(Object::new("UITabBarItem", values).named(name)), true))
}

/// The tab bar: at the height Xcode gives it whatever the document says.
pub(super) fn tab_bar(nib: &mut Nib, tab: Node, items: Vec<Value>) -> Result<Value, String> {
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
    // Xcode lays the bar out itself, so a flexible width and top margin
    // change nothing (two probes).
    for m in children(bar, "autoresizingMask") {
        if ![0, 10].contains(&autoresizing_mask(m)?) || m.has_children() {
            return fail("tab bars with an autoresizing mask are not supported by this ibtool");
        }
    }
    if children(bar, "color").any(|c| c.attribute("key") != Some("backgroundColor")) {
        return fail("tab bar colours other than the background are not supported by this ibtool");
    }
    let f = frame(bar)?;
    // 0x54, centred at its frame's x and 27 below its top (five probes).
    let mut values = vec![("UIBounds", doubles(&[0.0, 0.0, 0.0, 54.0])), ("UICenter", doubles(&[f.x, f.y + 27.0]))];
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
