pub mod input_manager;
pub mod node;
pub mod patch_manager;
pub mod typedef;
pub mod worker;

use input_manager::{InputManager, InputManagerBridge};
use node::NodeSeed;
use patch_manager::PatchManager;
use typedef::*;
use worker::Worker;

use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    rc::Rc,
    vec,
};

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

#[derive(Clone)]
pub(crate) enum WorkItem {
    Node(Rc<NodeCell>),
    Worker(Rc<NodeCell>, Rc<Worker>),
}

impl From<&Rc<NodeCell>> for WorkItem {
    fn from(node: &Rc<NodeCell>) -> Self {
        WorkItem::Node(Rc::clone(node))
    }
}

pub(crate) struct WorkItemNotifier {
    work_item: WorkItem,
    sender: crossbeam::channel::Sender<WorkItem>,
}

impl WorkItemNotifier {
    pub(crate) fn notify(&self) {
        if let Err(error) = self.sender.send(self.work_item.clone()) {
            eprintln!("{:?}", error);
        }
    }

    pub(crate) fn from_work_item(
        work_item: WorkItem,
        sender: &crossbeam::channel::Sender<WorkItem>,
    ) -> Self {
        Self {
            work_item,
            sender: sender.clone(),
        }
    }
}

pub struct NodeHost {
    // nodes: Vec<Weak<NodeCell>>,
    root: Rc<NodeCell>,
    child_map: HashMap<NodeHashKey, Vec<Rc<NodeCell>>>,
    pub(crate) work_channels: (
        crossbeam::channel::Sender<WorkItem>,
        crossbeam::channel::Receiver<WorkItem>,
    ),
    work_queue: VecDeque<WorkItem>,
}

impl NodeHost {
    pub fn create_root(seed: NodeSeed) -> NodeHost {
        let work_channels = crossbeam::channel::unbounded();

        let root_node = {
            let sender_ref = &work_channels.0;
            NodeSeed::into_node_cell_rc(seed, sender_ref)
        };

        let mut work_queue: VecDeque<WorkItem> = Default::default();
        work_queue.push_back(WorkItem::from(&root_node));

        let mut child_map: HashMap<NodeHashKey, Vec<Rc<NodeCell>>> = Default::default();
        child_map.insert(NodeHashKey::from(&root_node), Default::default());

        NodeHost {
            root: root_node,
            child_map,
            work_channels,
            work_queue,
        }
    }

    pub fn receive_work(&mut self) -> Result<(), crossbeam::channel::TryRecvError> {
        let receiver = &self.work_channels.1;
        let work_item = receiver.try_recv()?;
        self.work_queue.push_front(work_item);
        Ok(())
    }

    pub fn run_work(&mut self) -> bool {
        if let Some(work) = self.work_queue.pop_front() {
            match work {
                WorkItem::Node(node) => {
                    // Ignore nodes that have been destroyed
                    if let Some(children) = self.child_map.get_mut(&NodeHashKey::from(&node)) {
                        let seeds = {
                            let mut borrowed = (*node).borrow_mut();
                            borrowed.run()
                        };

                        let mut destroyable_node_rc_set: HashSet<NodeHashKey> = Default::default();
                        let mut insertable_hash_keys: Vec<NodeHashKey> = Default::default();

                        // Replace children value
                        (*children) = seeds
                            .into_iter()
                            .enumerate()
                            .map(|(index, seed)| -> Rc<NodeCell> {
                                let host_sender = &self.work_channels.0;
                                let child = match children.get_mut(index) {
                                    None => NodeSeed::into_node_cell_rc(seed, host_sender),
                                    Some(child) => match NodeSeed::try_merge(seed, child) {
                                        Err(seed) => {
                                            destroyable_node_rc_set
                                                .insert(NodeHashKey::from(&*child));
                                            NodeSeed::into_node_cell_rc(seed, host_sender)
                                        }
                                        // Optimize
                                        Ok(_) => child.clone(),
                                    },
                                };

                                insertable_hash_keys.push(NodeHashKey::from(&child));

                                child
                            })
                            .collect();

                        // For removed nodes: remove them from being HashMap keys
                        destroyable_node_rc_set.iter().for_each(|item| {
                            self.remove_node_recursively(item);
                        });

                        insertable_hash_keys
                            .into_iter()
                            .for_each(|item: NodeHashKey| {
                                if let None = self.child_map.get(&item) {
                                    self.child_map.insert(item, vec![]);
                                }
                            });
                    }
                }
                WorkItem::Worker(_, _) => {
                    println!("worker run triggered");
                }
            }
            true
        } else {
            false
        }
    }

    fn remove_node_recursively(&mut self, key: &NodeHashKey) {
        // Retrieve children as NodeHashKey
        let children_keys_option: Option<Vec<NodeHashKey>> = match self.child_map.get(key) {
            Some(children) => Some(
                children
                    .iter()
                    .map(|node_rc| -> NodeHashKey { NodeHashKey::from(&*node_rc) })
                    .collect(),
            ),
            _ => None,
        };

        // Wipe children recursively
        if let Some(keys) = children_keys_option {
            keys.iter().for_each(|c| {
                self.remove_node_recursively(c);
            });
        }

        self.child_map.remove(key);
    }
}

// Initialize

// 1. Create Node and bind to Root
// 2. Run host worker

// Host Worker

// 1. For each node, recursively from root, create_patch
// 2. Run patch (which will queue more patches)
// 3. Repeat

// Run Patch
// 1. Read dependencies (props, Context (implemented later))
// 2. (Optional) queue for more patches
// 3. Determine and prune children

// Children Determination, memoiozation?
// 1. pikir nanti
