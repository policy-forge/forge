//! Pure borrowed requirement tree, trace and pointer traversal shared by both private declaration families.
//!
//! These functions create no original, approval, currentness or source permission. The /1
//! wrapper supplies its existing genuine scope/relationship callback. The /2 producer must
//! supply the actual held native/source selection and its original caller work adapter.

use serde_json::Value;

use super::queries::{QueryResult, Reason, checkpoint};
use crate::workspace::preparation::WorkControl;

/// Complete native walk ceiling; the original /1 depth rule is unchanged.
const MAX_DEPTH: usize = 64;
/// Complete exact pointer ceiling shared with the maintained /1 projection.
const MAX_POINTER_BYTES: usize = 4096;

/// Walk complete Catalog groups and recursively nested controls without fallbacks.
pub(crate) fn catalog_nodes<'a>(
    node: &'a Value,
    depth: usize,
    visit: &mut impl FnMut(&'a Value, &mut dyn WorkControl) -> QueryResult<()>,
    charge: &mut dyn FnMut(usize) -> QueryResult<()>,
    control: &mut dyn WorkControl,
) -> QueryResult<()> {
    checkpoint(control)?;
    if depth > MAX_DEPTH {
        return Err(Reason::InvalidArtifact.into());
    }
    for group in array(node, "groups")? {
        charge(1)?;
        catalog_nodes(group, depth + 1, visit, charge, control)?;
    }
    for child in array(node, "controls")? {
        visit(child, control)?;
        catalog_nodes(child, depth + 1, visit, charge, control)?;
    }
    Ok(())
}

/// Treat absent optional arrays as empty, but never silently skip wrong types.
pub(crate) fn array<'a>(node: &'a Value, key: &str) -> QueryResult<&'a [Value]> {
    match node.get(key) {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        _ => Err(Reason::InvalidArtifact.into()),
    }
}

/// Borrow an actual nonempty native string without normalization or generated IDs.
pub(crate) fn native_string<'a>(node: &'a Value, key: &str) -> QueryResult<&'a str> {
    node.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Reason::InvalidArtifact.into())
}

/// Refuse ambiguous trace properties before the maintained first-match extractor.
pub(crate) fn unique_trace(node: &Value) -> QueryResult<()> {
    let namespace = crate::oscal::trace_embedding::FORGE_TRACE_NS;
    for name in ["source-file", "source-section", "source-line"] {
        let mut properties = array(node, "props")?.iter().filter(|property| {
            property.get("ns").and_then(Value::as_str) == Some(namespace)
                && property.get("name").and_then(Value::as_str) == Some(name)
        });
        let first = properties.next().ok_or(Reason::SourceSpanUnavailable)?;
        if first.get("value").and_then(Value::as_str).is_none() || properties.next().is_some() {
            return Err(Reason::SourceSpanUnavailable.into());
        }
    }
    Ok(())
}

/// Locate one actual logical source line with exact original UTF-8 byte offsets.
///
/// The one-based rule matches maintained trace line references. A trailing newline
/// does not create an invented last line. CR is removed only as part of CRLF.
pub(crate) fn line_span(text: &str, wanted: usize) -> Option<(usize, usize)> {
    if wanted == 0 {
        return None;
    }
    let mut start = 0;
    for (offset, line) in text.split_inclusive('\n').enumerate() {
        if offset + 1 == wanted {
            let mut end = start + line.len();
            if line.ends_with('\n') {
                end -= 1;
                if line.ends_with("\r\n") {
                    end -= 1;
                }
            }
            return Some((start, end));
        }
        start += line.len();
    }
    None
}

/// Locate an actual borrowed node without accepting any detached citation proof.
pub(crate) fn pointer_for_node(
    tree: &Value,
    node: &Value,
    control: &mut dyn WorkControl,
) -> QueryResult<String> {
    let mut path = String::new();
    if locate(tree, node, &mut path, 0, control)? {
        return Ok(path);
    }
    Err(Reason::InvalidArtifact.into())
}

/// Find by actual borrowed node identity, retaining only one bounded traversal path.
fn locate(
    node: &Value,
    target: &Value,
    path: &mut String,
    depth: usize,
    control: &mut dyn WorkControl,
) -> QueryResult<bool> {
    checkpoint(control)?;
    if depth > MAX_DEPTH {
        return Err(Reason::InvalidArtifact.into());
    }
    if std::ptr::eq(node, target) {
        return Ok(true);
    }
    match node {
        Value::Object(fields) => {
            for (key, child) in fields {
                let mark = path.len();
                push_pointer(path, key)?;
                if locate(child, target, path, depth + 1, control)? {
                    return Ok(true);
                }
                path.truncate(mark);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                let mark = path.len();
                push_pointer(path, &index.to_string())?;
                if locate(child, target, path, depth + 1, control)? {
                    return Ok(true);
                }
                path.truncate(mark);
            }
        }
        _ => {}
    }
    Ok(false)
}

