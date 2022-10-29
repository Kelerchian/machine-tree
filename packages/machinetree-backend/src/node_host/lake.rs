use crate::{
    embeddable::context_holder::ContextHolder,
    node::{NodeDataRaw, NodeKey, NodeKeyArc, NodeKeyWeak, NodeSeed},
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

#[derive(Default)]
pub struct NodeRelations {
    pub(crate) children: Vec<NodeKeyWeak>,
    pub(crate) parent: Option<NodeKeyWeak>,
}

pub struct NodeDataPoint {
    pub(crate) self_data: Rc<RefCell<NodeDataRaw>>,
    pub(crate) context_holder: Rc<RefCell<ContextHolder>>,
    pub(crate) relations: Rc<RefCell<NodeRelations>>,
}

impl NodeDataPoint {
    pub(crate) fn borrow_relations<'a>(&'a self) -> Ref<NodeRelations> {
        self.relations.borrow()
    }

    pub(crate) fn borrow_mut_relations<'a>(&'a self) -> RefMut<NodeRelations> {
        self.relations.borrow_mut()
    }

    pub(crate) fn borrow_mut_context<'a>(&'a self) -> RefMut<ContextHolder> {
        self.context_holder.borrow_mut()
    }

    pub(crate) fn borrow_data_mut<'a>(&'a self) -> RefMut<NodeDataRaw> {
        self.self_data.borrow_mut()
    }
}

#[derive(Clone)]
pub struct NodeData(Rc<RefCell<NodeDataPoint>>);

impl From<NodeDataPoint> for NodeData {
    fn from(data: NodeDataPoint) -> Self {
        NodeData(Rc::new(RefCell::new(data)))
    }
}

impl NodeData {
    pub(crate) fn borrow_self<'a>(&'a self) -> Ref<'a, NodeDataPoint> {
        self.0.borrow()
    }
}

#[derive(Default)]
pub struct NodeLake {
    pub(crate) data_map: HashMap<NodeKey, NodeData>,
}

impl NodeLake {
    pub(crate) fn remove(&mut self, node_key: &NodeKey) -> Option<NodeData> {
        self.data_map.remove(node_key)
    }

    pub(crate) fn consume_seed_as_linked_node(&mut self, node_seed: NodeSeed) -> (NodeKey, NodeData) {
        let (node_handle, node_raw) = node_seed.split();
        let node_key_arc: NodeKeyArc = node_handle.into();
        let node_key: NodeKey = node_key_arc.into();

        let node_data_pointer = self.entry(node_key.clone()).or_insert(
            NodeDataPoint {
                self_data: node_raw.into(),
                context_holder: Default::default(),
                relations: Default::default(),
            }
            .into(),
        );

        (node_key, node_data_pointer.clone())
    }

    pub(crate) fn get<'a>(&'a self, key: &NodeKey) -> Option<NodeData> {
        self.data_map.get(key).map(|x| x.clone())
    }

    pub(crate) fn entry<'a>(
        &'a mut self,
        node_rcc: NodeKey,
    ) -> std::collections::hash_map::Entry<'a, NodeKey, NodeData> {
        self.data_map.entry(NodeKey::from(node_rcc))
    }

    // pub fn borrow<'a>(&'a self, node_rcc: &'a NodeArc) -> Option<&'a NodeData> {
    //     self.data_map.get(&NodeHashKey::from(node_rcc))
    // }

    // pub fn borrow_mut<'a>(&'a mut self, node_rcc: &'a NodeArc) -> Option<&'a mut NodeData> {
    //     self.data_map.get_mut(&NodeHashKey::from(node_rcc))
    // }

    // pub fn borrow_or_create_mut<'a>(
    //     &'a mut self,
    //     node_hash_key: NodeHashKey,
    // ) -> &'a mut NodeDataContainer {
    //     self.data_map
    //         .entry(node_hash_key)
    //         .or_insert(Default::default())
    // }

    // pub fn borrow_or_create_mut_children_mapping<'a>(
    //     &'a mut self,
    //     node_hash_key: NodeHashKey,
    // ) -> &'a mut Vec<NodeWeak> {
    //     &mut self
    //         .data_map
    //         .entry(node_hash_key)
    //         .or_insert(Default::default())
    //         .children
    // }
}
