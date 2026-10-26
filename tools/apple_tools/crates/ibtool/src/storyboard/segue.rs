//! Segues (docs/clean-room/ibtool.md, "Segues"): which controllers they
//! make loadable, and the templates their sources' nibs hold.

use super::*;
use std::collections::HashSet;

/// The controller a scene holds, if it is one this ibtool compiles.
pub(super) fn scene_controller<'a, 'i>(scene: Node<'a, 'i>) -> Option<Node<'a, 'i>> {
    children(scene, "objects")
        .next()?
        .children()
        .find(|c| matches!(c.tag_name().name(), "viewController" | "tableViewController" | "collectionViewController" | "navigationController" | "tabBarController" | "splitViewController" | "viewControllerPlaceholder"))
}

/// A controller's class, as its nib's name begins without an identifier.
pub(super) fn original_class(c: Node) -> &'static str {
    match c.tag_name().name() {
        "tableViewController" => "UITableViewController",
        "collectionViewController" => "UICollectionViewController",
        "navigationController" => "UINavigationController",
        "tabBarController" => "UITabBarController",
        "splitViewController" => "UISplitViewController",
        _ => "UIViewController",
    }
}

/// The name of a controller's own nib: its storyboard identifier, or its
/// class and id.
pub(super) fn nib_name(c: Node) -> Result<String, String> {
    Ok(match c.attribute("storyboardIdentifier") {
        Some(identifier) => identifier.to_string(),
        None => format!("{}-{}", original_class(c), attr(c, "id")?),
    })
}

/// The segues leaving a controller's scene other than relationships: its
/// own and its controls'.
fn outgoing<'a, 'i>(c: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    c.descendants().filter(|d| d.has_tag_name("segue") && d.attribute("kind") != Some("relationship"))
}

/// The controllers only segues make loadable: those a segue leads to from
/// a scene that is loaded (initial or identified, reached by a segue, or a
/// loaded navigation controller's root), other than initial or identified
/// ones. A scene nothing loads takes its segues' destinations with it.
pub(super) fn segue_destinations<'a>(controllers: &[Node<'a, '_>], initial: Option<&str>) -> Result<HashSet<&'a str>, String> {
    let find = |id: &str| controllers.iter().copied().find(|c| c.attribute("id") == Some(id));
    let mut loaded: Vec<&'a str> = Vec::new();
    for c in controllers {
        let id = attr(*c, "id")?;
        if initial == Some(id) || c.attribute("storyboardIdentifier").is_some() {
            loaded.push(id);
        }
    }
    let mut reached = HashSet::new();
    let mut i = 0;
    while i < loaded.len() {
        let Some(c) = find(loaded[i]) else {
            i += 1;
            continue;
        };
        i += 1;
        if matches!(c.tag_name().name(), "navigationController" | "tabBarController" | "splitViewController") {
            for s in c.descendants().filter(|d| d.has_tag_name("segue") && d.attribute("kind") == Some("relationship")) {
                let root = attr(s, "destination")?;
                if !loaded.contains(&root) {
                    loaded.push(root);
                }
            }
        }
        // An unwind segue loads nothing: it leads to its scene's exit.
        for s in outgoing(c).filter(|s| s.attribute("kind") != Some("unwind")) {
            let destination = attr(s, "destination")?;
            if find(destination).is_none() {
                return fail("a segue to anything but a view, table view, collection view, navigation, tab bar or split view controller or a storyboard reference is not supported by this ibtool");
            }
            if !loaded.contains(&destination) {
                loaded.push(destination);
                reached.insert(destination);
            }
        }
    }
    Ok(reached)
}