/// Escape one actual pointer component under its byte ceiling before each append.
fn push_pointer(path: &mut String, component: &str) -> QueryResult<()> {
    let size = component
        .bytes()
        .try_fold(1_usize, |size, byte| {
            size.checked_add(if byte == b'~' || byte == b'/' { 2 } else { 1 })
        })
        .ok_or(Reason::OutputBoundExceeded)?;
    if path.len().checked_add(size).is_none_or(|length| length > MAX_POINTER_BYTES) {
        return Err(Reason::OutputBoundExceeded.into());
    }
    path.push('/');
    for character in component.chars() {
        match character {
            '~' => path.push_str("~0"),
            '/' => path.push_str("~1"),
            _ => path.push(character),
        }
    }
    Ok(())
}

/// Append an actual /2 traversal cursor component after its complete inspection charge.
/// The caller owns the bounded path scratch; this is ordinary traversal, never a supplied pointer.
pub(crate) fn append_located(
    path: &mut String,
    component: &str,
    charge: &mut dyn FnMut(usize) -> QueryResult<()>,
) -> QueryResult<()> {
    let passes = component
        .len()
        .div_ceil(32 * 1024)
        .checked_mul(2)
        .and_then(|amount| amount.checked_add(1))
        .ok_or(Reason::OutputBoundExceeded)?;
    charge(passes)?;
    push_pointer(path, component)
}

/// Walk complete /2 Catalog controls carrying exact actual native traversal cursors.
/// /1's original walker and pointer search bodies above remain unchanged. Actual JSON
/// container/array depth is also checked so every successful cursor has /1 search parity.
pub(crate) fn located_catalog_nodes<'a>(
    node: &'a Value,
    walk_depth: usize,
    tree_depth: usize,
    path: &mut String,
    visit: &mut impl FnMut(&'a Value, &str, &mut dyn WorkControl) -> QueryResult<()>,
    charge: &mut dyn FnMut(usize) -> QueryResult<()>,
    control: &mut dyn WorkControl,
) -> QueryResult<()> {
    checkpoint(control)?;
    if walk_depth > MAX_DEPTH || tree_depth > MAX_DEPTH {
        return Err(Reason::InvalidArtifact.into());
    }
    let mark = path.len();
    let groups = array(node, "groups")?;
    if !groups.is_empty() {
        append_located(path, "groups", charge)?;
        for (index, group) in groups.iter().enumerate() {
            charge(1)?;
            let array_mark = path.len();
            append_located(path, &index.to_string(), charge)?;
            located_catalog_nodes(
                group,
                walk_depth + 1,
                tree_depth + 2,
                path,
                visit,
                charge,
                control,
            )?;
            path.truncate(array_mark);
        }
        path.truncate(mark);
    }
    let controls = array(node, "controls")?;
    if !controls.is_empty() {
        append_located(path, "controls", charge)?;
        for (index, child) in controls.iter().enumerate() {
            checkpoint(control)?;
            if tree_depth + 2 > MAX_DEPTH {
                return Err(Reason::InvalidArtifact.into());
            }
            let array_mark = path.len();
            append_located(path, &index.to_string(), charge)?;
            visit(child, path.as_str(), control)?;
            located_catalog_nodes(
                child,
                walk_depth + 1,
                tree_depth + 2,
                path,
                visit,
                charge,
                control,
            )?;
            path.truncate(array_mark);
        }
        path.truncate(mark);
    }
    Ok(())
}

/// Walk every /2 Component and Capability implementation with actual structural cursors.
/// No trace property, native ID or caller citation is accepted as a pointer component.
pub(crate) fn located_component_nodes<'a>(
    root: &'a Value,
    path: &mut String,
    visit: &mut impl FnMut(&'a Value, &str, &mut dyn WorkControl) -> QueryResult<()>,
    charge: &mut dyn FnMut(usize) -> QueryResult<()>,
    control: &mut dyn WorkControl,
) -> QueryResult<()> {
    checkpoint(control)?;
    let root_mark = path.len();
    for field in ["components", "capabilities"] {
        append_located(path, field, charge)?;
        let field_mark = path.len();
        for (container_index, container) in array(root, field)?.iter().enumerate() {
            checkpoint(control)?;
            charge(1)?;
            append_located(path, &container_index.to_string(), charge)?;
            append_located(path, "control-implementations", charge)?;
            let container_mark = path.len();
            for (implementation_index, implementation) in
                array(container, "control-implementations")?.iter().enumerate()
            {
                checkpoint(control)?;
                charge(1)?;
                append_located(path, &implementation_index.to_string(), charge)?;
                append_located(path, "implemented-requirements", charge)?;
                let implementation_mark = path.len();
                for (requirement_index, node) in
                    array(implementation, "implemented-requirements")?.iter().enumerate()
                {
                    checkpoint(control)?;
                    append_located(path, &requirement_index.to_string(), charge)?;
                    visit(node, path.as_str(), control)?;
                    path.truncate(implementation_mark);
                }
                path.truncate(container_mark);
            }
            path.truncate(field_mark);
        }
        path.truncate(root_mark);
    }
    Ok(())
}
