use std::collections::{HashMap, HashSet};

use super::{
    lake::{NodeData, NodeDataPoint, NodeLake},
    ExternalRenderWorkQueue, NodeControl, NodeControlResult,
};
use crate::node::{NodeKey, NodeSeed};

pub(crate) struct RenderParam<'a> {
    pub(crate) lake: &'a mut NodeLake,
    pub(crate) external_render_work_queue: &'a ExternalRenderWorkQueue,
    pub(crate) node_key: &'a NodeKey,
    pub(crate) node_data_point: &'a NodeDataPoint,
}

pub(crate) struct RenderResult {
    pub(crate) new_nodes: Vec<NodeKey>,
    pub(crate) unused_nodes: Vec<NodeKey>,
    pub(crate) node_control_result: NodeControlResult,
}

pub(crate) fn render(param: RenderParam) -> RenderResult {
    let RenderParam {
        lake,
        external_render_work_queue,
        node_key,
        node_data_point,
    } = param;

    let StepResult {
        new_seeds,
        node_control_result,
    } = run_step_fn(StepParam {
        lake,
        node_key,
        node_data_point,
    });

    let ReconciliationResult {
        new_nodes,
        unused_nodes,
    } = reconcile(ReconciliationParam {
        lake,
        external_render_work_queue,
        node_data_point,
        new_seeds,
    });

    RenderResult {
        new_nodes,
        unused_nodes,
        node_control_result,
    }
}

struct StepParam<'a> {
    pub(crate) lake: &'a NodeLake,
    pub(crate) node_key: &'a NodeKey,
    pub(crate) node_data_point: &'a NodeDataPoint,
}
struct StepResult {
    pub(crate) new_seeds: Vec<NodeSeed>,
    pub(crate) node_control_result: NodeControlResult,
}

fn run_step_fn(param: StepParam) -> StepResult {
    let StepParam {
        lake,
        node_key,
        node_data_point,
    } = param;
    let node_data_borrow = node_data_point.borrow_data_mut();
    let mut step_fn_borrow = node_data_borrow.step_fn.borrow_mut();

    let mut control = NodeControl {
        rerender_flag: false,
        current: node_key.clone(),
        lake: &lake,
    };

    let produced_nodes = (step_fn_borrow)(&mut control, &node_data_borrow.input);
    let node_control_result: NodeControlResult = control.into();

    StepResult {
        new_seeds: produced_nodes,
        node_control_result,
    }
}

struct ReconciliationParam<'a> {
    pub(crate) lake: &'a mut NodeLake,
    pub(crate) external_render_work_queue: &'a ExternalRenderWorkQueue,
    pub(crate) node_data_point: &'a NodeDataPoint,
    pub(crate) new_seeds: Vec<NodeSeed>,
}

struct ReconciliationResult {
    pub(crate) new_nodes: Vec<NodeKey>,
    pub(crate) unused_nodes: Vec<NodeKey>,
}