/// The order Xcode lists controllers' nibs in the Info.plist: depth first
/// from each initial or identified controller, in document order, through
/// its relationships, its own segues and then its view's.
pub(super) fn load_order<'a>(controllers: &[Node<'a, '_>], initial: Option<&str>) -> Vec<&'a str> {
    fn visit<'a>(c: Node<'a, '_>, controllers: &[Node<'a, '_>], order: &mut Vec<&'a str>) {
        let Some(id) = c.attribute("id") else { return };
        if order.contains(&id) {
            return;
        }
        order.push(id);
        let segues = c.descendants().filter(|d| d.has_tag_name("segue") && d.attribute("kind") != Some("unwind"));
        let own = |s: &Node| s.parent().is_some_and(|p| p.has_tag_name("connections") && p.parent() == Some(c));
        let relationships = segues.clone().filter(|s| s.attribute("kind") == Some("relationship"));
        let owned = segues.clone().filter(|s| s.attribute("kind") != Some("relationship") && own(s));
        let viewed = segues.filter(|s| s.attribute("kind") != Some("relationship") && !own(s));
        for s in relationships.chain(owned).chain(viewed) {
            if let Some(d) = controllers.iter().find(|d| d.attribute("id").is_some() && d.attribute("id") == s.attribute("destination")) {
                visit(*d, controllers, order);
            }
        }
    }
    let mut order = Vec::new();
    for c in controllers {
        if c.attribute("id") == initial || c.attribute("storyboardIdentifier").is_some() {
            visit(*c, controllers, &mut order);
        }
    }
    order
}

/// A segue's template: its identifier, its destination's nib and how it is
/// performed.
fn template(nib: &mut Nib, s: Node, names: &HashMap<String, String>) -> Result<Object, String> {
    if s.attribute("kind") == Some("unwind") {
        return unwind_template(nib, s);
    }
    only_attributes(s, &[
        ("destination", None), ("kind", None), ("identifier", None), ("animates", Some("NO")), ("modalPresentationStyle", None),
        ("modalTransitionStyle", Some("crossDissolve")), ("id", None),
    ])?;
    let destination = names.get(attr(s, "destination")?).ok_or("a segue to anything but a view, table view, collection view, navigation, tab bar or split view controller or a storyboard reference is not supported by this ibtool")?;
    let mut values = Vec::new();
    if let Some(identifier) = s.attribute("identifier") {
        values.push(("UIIdentifier", nib.string(identifier)));
    }
    values.push(("UIDestinationViewControllerIdentifier", nib.string(destination)));
    let animates = s.attribute("animates").is_some();
    let style = s.attribute("modalPresentationStyle");
    let transition = s.attribute("modalTransitionStyle").is_some();
    let kind = attr(s, "kind")?;
    if kind != "presentation" && (style.is_some() || transition) {
        return fail(format!("a {kind} segue's presentation styles are not supported by this ibtool"));
    }
    let class = match kind {
        "show" | "showDetail" => {
            if animates {
                values.push(("UIAnimates", Value::Bool(false)));
            }
            let action = if kind == "show" { "showViewController:sender:" } else { "showDetailViewController:sender:" };
            values.push(("UIActionName", nib.string(action)));
            "UIStoryboardShowSegueTemplate"
        }
        "presentation" => {
            if [style.is_some(), transition, animates].iter().filter(|x| **x).count() > 1 {
                return fail("a presentation segue with more than one of its styles and animates is not supported by this ibtool");
            }
            if let Some(style) = style {
                let value = match style {
                    "fullScreen" => 0,
                    "pageSheet" => 1,
                    "formSheet" => 2,
                    "currentContext" => 3,
                    "overFullScreen" => 5,
                    "overCurrentContext" => 6,
                    "popover" => 7,
                    "automatic" => -2,
                    other => return fail(format!("modalPresentationStyle=\"{other}\" is not supported by this ibtool")),
                };
                values.push(("UIModalPresentationStyle", Value::Int(value)));
            }
            if transition {
                values.push(("UIModalTransitionStyle", Value::Int(2)));
            }
            if animates {
                values.push(("UIAnimates", Value::Bool(false)));
            }
            "UIStoryboardPresentationSegueTemplate"
        }
        "embed" => {
            if animates {
                return fail("an embed segue that doesn't animate is not supported by this ibtool");
            }
            values.push(("UIPerformOnViewLoad", Value::Bool(true)));
            values.push(("UIContainerView", Value::Nil));
            "UIStoryboardEmbedSegueTemplate"
        }
        "push" => {
            if animates {
                return fail("a push segue that doesn't animate is not supported by this ibtool");
            }
            values.push(("UIDestinationContainmentContext", Value::Int(0)));
            values.push(("UISplitViewControllerIndex", Value::Int(0)));
            "UIStoryboardPushSegueTemplate"
        }
        other => return fail(format!("{other} segues are not supported by this ibtool")),
    };
    Ok(Object::new(class, values))
}

