pub mod embeddable;
pub mod node;
pub mod node_host;
pub mod node_seed;
pub mod typedef;
pub mod worker;

use typedef::*;
use worker::Worker;

use std::{hash::Hash, rc::Rc};

pub struct NodeHashKey(Rc<NodeCell>);

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

impl From<&Rc<NodeCell>> for NodeHashKey {
    fn from(node_rc: &Rc<NodeCell>) -> Self {
        NodeHashKey(node_rc.clone())
    }
}

// TODO: create many kinds of workitem
// Unique workitem
// For Node:
//  Input workitem (which is unique and precede effect workitem in a batch)
//  Effect workitem (which is unique)
//  Rerender workitem (which is unique and succeed effect workitem in a batch)
// Or should we just make WorkItem a trait as well? So we can just ... Ok, WorkItem is part of Node anyway so yeah Trait
// But we should add more info, for example, is WorkItem sent from Effect? Input? Rerender?
// Oh wait, Input and Rerender should trigger step_fn while Effect should trigger effects to be done.

// REVISE ABOVE:
// WorkItem is always FromInput FromEffect.
// Both triggers rerender.
// In addition, FromEffect investigate worker effect queue and swap it (and occasionally generate FromEffect)
// worker effect investigation and swaps are done before the rerender

#[derive(Clone)]
pub(crate) struct WorkItem {
    kind: WorkItemKind,
    source: WorkItemSource,
}

#[derive(Clone)]
pub(crate) enum WorkItemSource {
    Node(NodeCellRc),
    Worker(NodeCellRc, WorkerCellRc),
}

impl From<&NodeCellRc> for WorkItemSource {
    fn from(node_rc: &NodeCellRc) -> Self {
        WorkItemSource::Node(node_rc.clone())
    }
}

#[derive(Clone)]
pub(crate) enum WorkItemKind {
    Step,
    Effect,
}

pub(crate) struct WorkItemNotifier {
    source: WorkItemSource,
    sender: crossbeam::channel::Sender<WorkItem>,
}

impl WorkItemNotifier {
    pub(crate) fn notify(&self, kind: WorkItemKind) {
        if let Err(error) = self.sender.send(WorkItem {
            kind,
            source: self.source.clone(),
        }) {
            eprintln!("{:?}", error);
        }
    }

    pub(crate) fn from_work_item_source(
        source: WorkItemSource,
        sender: &crossbeam::channel::Sender<WorkItem>,
    ) -> Self {
        Self {
            source,
            sender: sender.clone(),
        }
    }
}
