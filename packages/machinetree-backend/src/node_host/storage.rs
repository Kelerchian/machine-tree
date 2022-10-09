use crate::{node::NodeRcc, NodeHashKey};
use std::collections::HashMap;

#[derive(Default)]
pub struct NodeStorage {
    pub nodes: HashMap<NodeHashKey, Vec<NodeRcc>>,
}

impl NodeStorage {
    pub fn insert(&mut self, node: &NodeRcc) {
        let removed_nodes_opt = self.nodes.insert(NodeHashKey::from(node), vec![]);
        if let Some(removed_nodes) = removed_nodes_opt {
            removed_nodes.iter().for_each(|item| {
                self.unlink_recursively(item);
            });
        }
    }

    pub fn unlink_recursively(&mut self, node: &NodeRcc) -> Vec<NodeRcc> {
        let key = NodeHashKey::from(node);
        let removed_nodes_opt = self.nodes.remove(&key);

        let mut all_removed: Vec<NodeRcc> = vec![];
        all_removed.push(key.0);

        if let Some(removed_nodes) = removed_nodes_opt {
            let all_removed_children: Vec<Vec<NodeRcc>> = removed_nodes
                .iter()
                .map(|item| self.unlink_recursively(item))
                .collect();
            all_removed.extend(removed_nodes);
            all_removed_children.into_iter().for_each(|removed| {
                all_removed.extend(removed);
            })
        }

        all_removed
    }

    pub fn borrow_children_mapping<'a>(&'a mut self, parent_node: &'a NodeRcc) -> &'a mut Vec<NodeRcc> {
        let key = NodeHashKey::from(parent_node);
        self.nodes.entry(key).or_insert(Default::default())
    }
}
