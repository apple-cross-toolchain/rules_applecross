//! Split view controllers (docs/clean-room/ibtool.md, "Split view
//! controllers"): classic master and detail columns.

use super::*;

/// A split view controller's children: its relationships (master, detail),
/// as (relationship, id) in document order.
pub(super) fn split_children<'a>(split: Node<'a, '_>) -> Result<Vec<(&'a str, &'a str)>, String> {
    only_children(split, &["connections", "toolbarItems"])?;
    for c in children(split, "toolbarItems") {
        if c.has_children() {
            return fail("split view controllers with toolbar items are not supported by this ibtool");
        }
    }
    let mut kids = Vec::new();
    for connections in children(split, "connections") {
        only_children(connections, &["segue"])?;
        for s in children(connections, "segue") {
            only_attributes(s, &[("destination", None), ("kind", Some("relationship")), ("relationship", None), ("id", None)])?;
            let relationship = attr(s, "relationship")?;
            if !matches!(relationship, "masterViewController" | "detailViewController") {
                return fail(format!("a split view controller's {relationship} is not supported by this ibtool"));
            }
            if kids.iter().any(|(r, _)| *r == relationship) {
                return fail(format!("a split view controller with two of {relationship} is not supported by this ibtool"));
            }
            kids.push((relationship, attr(s, "destination")?));
        }
    }
    Ok(kids)
}
