use std::collections::{HashMap, HashSet};

use crate::key::{Key, Seed};

use super::{
    lake::{NodeData, NodeDataPoint, NodeLake},
    ExternalRenderWorkQueue, NodeControl, NodeControlResult,
};

pub(crate) struct RenderParam<'a> {
    pub(crate) lake: &'a mut NodeLake,
    pub(crate) external_render_work_queue: &'a ExternalRenderWorkQueue,
    pub(crate) node_key: &'a Key,
    pub(crate) node_data_point: &'a NodeDataPoint,
}

pub(crate) struct RenderResult {
    pub(crate) new_nodes: Vec<Key>,
    pub(crate) unused_nodes: Vec<Key>,
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
    pub(crate) node_key: &'a Key,
    pub(crate) node_data_point: &'a NodeDataPoint,
}
struct StepResult {
    pub(crate) new_seeds: Vec<Seed>,
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
    pub(crate) new_seeds: Vec<Seed>,
}

struct ReconciliationResult {
    pub(crate) new_nodes: Vec<Key>,
    pub(crate) unused_nodes: Vec<Key>,
}

fn reconcile<'a>(
    ReconciliationParam {
        lake,
        external_render_work_queue,
        node_data_point,
        new_seeds,
    }: ReconciliationParam,
) -> ReconciliationResult {
    let children = &mut node_data_point.borrow_mut_relations().children;

    let supposed_new_order: HashMap<_, _> = new_seeds.iter().map(|x| (&x.key, &x)).collect();

    // Set aside keyed

    // Inquire trashed nodes
    let mut unused_node_keys: HashSet<Key> = {
        // HashMap mapping new seeds by its key
        let new_seed_lookup_map: HashMap<&String, &Seed> =
            new_seeds.iter().map(|seed| (&seed.key.key, seed)).collect();

        let unused_nodes = children
            .iter()
            .filter_map(|child| child.upgrade())
            .filter_map(|child| child.lock().ok())
            .filter_map(|child| -> Option<Key> {
                let is_a_match = child.lock().map_or(false, |child_key| {
                    new_seed_lookup_map
                        .get(&child_key.key)
                        .map_or(false, |new_seed| new_seed.key.type_id == child_key.type_id)
                });

                if !is_a_match {
                    Some(child.into())
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

        unused_nodes
    };

    let new_node_keys: Vec<Key> = {
        let mut old_children_lookup_map: HashMap<String, Key> = children
            .iter()
            .filter_map(|child| -> Option<Key> { child.try_into().ok() })
            .filter_map(|child| -> Option<(String, Key)> {
                let key = child.0.try_lock().ok()?.key.clone();
                Some((key, child.into()))
            })
            .collect();

        new_seeds
            .into_iter()
            .map(|new_seed| -> Key {
                let mut unused_old_key_opt = None;

                // Find old key, reuse if possible, mark as unused if not
                if let Some(old_key) = old_children_lookup_map.remove(&new_seed.key.key) {
                    if let Ok(_) = merge_seed_to_nodekey(lake, &new_seed, &old_key) {
                        // Old_child is reusable
                        return old_key;
                    }

                    // Set trashed_old_child_arc
                    unused_old_key_opt = Some(old_key);
                }

                // Past this point, use new seed to create a new node

                if let Some(unused_old_key) = unused_old_key_opt {
                    unused_node_keys.insert(unused_old_key.clone());
                }

                // Consume seed into lake, and get the linked nodekey
                let (node_key, _) = lake.sprout_and_link(new_seed);

                // TODO: handle deadlock
                // Set self-signal on a new node_key
                if let Ok(mut node_key_raw) = node_key.lock() {
                    node_key_raw.self_render.set_self(
                        &node_key.clone(),
                        &external_render_work_queue.sender.clone(),
                    );
                };

                node_key
            })
            .collect()
    };

    ReconciliationResult {
        new_nodes: new_node_keys,
        unused_nodes: unused_node_keys.into_iter().collect(),
    }
}

fn merge_seed_to_nodekey(lake: &mut NodeLake, new_seed: &Seed, node_key: &Key) -> Result<(), ()> {
    // TODO: handle deadlocks
    // Don't merge if node_key.lock() fails
    let old_child_handle = node_key.lock().map_err(|_| Default::default())?;

    // Don't if type_id is different
    if old_child_handle.type_id != new_seed.key.type_id {
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
    node_key: &Key,
    node_data_point: &'a NodeDataPoint,
    new_nodes: &Vec<Key>,
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

pub type UnlinkedPair = (Key, NodeData);

pub(crate) fn unlink_unused_nodes<'a>(
    lake: &'a mut NodeLake,
    unused_nodes: Vec<Key>,
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
    into_nodeshell: impl TryInto<Key>,
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