fn reconcile<'a>(param: ReconciliationParam) -> ReconciliationResult {
    let ReconciliationParam {
        lake,
        external_render_work_queue,
        node_data_point,
        new_seeds,
    } = param;

    let children_ref_mut = &mut node_data_point.borrow_mut_relations().children;

    // Inquire trashed nodes
    let mut unused_nodes: HashSet<NodeKey> = {
        // HashMap mapping new seeds by its key
        let new_seed_lookup_map: HashMap<&String, &NodeSeed> =
            new_seeds.iter().map(|seed| (&seed.key, seed)).collect();

        let unused_nodes = children_ref_mut
            .iter()
            .filter_map(|child| child.upgrade())
            .filter_map(|child| -> Option<NodeKey> {
                let matching_seed_found = child.lock().map_or(false, |child_key_raw| {
                    new_seed_lookup_map
                        .get(&child_key_raw.key)
                        .map_or(false, |new_seed| new_seed.type_id == child_key_raw.type_id)
                });

                if !matching_seed_found {
                    Some(child.into())
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

        unused_nodes
    };

    let new_nodes: Vec<NodeKey> = {
        let mut old_children_lookup_map: HashMap<String, NodeKey> = children_ref_mut
            .iter()
            .filter_map(|child| -> Option<NodeKey> { child.try_into().ok() })
            .filter_map(|child| -> Option<(String, NodeKey)> {
                let key = child.0.try_lock().ok()?.key.clone();
                Some((key, child.into()))
            })
            .collect();

        new_seeds
            .into_iter()
            .map(|new_seed| -> NodeKey {
                let mut unused_old_key_opt = None;

                // Find old key, reuse if possible, mark as unused if not
                if let Some(old_key) = old_children_lookup_map.remove(&new_seed.key) {
                    if let Ok(_) = merge_seed_to_nodekey(lake, &new_seed, &old_key) {
                        // Old_child is reusable
                        return old_key;
                    }

                    // Set trashed_old_child_arc
                    unused_old_key_opt = Some(old_key);
                }

                // Past this point, use new seed to create a new node

                if let Some(unused_old_key) = unused_old_key_opt {
                    unused_nodes.insert(unused_old_key.clone());
                }

                // Consume seed into lake, and get the linked nodekey
                let (node_key, _) = lake.consume_seed_as_linked_node(new_seed);

                // TODO: handle deadlock
                // Set self-signal on a new node_key
                if let Ok(mut node_key_raw) = node_key.lock() {
                    node_key_raw.self_render_signaler.set_self(
                        &node_key.clone(),
                        &external_render_work_queue.sender.clone(),
                    );
                };

                node_key
            })
            .collect()
    };

    ReconciliationResult {
        new_nodes,
        unused_nodes: unused_nodes.into_iter().collect(),
    }
}

fn merge_seed_to_nodekey(
    lake: &mut NodeLake,
    new_seed: &NodeSeed,
    node_key: &NodeKey,
) -> Result<(), ()> {
    // TODO: handle deadlocks
    // Don't merge if node_key.lock() fails
    let old_child_handle = node_key.lock().map_err(|_| Default::default())?;

    // Don't if type_id is different
    if old_child_handle.type_id != new_seed.type_id {
        return Err(Default::default());
    }

    // Don't merge if lake.get fails
    let node_data = lake.get(&node_key).ok_or_else(|| Default::default())?;

    let node_data_point = node_data.borrow_self();
    let mut node_raw_data = node_data_point.borrow_data_mut();
    node_raw_data.input = new_seed.clone_input();

    return Ok(());
}

pub(crate) fn link_children_to_lake<'a>(
    lake: &'a mut NodeLake,
    node_key: &NodeKey,
    node_data_point: &'a NodeDataPoint,
    new_nodes: &Vec<NodeKey>,
) {
    // Assign children weak nodes
    node_data_point.borrow_mut_relations().children =
        new_nodes.iter().map(|child_key| child_key.into()).collect();

    // Set children's parent to node_key
    new_nodes.iter().for_each(|child_key| {
        lake.get(child_key).map(|child_data_pointer| {
            child_data_pointer
                .borrow_self()
                .borrow_mut_relations()
                .parent = Some(node_key.into())
        });
    });
}

pub type UnlinkedPair = (NodeKey, NodeData);

pub(crate) fn unlink_unused_nodes<'a>(
    lake: &'a mut NodeLake,
    unused_nodes: Vec<NodeKey>,
) -> Vec<UnlinkedPair> {
    unused_nodes
        .into_iter()
        .map(|node_key| unlink_recursively(lake, node_key))
        .fold(vec![], |mut a, mut b| {
            a.append(&mut b);
            a
        })
}

fn unlink_recursively<'a>(
    lake: &'a mut NodeLake,
    into_nodeshell: impl TryInto<NodeKey>,
) -> Vec<UnlinkedPair> {
    into_nodeshell
        .try_into()
        .map_or(Default::default(), |node_key| {
            lake.remove(&node_key)
                .map_or(Default::default(), |removed| {
                    let removed_children = removed
                        .borrow_self()
                        .borrow_relations()
                        .children
                        .iter()
                        .filter_map(|node_key_weak| node_key_weak.upgrade())
                        .map(|child_key_raw| unlink_recursively(lake, child_key_raw))
                        .collect::<Vec<_>>();

                    let unlinked_pairs = removed_children.into_iter().fold(
                        vec![(node_key.clone(), removed)],
                        |mut all_unlinked_pairs, mut unlinked_pairs| {
                            all_unlinked_pairs.append(&mut unlinked_pairs);
                            all_unlinked_pairs
                        },
                    );

                    unlinked_pairs
                })
        })
}
