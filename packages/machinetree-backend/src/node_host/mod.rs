mod storage;
use crate::node::{Control, NodeRcc};
use crate::{node::Node, WorkItem};
use std::collections::{HashMap, VecDeque};
use storage::NodeStorage;

// TODO: convert to trait
// Trait Host
// User can implement custom host, but we provide a default host
// 2 built-in host I am thinking about: Sync-blocky and Total-async

pub struct NodeControl {
    rerender_flag: bool,
}
impl Control for NodeControl {
    fn rerender(&mut self) -> () {
        self.rerender_flag = true;
    }
}

impl Default for NodeControl {
    fn default() -> Self {
        Self {
            rerender_flag: false,
        }
    }
}

pub struct StepReport {
    pub render_count: u32,
}

impl Default for StepReport {
    fn default() -> Self {
        Self {
            render_count: Default::default(),
        }
    }
}

pub struct NodeHost {
    root: NodeRcc,
    storage: NodeStorage,
    work_queue: VecDeque<WorkItem>,
}

impl NodeHost {
    pub fn create_with_root(root_node: Node) -> NodeHost {
        let mut work_queue: VecDeque<WorkItem> = Default::default();
        let mut storage: NodeStorage = Default::default();
        let root: NodeRcc = root_node.into();

        storage.insert(&root);

        work_queue.push_front(WorkItem {
            kind: crate::WorkItemKind::Rerender,
            source: root.clone(),
        });

        NodeHost {
            root,
            storage,
            work_queue,
        }
    }

    // TODO: change to work report
    pub fn step(&mut self) -> StepReport {
        let work_opt = self.work_queue.pop_front();
        match &work_opt {
            Some(work) => match work.kind {
                crate::WorkItemKind::Rerender => self.rerender(&work.source),
            },
            None => StepReport::default(),
        }
    }

    fn rerender(&mut self, node_rcc: &NodeRcc) -> StepReport {
        let mut render_count = 0_u32;
        let mut renderables_in_current_cycle = VecDeque::from(vec![node_rcc.clone()]);
        let mut local_work_queue = VecDeque::new();

        loop {
            let renderable = renderables_in_current_cycle.pop_back();
            match renderable {
                None => break,
                Some(currently_rendered_node_rcc) => {
                    let mut control = NodeControl::default();
                    let mut trashed_nodes: Vec<NodeRcc> = Default::default();
                    let children = self
                        .storage
                        .borrow_children_mapping(&currently_rendered_node_rcc);
                    {
                        let node_borrow = currently_rendered_node_rcc.borrow_mut();
                        let mut step_fn_borrow = node_borrow.step_fn.borrow_mut();

                        let cached_children_map = {
                            let mut children_map: HashMap<String, NodeRcc> = HashMap::new();
                            children.iter().for_each(|child| {
                                let key = child.borrow().key.clone();
                                children_map.insert(key, child.clone());
                            });
                            children_map
                        };

                        // execute_step_fn
                        let new_nodes: Vec<NodeRcc> =
                            (*step_fn_borrow)(&mut control, &node_borrow.input)
                                .into_iter()
                                .map(|new_child_node| -> NodeRcc {
                                    let old_child = cached_children_map.get(&new_child_node.key);
                                    match old_child {
                                        None => new_child_node.into(),
                                        Some(old_node_rcc) => {
                                            if {
                                                let old_node_borrow = old_node_rcc.borrow();
                                                !Node::equal_as_node_reference(
                                                    &new_child_node,
                                                    &old_node_borrow,
                                                )
                                            } {
                                                trashed_nodes.push(old_node_rcc.clone());
                                                new_child_node.into()
                                            } else {
                                                let mut old_node_borrow = old_node_rcc.borrow_mut();
                                                old_node_borrow.input =
                                                    new_child_node.inherit_input();
                                                old_node_rcc.clone()
                                            }
                                        }
                                    }
                                })
                                .collect();

                        // append  to renderables
                        renderables_in_current_cycle.append(&mut VecDeque::from(new_nodes.clone()));

                        // mutate children
                        *children = new_nodes;

                        // End of render lifetime
                        // Borrow mut is dropped here
                    }

                    // clean up trash
                    trashed_nodes.iter().for_each(|trashed_node_rcc| {
                        self.storage.unlink_recursively(trashed_node_rcc);
                    });

                    if control.rerender_flag {
                        local_work_queue.push_back(WorkItem {
                            kind: crate::WorkItemKind::Rerender,
                            source: currently_rendered_node_rcc.clone(),
                        });
                    }

                    render_count = match render_count.checked_add(1) {
                        Some(x) => render_count + x,
                        None => render_count,
                    }
                }
            }
        }

        self.work_queue.append(&mut local_work_queue);

        StepReport { render_count }
    }
}
