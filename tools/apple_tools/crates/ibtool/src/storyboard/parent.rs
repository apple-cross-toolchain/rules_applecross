//! A controller whose nib holds its children: a tab bar controller
//! (docs/clean-room/ibtool.md, "Tab bar controllers") or a split view
//! controller ("Split view controllers"). Its children are view and table
//! view controllers and navigation controllers, as in their own nibs.

use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Tab,
    Split,
}

impl Kind {
    pub(super) fn class(self) -> &'static str {
        match self {
            Kind::Tab => "UITabBarController",
            Kind::Split => "UISplitViewController",
        }
    }

    /// A child's place among `UIChildViewControllers`: a split view
    /// controller's columns go master first; a tab bar controller's keep
    /// their document order.
    fn column(self, relationship: &str) -> usize {
        match (self, relationship) {
            (Kind::Split, "masterViewController") => 0,
            (Kind::Split, _) => 1,
            (Kind::Tab, _) => 0,
        }
    }
}

/// A child: a view or table view controller, or a navigation controller
/// with its root.
pub(super) enum Child<'r, 'a, 'i> {
    Controller(&'r RootController<'a, 'i>),
    Navigation(Node<'a, 'i>, &'r RootController<'a, 'i>),
}

/// The ids of a controller's children, with their relationships, in
/// document order.
pub(super) fn children_of<'a>(kind: Kind, node: Node<'a, '_>) -> Result<Vec<(&'a str, &'a str)>, String> {
    match kind {
        Kind::Tab => Ok(tab::tab_children(node)?.into_iter().map(|id| ("viewControllers", id)).collect()),
        Kind::Split => split::split_children(node),
    }
}