/// An unwind segue's template: no destination, the action it unwinds to.
fn unwind_template(nib: &mut Nib, s: Node) -> Result<Object, String> {
    only_attributes(s, &[("destination", None), ("kind", None), ("identifier", None), ("unwindAction", None), ("animates", Some("NO")), ("id", None)])?;
    // Interface Builder connects it to the exit of its own scene.
    let destination = attr(s, "destination")?;
    let scene = s.ancestors().find(|a| a.has_tag_name("scene"));
    let exit = scene.and_then(|scene| scene.descendants().find(|d| d.attribute("id") == Some(destination)));
    if !exit.is_some_and(|e| e.has_tag_name("exit")) {
        return fail("an unwind segue to anything but its scene's exit is not supported by this ibtool");
    }
    let mut values = Vec::new();
    if let Some(identifier) = s.attribute("identifier") {
        values.push(("UIIdentifier", nib.string(identifier)));
    }
    values.push(("UIDestinationViewControllerIdentifier", Value::Nil));
    if s.attribute("animates").is_some() {
        values.push(("UIAnimates", Value::Bool(false)));
    }
    values.push(("UIAction", nib.string(attr(s, "unwindAction")?)));
    Ok(Object::new("UIStoryboardUnwindSegueTemplate", values))
}

/// A controller's segue templates as one of its nibs (its own, or its
/// navigation controller's) holds them.
pub(super) struct Templates {
    /// The external objects table for its view's placeholders: the
    /// controller's (`UpstreamPlaceholder-1`) if its view refers to it, then
    /// each control's segue template, in the view's order.
    pub external: Vec<Value>,
    /// `UIStoryboardSegueTemplates`: the controller's own segues, then its
    /// controls', each in document order.
    pub array: Option<Value>,
    /// The templates, as the nib's objects list them.
    pub objects: Vec<Value>,
    /// The templates' names, in the order of their `viewController`
    /// outlets: the array's reversed, then (stably) by class name.
    pub outlets: Vec<String>,
}

/// The templates of a controller (`controller`, its object's name in the
/// nib) whose view triggers `view_segues` (as listed, each with its
/// placeholder's index) and which itself has `own_segues`.
#[allow(clippy::too_many_arguments)]
pub(super) fn templates(
    nib: &mut Nib, controller: &str, upstream_used: bool, first: usize, view_segues: &[(usize, Node)], own_segues: &[Node], names: &HashMap<String, String>,
) -> Result<Templates, String> {
    // `first` is the controller's placeholder's number (a gesture
    // recognizer's comes before it).
    let mut external = Vec::new();
    if upstream_used {
        external.push(nib.string(&format!("UpstreamPlaceholder-{first}")));
        external.push(reference(controller));
    }
    // The external objects in placeholder order, the rest as listed.
    let mut by_placeholder: Vec<&(usize, Node)> = view_segues.iter().collect();
    by_placeholder.sort_by_key(|s| s.0);
    let mut classes = HashMap::new();
    for (i, s) in by_placeholder {
        let name = format!("segue {i}");
        external.push(nib.string(&format!("UpstreamPlaceholder-{}", i + first + usize::from(upstream_used))));
        let t = template(nib, *s, names)?;
        classes.insert(*i, t.class);
        external.push(Value::New(t.named(name)));
    }
    let view_classes: Vec<(&'static str, String)> = view_segues.iter().map(|(i, _)| (classes[i], format!("segue {i}"))).collect();
    let mut outlets: Vec<(&'static str, String)> = Vec::new();
    let mut array = Vec::new();
    let mut objects = Vec::new();
    for (i, s) in own_segues.iter().enumerate() {
        if s.attribute("identifier").is_none() {
            return fail("a controller's segue without an identifier is not supported by this ibtool");
        }
        let name = format!("own segue {i}");
        let t = template(nib, *s, names)?;
        outlets.push((t.class, name.clone()));
        array.push(Value::New(t.named(name.clone())));
        objects.push(reference(name));
    }
    for (i, _) in view_segues {
        array.push(reference(format!("segue {i}")));
        objects.push(reference(format!("segue {i}")));
    }
    outlets.extend(view_classes);
    let array = if array.is_empty() { None } else { Some(nib.array("NSArray", array)) };
    outlets.reverse();
    outlets.sort_by(|a, b| a.0.cmp(b.0));
    Ok(Templates { external, array, objects, outlets: outlets.into_iter().map(|e| e.1).collect() })
}
