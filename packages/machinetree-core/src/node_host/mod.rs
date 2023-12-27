pub mod context_access;
mod lake;
mod render;

use crate::{
    key::{Key, Seed},
    node::WorkItem,
};
use lake::NodeLake;
use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
    vec,
};

use self::{context_access::ContextAccess, render::UnlinkedPair};

pub struct NodeControl<'a> {
    lake: &'a NodeLake,
    current: Key,
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
    pub unrendered_keys: Vec<Key>,
    pub rendered_keys: Vec<Key>,
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
    sender: crossbeam::channel::Sender<Key>,
    receiver: crossbeam::channel::Receiver<Key>,
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
    pub fn make_root(seed: Seed) -> NodeHost {
        let mut queue: VecDeque<WorkItem> = Default::default();
        let mut lake: NodeLake = Default::default();
        let (node_key, _) = lake.sprout_and_link(seed);

        queue.push_front(WorkItem::Render(node_key.into()));

        NodeHost {
            lake,
            work_queue: queue,
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

    pub fn poll_work(&mut self) -> () {
        let sources = {
            let mut sources: VecDeque<_> = vec![].into();
            let mut memo = HashSet::new();

            while let Ok(key) = self.external_render_work_queue.receiver.try_recv() {
                let ptr = key.read_ptr_as_usize();
                if !memo.contains(&ptr) {
                    memo.insert(ptr);
                    sources.push_back(key);
                };
            }
            sources
        };

        self.work_queue
            .extend(&mut sources.into_iter().map(|source| WorkItem::Render(source)));
    }

    fn render_node(&mut self, node_key: Key) -> RenderReport {
        let mut report = RenderReport::default();
        let mut next_local_queue = VecDeque::from(vec![node_key]);
        let mut next_global_queue = VecDeque::new();

        loop {
            let mut now_local_render_queues = VecDeque::new();
            std::mem::swap(&mut next_local_queue, &mut now_local_render_queues);

            if now_local_render_queues.len() == 0 {
                break;
            }

            now_local_render_queues.into_iter().for_each(|node_key| {
                use render::*;

                let node_data = match self.lake.get(&node_key) {
                    None => {
                        report.unrendered_keys.push(node_key);
                        return;
                    }
                    Some(node_data) => node_data,
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
                report
                    .unlinked_node_pairs
                    .append(&mut unlink_unused_nodes(&mut self.lake, unused_nodes));

                // Push rerender to workqueue
                if node_control_result.rerender_flag {
                    next_global_queue.push_back(WorkItem::Render(node_key.clone()));
                }

                // Append render tasks to queue
                next_local_queue.append(&mut new_nodes.into_iter().collect());

                // Mark key as rendered
                report.rendered_keys.push(node_key);
            });
        }

        self.work_queue.append(&mut next_global_queue);

        report
    }
}
