use crate::{
    embeddable::context_holder::ContextHolder,
    key::{Key, KeyArc, KeyWeak, RawData, Seed},
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

#[derive(Default)]
pub struct NodeRelations {
    pub(crate) children: Vec<KeyWeak>,
    pub(crate) parent: Option<KeyWeak>,
}

pub struct NodeDataPoint {
    pub(crate) self_data: Rc<RefCell<RawData>>,
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

    pub(crate) fn borrow_data_mut<'a>(&'a self) -> RefMut<RawData> {
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
    pub(crate) data_map: HashMap<Key, NodeData>,
}

impl NodeLake {
    pub(crate) fn remove(&mut self, node_key: &Key) -> Option<NodeData> {
        self.data_map.remove(node_key)
    }

    pub(crate) fn sprout_and_link(&mut self, node_seed: Seed) -> (Key, NodeData) {
        let (raw_key, raw_data) = node_seed.sprout();
        let node_key_arc: KeyArc = raw_key.into();
        let node_key = Key::new_from_raw(raw_key);

        let node_data_pointer = self.entry(node_key.clone()).or_insert(
            NodeDataPoint {
                self_data: raw_data.into(),
                context_holder: Default::default(),
                relations: Default::default(),
            }
            .into(),
        );

        (node_key, node_data_pointer.clone())
    }

    pub(crate) fn get<'a>(&'a self, key: &Key) -> Option<NodeData> {
        self.data_map.get(key).map(|x| x.clone())
    }

    pub(crate) fn entry<'a>(
        &'a mut self,
        node_rcc: Key,
    ) -> std::collections::hash_map::Entry<'a, Key, NodeData> {
        self.data_map.entry(Key::from(node_rcc))
    }
}
