use k2f_core::{for_each_node, RunningBlockNode};
use std::collections::HashSet;

pub(crate) fn running_ids(running: &[RunningBlockNode]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for rb in running {
        for_each_node(&rb.node, &mut |n| {
            ids.insert(n.id.clone());
        });
    }
    ids
}
