pub mod context_access;
mod lake;
mod render;

use crate::node::{NodeKey, NodeSeed, WorkItem};
use lake::NodeLake;
use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
    vec,
};

use self::{context_access::ContextAccess, render::UnlinkedPair};

pub struct NodeControl<'a> {
    lake: &'a NodeLake,
    current: NodeKey,
    rerender_flag: bool,
}

impl<'a> NodeControl<'a> {
    pub fn rerender(&mut self) -> () {
        self.rerender_flag = true;
    }

    pub fn use_context<'b>(&'a mut self) -> ContextAccess
    where
        'a: 'b,
    {
        ContextAccess {
            lake: &*self.lake,
            node_key_pointer: self.current.clone(),
        }
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

#[derive(Default)]
pub struct RenderReport {
    pub unrendered_keys: Vec<NodeKey>,
    pub rendered_keys: Vec<NodeKey>,
    pub unlinked_node_pairs: Vec<UnlinkedPair>,
}

impl Display for RenderReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rendered_keys_string = self
            .rendered_keys
            .iter()
            .map(|key| key.debug_attempt_get_name())
            .map(|name| format!("\n  - {}", &name))
            .collect::<Vec<_>>()
            .join("");
        let unlinked_keys_string = self
            .unlinked_node_pairs
            .iter()
            .map(|(key, _)| key.debug_attempt_get_name())
            .map(|name| format!("\n  - {}", &name))
            .collect::<Vec<_>>()
            .join("");
        let unrendered_keys_string = self
            .unrendered_keys
            .iter()
            .map(|key| key.debug_attempt_get_name())
            .map(|name| format!("\n  - {}", &name))
            .collect::<Vec<_>>()
            .join("");

        f.write_fmt(format_args!(
            "RenderReport:\n- RenderedKeys:{}\n- UnlinkedKeys:{}\n- UnrenderedKeys:{}",
            &rendered_keys_string, &unlinked_keys_string, &unrendered_keys_string
        ))
    }
}

pub struct ExternalRenderWorkQueue {
    sender: crossbeam::channel::Sender<NodeKey>,
    receiver: crossbeam::channel::Receiver<NodeKey>,
}

impl Default for ExternalRenderWorkQueue {
    fn default() -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded();
        Self { sender, receiver }
    }
}

pub struct NodeHost {
    lake: NodeLake,
    work_queue: VecDeque<WorkItem>,
    external_render_work_queue: ExternalRenderWorkQueue,
}

impl NodeHost {
    pub fn create_with_root(root_node_seed: NodeSeed) -> NodeHost {
        let mut work_queue: VecDeque<WorkItem> = Default::default();
        let mut lake: NodeLake = Default::default();
        let (node_key, _) = lake.consume_seed_as_linked_node(root_node_seed);

        work_queue.push_front(WorkItem::Render(node_key.into()));

        NodeHost {
            lake,
            work_queue,
            external_render_work_queue: Default::default(),
        }
    }

    pub fn render(&mut self) -> RenderReport {
        let work_opt = self.work_queue.pop_front();
        match work_opt {
            Some(work) => match work {
                WorkItem::Render(x) => self.render_node(x),
            },
            None => RenderReport::default(),
        }
    }

    pub fn poll_external_work(&mut self) -> () {
        let sources = {
            let mut sources: VecDeque<_> = vec![].into();
            let mut render_sources_memoizer = HashSet::new();

            while let Ok(key_pointer) = self.external_render_work_queue.receiver.try_recv() {
                let ptr = key_pointer.read_ptr_as_usize();
                if render_sources_memoizer.contains(&ptr) {
                    continue;
                };
                render_sources_memoizer.insert(ptr);

                sources.push_back(key_pointer);
            }
            sources
        };

        let mut sources = sources
            .into_iter()
            .map(|source| WorkItem::Render(source))
            .collect();

        self.work_queue.append(&mut sources);
    }

    fn render_node(&mut self, node_key: NodeKey) -> RenderReport {
        let mut render_report = RenderReport::default();
        let mut next_local_render_queues = VecDeque::from(vec![node_key]);
        let mut next_global_render_queues = VecDeque::new();

        loop {
            let mut now_local_render_queues = VecDeque::new();
            std::mem::swap(&mut next_local_render_queues, &mut now_local_render_queues);

            if now_local_render_queues.len() == 0 {
                break;
            }

            now_local_render_queues.into_iter().for_each(|node_key| {
                use render::*;

                let node_data = match self.lake.get(&node_key) {
                    Some(node_data) => node_data,
                    None => {
                        // Mark unrendered keys as push
                        render_report.unrendered_keys.push(node_key);
                        return;
                    }
                };

                let node_data_point = node_data.borrow_self();

                let RenderResult {
                    new_nodes,
                    unused_nodes,
                    node_control_result,
                } = render(RenderParam {
                    lake: &mut self.lake,
                    external_render_work_queue: &self.external_render_work_queue,
                    node_key: &node_key,
                    node_data_point: &&node_data_point,
                });

                link_children_to_lake(&mut self.lake, &node_key, &node_data_point, &new_nodes);

                // Mark pairs as unlinked
                render_report
                    .unlinked_node_pairs
                    .append(&mut unlink_unused_nodes(&mut self.lake, unused_nodes));

                // Push rerender to workqueue
                if node_control_result.rerender_flag {
                    next_global_render_queues.push_back(WorkItem::Render(node_key.clone()));
                }

                // Append render tasks to queue
                next_local_render_queues.append(&mut new_nodes.into_iter().collect());

                // Mark key as rendered
                render_report.rendered_keys.push(node_key);
            });
        }

        self.work_queue.append(&mut next_global_render_queues);

        render_report
    }
}
