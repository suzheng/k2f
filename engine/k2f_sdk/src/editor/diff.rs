use super::types::{Change, ChangeOp};
use k2f_core::{find_in_trees, for_each_node, node_text, RunningBlockNode, SemanticNode};
use std::collections::BTreeSet;

pub fn diff_trees(
    base_root: &SemanticNode,
    base_running: &[RunningBlockNode],
    current_root: &SemanticNode,
    current_running: &[RunningBlockNode],
) -> Vec<Change> {
    let base_ids = all_ids(base_root, base_running);
    let current_ids = all_ids(current_root, current_running);
    let mut changes = Vec::new();

    for id in base_ids.union(&current_ids) {
        let before = find_in_trees(base_root, base_running, id);
        let after = find_in_trees(current_root, current_running, id);
        match (before, after) {
            (None, Some(a)) => changes.push(Change {
                id: id.clone(),
                op: ChangeOp::Add,
                text_before: None,
                text_after: node_text(a).map(str::to_string),
                role_before: None,
                role_after: Some(a.role.clone()),
            }),
            (Some(b), None) => changes.push(Change {
                id: id.clone(),
                op: ChangeOp::Delete,
                text_before: node_text(b).map(str::to_string),
                text_after: None,
                role_before: Some(b.role.clone()),
                role_after: None,
            }),
            (Some(b), Some(a)) => {
                let text_before = node_text(b).map(str::to_string);
                let text_after = node_text(a).map(str::to_string);
                let role_changed = b.role != a.role;
                let text_changed = text_before != text_after;
                if role_changed || text_changed {
                    changes.push(Change {
                        id: id.clone(),
                        op: ChangeOp::Update,
                        text_before,
                        text_after,
                        role_before: if role_changed {
                            Some(b.role.clone())
                        } else {
                            None
                        },
                        role_after: if role_changed {
                            Some(a.role.clone())
                        } else {
                            None
                        },
                    });
                }
            }
            (None, None) => {}
        }
    }
    changes
}

fn all_ids(root: &SemanticNode, running: &[RunningBlockNode]) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for_each_node(root, &mut |n| {
        set.insert(n.id.clone());
    });
    for rb in running {
        for_each_node(&rb.node, &mut |n| {
            set.insert(n.id.clone());
        });
    }
    set
}
