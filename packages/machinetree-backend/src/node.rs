use crate::{
    input_manager,
    typedef::{HeapDataCell, NodeCell, NodeStepFn},
    worker::{Worker, WorkerOperationBridge},
    InputManager, InputManagerBridge, PatchManager, WorkItem, WorkItemNotifier,
};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

// TODO: explain capabilities
pub struct NodeOperationBridge<'a> {
    node: &'a Node,
}

impl<'a> NodeOperationBridge<'a> {
    fn use_worker<ReturnType>(
        &'a self,
        id: &String,
        operation_fn: Box<dyn Fn(&mut WorkerOperationBridge) -> ReturnType>,
    ) -> Option<ReturnType> {
        let workers = self.node.workers.borrow_mut();
        if let Some(worker) = (workers).get(id) {
            let return_value = {
                let borrowed = (worker).borrow_mut();
                let mut operation_bridge = WorkerOperationBridge::from(&*borrowed);
                let return_value = operation_fn(&mut operation_bridge);

                // TODO: signal to patch manager, or write it in worker.rs instead

                return_value
            };
            Some(return_value)
        } else {
            None
        }
    }
}

pub struct Node {
    /**
     * Used to match with NodeSeed
     */
    type_id: TypeId,
    key: Option<String>,
    input_manager: RefCell<InputManager>,
    patch_manager: Arc<RwLock<PatchManager>>,
    workers: RefCell<HashMap<String, Rc<RefCell<Worker>>>>,
    step_fn: Box<NodeStepFn>,
}

impl Node {
    pub fn swap_patches(&self) {
        match self.patch_manager.write() {
            Ok(mut patch_manager) => {
                (*patch_manager).swap_patch();
            }
            Err(_poison_error) => {
                // TODO: tell the host there's a poison error
            }
        };
    }

    pub fn run<'a>(&'a mut self) -> Vec<NodeSeed> {
        // These codes are put in a bloc kto avoid borrowing param manager twice
        let mut input_manager = self.input_manager.borrow_mut();
        let mut input_manager_bridge: InputManagerBridge = From::from(&mut *input_manager);

        let step_fn = &self.step_fn;
        let mut operation_bridge = NodeOperationBridge { node: self };

        let seeds = step_fn((&mut input_manager_bridge, &mut operation_bridge));

        seeds
    }

    pub fn consume_seed(&self, seed: NodeSeed) {
        let mut input_manager = self.input_manager.borrow_mut();
        input_manager.push(seed.param);
    }
}

/**
 * Node should work like pipe:
 * Triggers are:
 * 1. Param push
 * 2. Worker notification
 */

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
    type_id: TypeId,
    key: Option<String>,
    param: Box<HeapDataCell>,
    generate_step_fn: Box<dyn Fn() -> Box<NodeStepFn>>,
}

impl NodeSeed {
    pub fn create(
        type_id: TypeId,
        key: Option<String>,
        param: Box<HeapDataCell>,
        generate_step_fn: Box<dyn Fn() -> Box<NodeStepFn>>,
    ) -> Self {
        Self {
            type_id,
            key,
            param,
            generate_step_fn,
        }
    }

    pub(crate) fn try_merge(seed: NodeSeed, node_rc: &mut Rc<NodeCell>) -> Result<(), NodeSeed> {
        let node_borrow = (*node_rc).borrow_mut();
        if seed.type_id == node_borrow.type_id && seed.key == node_borrow.key {
            node_borrow.consume_seed(seed);
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
        let step_fn = (seed.generate_step_fn)();

        let node = Node {
            type_id: seed.type_id,
            key: seed.key.clone(),
            input_manager: Default::default(),
            patch_manager: Default::default(),
            workers: Default::default(),
            step_fn,
        };

        let node_rc = Rc::new(RefCell::new(node));
        {
            let node = node_rc.borrow();

            if let Ok(mut patch_manager_write_guard) = node.patch_manager.write() {
                let work_item = WorkItem::from(&node_rc);
                let work_item_notifier = WorkItemNotifier::from_work_item(work_item, &sender);
                patch_manager_write_guard.work_item_notifier = Some(work_item_notifier);
                drop(patch_manager_write_guard);
            }

            {
                let mut input_manager = node.input_manager.borrow_mut();
                let work_item = WorkItem::from(&node_rc);
                (*input_manager).work_item_notifier =
                    Some(WorkItemNotifier::from_work_item(work_item, &sender));
            }

            // IMPORTANT: must be done after patch_manager.on_mutate_listener is installed
            node.consume_seed(seed);
            drop(node);

            // TODO: error handling for "else" block
            // which is a never scenario
        }

        node_rc
    }
}
