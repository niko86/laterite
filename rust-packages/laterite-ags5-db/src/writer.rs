//! Topological ordering helper for parent-before-child group walks.
//!
//! Previously this module held a generic `BuildContext` writer pipeline
//! intended as the foundation for a future msgspec-style writer that
//! would walk the nested PROJ-rooted model graph. That writer was never
//! built — `ags4-to-db` works in flat `dict[code, list[row_dict]]` form
//! and bypasses the model tree entirely — so the scaffolding sat unused.
//! Removed in favour of just the one function still in active use:
//! `topological_order`, called by `ags4-to-db` to insert parents before
//! children.

use std::collections::HashSet;

use laterite_ags4_core::registry::Registry;

/// Parent-before-child group ordering. Mirrors Python's
/// `_topological_order` — a depth-first walk from each group, visiting
/// each ancestor before the group itself.
pub fn topological_order(registry: &Registry) -> Vec<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut order: Vec<String> = Vec::new();

    fn visit(reg: &Registry, code: &str, visited: &mut HashSet<String>, order: &mut Vec<String>) {
        if !visited.insert(code.to_string()) {
            return;
        }
        if let Some(g) = reg.get(code) {
            if let Some(p) = g.parent.as_deref() {
                visit(reg, p, visited, order);
            }
        }
        order.push(code.to_string());
    }

    for g in registry.iter() {
        visit(registry, &g.code, &mut visited, &mut order);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use laterite_ags4_core::registry::registry;

    #[test]
    fn topological_order_proj_first() {
        let order = topological_order(registry());
        let proj_idx = order.iter().position(|c| c == "PROJ").unwrap();
        let loca_idx = order.iter().position(|c| c == "LOCA").unwrap();
        let samp_idx = order.iter().position(|c| c == "SAMP").unwrap();
        assert!(proj_idx < loca_idx, "PROJ must come before LOCA");
        assert!(loca_idx < samp_idx, "LOCA must come before SAMP");
    }
}
