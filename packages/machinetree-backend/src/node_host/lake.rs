use crate::{node::NodeRcc, NodeHashKey};
use std::collections::HashMap;

#[derive(Default)]
pub struct NodeLake {
    pub children_map: HashMap<NodeHashKey, Vec<NodeRcc>>,
    pub parent_map: HashMap<NodeHashKey, NodeRcc>,
}

impl NodeLake {
    pub fn unlink_recursively(&mut self, node: &NodeRcc) -> Vec<NodeRcc> {
        let key = NodeHashKey::from(node);
        let removed_nodes_opt = self.children_map.remove(&key);

        self.parent_map.remove(&key);

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

    pub fn mutate_children_mapping<'a>(
        &'a mut self,
        parent_node: &'a NodeRcc,
    ) -> &'a mut Vec<NodeRcc> {
        let key = NodeHashKey::from(parent_node);
        self.children_map.entry(key).or_insert(Default::default())
    }

    pub fn generate_parent_link_for_children<'a>(&'a mut self, parent_node: &'a NodeRcc) {
        let key = NodeHashKey::from(parent_node);

        if let Some(children) = self.children_map.get(&key) {
            children.into_iter().for_each(|child| {
                self.parent_map
                    .insert(NodeHashKey::from(child), parent_node.clone());
            });
        }
    }
}
