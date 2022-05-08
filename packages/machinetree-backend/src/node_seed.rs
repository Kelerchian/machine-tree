use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    embeddable::input_manager::InputManager,
    node::Node,
    typedef::{HeapDataCell, NodeCell, NodeStepFn, WorkerCellRc, WorkerMap},
    worker::{Worker, WorkerSeed},
    WorkItem, WorkItemNotifier, WorkItemSource,
};

/**
 * Used to create a new Node or append param into a Node
 */
pub struct NodeSeed {
    /**
     * Used to match with Node.
     * If a NodeSeed's type_id matches Node's type_id,
     * instead of creating a new Node
     * It appends the param into the Node's param
     */
    pub(crate) type_id: TypeId,
    pub(crate) key: Option<String>,
    pub(crate) input: HeapDataCell,
    pub(crate) generate_workers: Option<Box<dyn FnOnce() -> HashMap<String, WorkerSeed>>>,
    pub(crate) step_fn: Box<NodeStepFn>,
}

impl NodeSeed {
    pub fn create(
        type_id: TypeId,
        key: Option<String>,
        input: HeapDataCell,
        generate_workers: Option<Box<dyn FnOnce() -> HashMap<String, WorkerSeed>>>,
        step_fn: Box<NodeStepFn>,
    ) -> Self {
        Self {
            type_id,
            key,
            input,
            generate_workers,
            step_fn,
        }
    }

    pub(crate) fn try_merge(seed: NodeSeed, node_rc: &mut Rc<NodeCell>) -> Result<(), NodeSeed> {
        let node_borrow = (*node_rc).borrow_mut();
        if seed.type_id == node_borrow.type_id && seed.key == node_borrow.key {
            node_borrow.consume_input(seed.input);
            drop(node_borrow);
            Ok(())
        } else {
            Err(seed)
        }
    }

    pub(crate) fn into_node_cell_rc(
        seed: NodeSeed,
        // TODO: rename, it is ugly
        sender: &crossbeam::channel::Sender<WorkItem>,
    ) -> Rc<RefCell<Node>> {
        let NodeSeed {
            type_id,
            key,
            input,
            generate_workers,
            step_fn,
        }: NodeSeed = seed;

        let node_rc = Rc::new(RefCell::new(Node {
            type_id,
            key: key.clone(),
            state_manager: Default::default(),
            input_manager: RefCell::new(InputManager::new(input)),
            effect_manager: Default::default(),
            workers: Default::default(),
            step_fn,
        }));

        {
            let mut node = node_rc.borrow_mut();

            if let Ok(mut effect_manager_write_guard) = node.effect_manager.write() {
                effect_manager_write_guard.work_item_notifier =
                    Some(WorkItemNotifier::from_work_item_source(
                        WorkItemSource::from(&node_rc),
                        &sender,
                    ));
            }

            {
                let mut input_manager = node.input_manager.borrow_mut();
                (*input_manager).work_item_notifier =
                    Some(WorkItemNotifier::from_work_item_source(
                        WorkItemSource::from(&node_rc),
                        &sender,
                    ));
                // Must be done to run first step
                input_manager.notify_work();
            }

            // Worker assignment
            {
                let new_workers = match generate_workers {
                    Some(generate_workers_fn) => {
                        let workers = generate_workers_fn();
                        Some(workers)
                    }
                    None => None,
                };

                if let Some(new_workers) = new_workers {
                    let worker_map: WorkerMap = new_workers
                        .into_iter()
                        .map(|x| {
                            (x.0, {
                                let worker: Worker = x.1.into();
                                worker.into()
                            })
                        })
                        .collect();
                    node.workers = RefCell::new(worker_map);
                }
            }

            // TODO: error handling for "else" block
            // which is a never scenario
        }

        node_rc
    }
}