/// The controller's nib: a storyboard placeholder per controller it holds
/// (its children's and their roots', in reverse), the scene's own
/// placeholders, and the controller with its children (and a tab bar
/// controller's tab bar). `kids` are in document order.
#[allow(clippy::too_many_arguments)]
pub(super) fn parent_controller_nib(
    platform: Platform,
    color_matched: bool,
    localizable: bool,
    module: Option<&str>,
    kind: Kind,
    node: Node,
    kids: &[(&str, Child)],
    swapped: bool,
    nib_names: &HashMap<String, String>,
) -> Result<(String, Vec<u8>), String> {
    let mut allowed = vec![
        ("id", None),
        ("customClass", None),
        ("customModule", None),
        ("customModuleProvider", None),
        ("storyboardIdentifier", None),
        ("useStoryboardIdentifierAsRestorationIdentifier", Some("YES")),
        ("userLabel", None),
        ("sceneMemberID", Some("viewController")),
    ];
    if kind == Kind::Tab {
        allowed.extend([
            ("title", None),
            ("automaticallyAdjustsScrollViewInsets", Some("NO")),
            ("definesPresentationContext", Some("YES")),
            ("providesPresentationContextTransitionStyle", Some("YES")),
        ]);
    }
    only_attributes(node, &allowed)?;
    let id = attr(node, "id")?;
    if let Some(identifier) = node.attribute("storyboardIdentifier") {
        if identifier.is_empty() || identifier.contains('/') || !identifier.is_ascii() {
            return fail(format!("storyboard identifier {identifier:?} is not supported by this ibtool"));
        }
    } else if node.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
        return fail("useStoryboardIdentifierAsRestorationIdentifier without a storyboard identifier is not supported by this ibtool");
    }
    let class = kind.class();
    let name = node.attribute("storyboardIdentifier").map_or_else(|| format!("{class}-{id}"), str::to_string);
    let mut nib = Nib::new(platform, color_matched, localizable);

    // The controllers it holds in preorder (each child, then a navigation
    // controller's root) with their original classes; their scenes'
    // storyboard placeholders come in reverse.
    let mut held: Vec<(String, &str)> = Vec::new();
    for (i, (_, k)) in kids.iter().enumerate() {
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
    if let Some(title) = node.attribute("title") {
        values.push(("UITitle", nib.text(id, "title", title)));
    }
    if let Some(identifier) = node.attribute("storyboardIdentifier") {
        values.push(("UIStoryboardIdentifier", nib.string(identifier)));
        if node.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            values.push(("UIRestorationIdentifier", nib.string(identifier)));
        }
    }
    // The children are archived in column order, and listed among the
    // objects in their relationships' order; then, in that order, each
    // child's parts (a view controller's navigation item and tab bar item;
    // a navigation controller's bar, tab bar item and root), then the
    // roots' navigation items.
    let mut columns: Vec<usize> = (0..kids.len()).collect();
    columns.sort_by_key(|i| kind.column(kids[*i].0));
    let mut listed: Vec<usize> = (0..kids.len()).collect();
    listed.sort_by_key(|i| kids[*i].0);
    let mut parts: Vec<Vec<Value>> = Vec::new();
    let mut deeper: Vec<Vec<Value>> = Vec::new();
    let mut child_values: Vec<Value> = Vec::new();
    let mut items = Vec::new();
    // A navigation child's root's segue templates (objects, then outlets
    // to its root), after the children's parts.
    let mut template_objects: Vec<Value> = Vec::new();
    let mut template_outlets: Vec<(String, String)> = Vec::new();
    for &i in &columns {
        let (_, k) = &kids[i];
        let child_name = format!("child {i}");
        let item_name = format!("child {i} tab bar item");
        let no_segues = |r: &RootController| r.view_segues.is_empty() && r.own_segues.is_empty();
        let mut own_parts = Vec::new();
        let mut own_deeper = Vec::new();
        match k {
            Child::Controller(r) => {
                if !no_segues(r) {
                    return fail("a child controller with segues is not supported by this ibtool");
                }
                let (item, explicit) = match kind {
                    Kind::Tab => {
                        let (item, explicit) = tab::tab_bar_item(&mut nib, r.vc, &item_name)?;
                        (Some(item), explicit)
                    }
                    Kind::Split => (None, false),
                };
                let embedded = embedded_controller(&mut nib, r, &child_name, "controller", None, item, nib_names)?;
                if !embedded.bar_items.is_empty() {
                    return fail("a child controller's bar button items are not supported by this ibtool");
                }
                if let Some(n) = &embedded.navigation_item {
                    own_parts.push(reference(n.clone()));
                }
                if explicit {
                    own_parts.push(reference(item_name.clone()));
                }
                child_values.push(embedded.value);
            }
            Child::Navigation(nav, r) => {
                check_navigation_controller(*nav)?;
                if nav.attribute("storyboardIdentifier").is_some() {
                    return fail("a child navigation controller with a storyboard identifier is not supported by this ibtool");
                }
                let root_name = format!("child {i} root");
                let title = navigation_title(&mut nib, *nav)?;
                let (item, explicit) = match kind {
                    Kind::Tab => {
                        let (item, explicit) = tab::tab_bar_item(&mut nib, *nav, &item_name)?;
                        (Some(item), explicit)
                    }
                    Kind::Split => {
                        if child(*nav, "tabBarItem", "tabBarItem").is_some() {
                            return fail("a split view controller's navigation controller with a tab bar item is not supported by this ibtool");
                        }
                        (None, false)
                    }
                };
                let mut embedded = embedded_controller(&mut nib, r, &root_name, &child_name, Some(&format!("{child_name} bar")), None, nib_names)?;
                if !embedded.bar_items.is_empty() {
                    return fail("a child navigation controller's root's bar button items are not supported by this ibtool");
                }
                // Its root's segues (Xcode's master-detail template): the
                // templates after the children's parts, their outlets to the
                // root after the scene's (one corpus storyboard, two ways).
                if !no_segues(r) {
                    if !template_objects.is_empty() || !embedded.templates.performs.is_empty() || !embedded.gestures.is_empty() {
                        return fail("segues from more than one child, or with bar button items or gestures, are not supported by this ibtool");
                    }
                    template_objects.extend(std::mem::take(&mut embedded.templates.objects));
                    template_outlets.extend(std::mem::take(&mut embedded.templates.outlets).into_iter().map(|t| (t, root_name.clone())));
                }
                let root_item = embedded.navigation_item.clone();
                child_values.push(navigation_controller(&mut nib, module, *nav, &child_name, embedded, &root_name, false, title, Some((item, "controller")))?);
                own_parts.push(reference(format!("{child_name} bar")));
                if explicit {
                    own_parts.push(reference(item_name.clone()));
                }
                own_parts.push(reference(root_name));
                if let Some(n) = root_item {
                    own_deeper.push(reference(n));
                }
            }
        }
        parts.push(own_parts);
        deeper.push(own_deeper);
        if kind == Kind::Tab {
            items.push(reference(item_name));
        }
    }
    // Back from column order to the objects' order.
    let at = |i: usize| columns.iter().position(|c| *c == i).unwrap_or(0);
    let child_refs = || columns.iter().map(|i| reference(format!("child {i}"))).collect::<Vec<Value>>();
    if !kids.is_empty() {
        values.push(("UIChildViewControllers", nib.array("NSMutableArray", child_values)));
    }
    presentation_context(node, &mut values);
    if node.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
        values.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
    }
    match kind {
        Kind::Tab => {
            values.push(("UITabBar", tab::tab_bar(&mut nib, node, items)?));
            values.push(("UICustomizableViewControllers", nib.array("NSMutableArray", child_refs())));
            values.push(("UIViewControllers", nib.array("NSMutableArray", child_refs())));
        }
        Kind::Split => values.push(("UISplitViewControllerPrimaryEdge", Value::Int(0))),
    }
    let controller = if swapped || node.attribute("customClass").is_some() {
        let runtime = runtime_class_name(node, module, class)?;
        values.push(("UIClassName", nib.string(&runtime)));
        values.push(("UIOriginalClassName", nib.string(class)));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new(class, values)
    };
    top.push(Value::New(controller.named("controller")));

    let mut objects: Vec<Value> = held.iter().rev().map(|(n, _)| reference(format!("{n} storyboard"))).collect();
    objects.extend([reference("owner"), reference("responder"), reference("storyboard"), reference("controller")]);
    if kind == Kind::Tab {
        objects.push(reference("tab bar"));
    }
    objects.extend(listed.iter().map(|i| reference(format!("child {i}"))));
    for &i in &listed {
        objects.append(&mut parts[at(i)]);
    }
    for &i in &listed {
        objects.append(&mut deeper[at(i)]);
    }
    objects.extend(template_objects);
    // The controller's storyboard outlet, then the others in preorder,
    // stably by their controllers' original classes.
    let mut connections = vec![outlet(&mut nib, "storyboard", "controller", "storyboard")];
    let mut by_class = held.clone();
    by_class.sort_by(|a, b| a.1.cmp(b.1));
    for (held_name, _) in &by_class {
        connections.push(outlet(&mut nib, "storyboard", held_name, &format!("{held_name} storyboard")));
    }
    connections.push(outlet(&mut nib, "sceneViewController", "owner", "controller"));
    for (template, root) in &template_outlets {
        connections.push(outlet(&mut nib, "viewController", template, root));
    }
    Ok((name, nib::archive(root_object(&mut nib, top, objects, connections))?))
}
