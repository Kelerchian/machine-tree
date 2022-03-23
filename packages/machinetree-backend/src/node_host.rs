use crate::{
    node_seed::NodeSeed, typedef::NodeCell, NodeHashKey, WorkItem, WorkItemKind, WorkItemSource,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

// TODO: convert to trait
// Trait Host
// User can implement custom host, but we provide a default host
// 2 built-in host I am thinking about: Sync-blocky and Total-async

pub struct HostWorkQueue {
    queue: VecDeque<WorkItem>,
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
        let work_queue: VecDeque<WorkItem> = Default::default();
        let work_channels = crossbeam::channel::unbounded();

        let root_node = {
            let sender_ref = &work_channels.0;
            NodeSeed::into_node_cell_rc(seed, sender_ref)
        };

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
                WorkItem {
                    kind: WorkItemKind::Step,
                    source: WorkItemSource::Node(node),
                } => {
                    // Ignore nodes that have been destroyed
                    if let Some(children) = self.child_map.get_mut(&NodeHashKey::from(&node)) {
                        let seeds_res = {
                            let mut borrowed = (*node).borrow_mut();
                            borrowed.run()
                        };

                        match seeds_res {
                            Ok(seeds) => {
                                let mut destroyable_node_rc_set: HashSet<NodeHashKey> =
                                    Default::default();
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
                            Err(_) => {
                                // TODO: handle runtime errors
                            }
                        }
                    }
                }
                WorkItem {
                    kind: WorkItemKind::Effect,
                    source: WorkItemSource::Node(node),
                } => {
                    // DO NOTHING
                }
                WorkItem {
                    kind: WorkItemKind::Step,
                    source: WorkItemSource::Worker(node, worker),
                } => {
                    let worker = worker.borrow_mut();
                    match worker.run() {
                        Ok(x) => {}
                        Err(x) => {}
                    }
                }
                WorkItem {
                    kind: WorkItemKind::Effect,
                    source: WorkItemSource::Worker(node, worker),
                } => {
                    let worker = worker.borrow_mut();
                    let effect_manager = &worker.effect_manager;
                    let effect_manager_write_lock = effect_manager.write();
                    match effect_manager_write_lock {
                        Ok(mut effect) => {
                            effect.run_all();
                        }
                        Err(_) => {
                            // TODO:
                        }
                    }
                    // run effects
                    // notify node
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
