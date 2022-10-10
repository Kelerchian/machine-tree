pub mod embeddable;
pub mod node;
pub mod node_host;

use node::{NodeC, NodeRcc};

use std::{hash::Hash, rc::Rc};

pub struct NodeHashKey(Rc<NodeC>);

impl Hash for NodeHashKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let borrowed = &*self.0.borrow();
        std::ptr::hash(borrowed, state);
    }
}

impl PartialEq for NodeHashKey {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NodeHashKey {}

impl From<&Rc<NodeC>> for NodeHashKey {
    fn from(node_rc: &Rc<NodeC>) -> Self {
        NodeHashKey(node_rc.clone())
    }
}

#[derive(Clone)]
pub(crate) struct WorkItem {
    kind: WorkItemKind,
    source: NodeRcc,
}

#[derive(Clone)]
pub(crate) enum WorkItemKind {
    Render,
}
