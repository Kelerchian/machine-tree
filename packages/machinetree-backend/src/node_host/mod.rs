mod lake;
use crate::embeddable::context_holder::ContextAccess;
use crate::node::{Control, NodeRcc};
use crate::{node::Node, WorkItem};
use lake::NodeLake;
use std::collections::{HashMap, VecDeque};

// TODO: convert to trait
// Trait Host
// User can implement custom host, but we provide a default host
// 2 built-in host I am thinking about: Sync-blocky and Total-async

pub struct NodeControl<'a> {
    context_access: ContextAccess<'a>,
    rerender_flag: bool,
}

impl<'a> Control<'a> for NodeControl<'a> {
    fn rerender(&mut self) -> () {
        self.rerender_flag = true;
    }

    fn access_context<'f>(&'a mut self) -> &'f mut ContextAccess<'a>
    where
        'a: 'f,
    {
        &mut self.context_access
    }
}
impl From<NodeControl<'_>> for NodeControlResult {
    fn from(control: NodeControl) -> Self {
        Self {
            rerender_flag: control.rerender_flag,
        }
    }
}

pub struct NodeControlResult {
    rerender_flag: bool,
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
    storage: NodeLake,
    work_queue: VecDeque<WorkItem>,
}

impl NodeHost {
    pub fn create_with_root(root_node: Node) -> NodeHost {
        let mut work_queue: VecDeque<WorkItem> = Default::default();
        let storage: NodeLake = Default::default();
        let root: NodeRcc = root_node.into();

        work_queue.push_front(WorkItem {
            kind: crate::WorkItemKind::Render,
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
                crate::WorkItemKind::Render => self.render(&work.source),
            },
            None => StepReport::default(),
        }
    }

    fn render(&mut self, node_rcc: &NodeRcc) -> StepReport {
        let mut render_count = 0_u32;
        let mut next_render_queues = VecDeque::from(vec![node_rcc.clone()]);
        let mut local_work_queue = VecDeque::new();

        loop {
            let mut local_render_queues = VecDeque::new();
            std::mem::swap(&mut next_render_queues, &mut local_render_queues);

            if local_render_queues.len() == 0 {
                break;
            }

            local_render_queues
                .into_iter()
                .for_each(|currently_rendered_node_rcc| {
                    let mut trashed_nodes: Vec<NodeRcc> = Default::default();
                    let children = self
                        .storage
                        .mutate_children_mapping(&currently_rendered_node_rcc);

                    let (node_control_result, new_nodes) = {
                        let mut node_borrow_refmut = currently_rendered_node_rcc.borrow_mut();
                        let node_borrow = &mut *node_borrow_refmut;
                        let step_fn_borrow = &mut *(node_borrow.step_fn.borrow_mut());
                        let context_holder_borrow = &mut node_borrow.context_holder;

                        let mut control = NodeControl {
                            context_access: ContextAccess {
                                context_holder_ref: context_holder_borrow,
                            },
                            rerender_flag: false,
                        };

                        let children_lookup_map: HashMap<String, &NodeRcc> = children
                            .iter()
                            .map(|child| (child.borrow().key.clone(), child))
                            .collect();

                        // execute_step_fn
                        let new_nodes: Vec<NodeRcc> =
                            (step_fn_borrow)(&mut control, &node_borrow.input)
                                .into_iter()
                                .map(|new_child_node| -> NodeRcc {
                                    let old_child = children_lookup_map.get(&new_child_node.key);
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
                                                trashed_nodes.push((*old_node_rcc).clone());
                                                new_child_node.into()
                                            } else {
                                                let mut old_node_borrow = old_node_rcc.borrow_mut();
                                                old_node_borrow.input =
                                                    new_child_node.clone_input();
                                                (*old_node_rcc).clone()
                                            }
                                        }
                                    }
                                })
                                .collect();

                        let result: NodeControlResult = control.into();
                        (result, new_nodes)
                    };

                    // Append render tasks to queue
                    next_render_queues.append(&mut VecDeque::from(new_nodes.clone()));

                    // assign children
                    *children = new_nodes;

                    // create reverse children mapping
                    self.storage
                        .generate_parent_link_for_children(&currently_rendered_node_rcc);

                    // clean up trash
                    trashed_nodes.iter().for_each(|trashed_node_rcc| {
                        self.storage.unlink_recursively(trashed_node_rcc);
                    });

                    // push rerender to workqueue
                    if node_control_result.rerender_flag {
                        local_work_queue.push_back(WorkItem {
                            kind: crate::WorkItemKind::Render,
                            source: currently_rendered_node_rcc.clone(),
                        });
                    }

                    render_count = match render_count.checked_add(1) {
                        Some(x) => render_count + x,
                        None => render_count,
                    }
                });
        }

        self.work_queue.append(&mut local_work_queue);

        StepReport { render_count }
    }
}
