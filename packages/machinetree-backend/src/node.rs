use crate::{
    typedef::{HeapDataCell, NodeCell, NodeStepFn},
    worker::Worker,
    ParamManager, ParamManagerBridge, PatchManager,
};
use std::{
    any::TypeId,
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    ops::DerefMut,
    rc::Rc,
    sync::{Arc, RwLock},
    vec,
};

// TODO: explain capabilities
pub struct NodeMutationBridge<'a> {
    node: &'a Node,
}

impl<'a> NodeMutationBridge<'a> {}

pub struct Node {
    /**
     * Used to match with NodeSeed
     */
    type_id: TypeId,
    param_manager: RefCell<ParamManager>,
    patch_manager: Arc<RwLock<PatchManager>>,
    workers: HashMap<String, RefCell<Worker>>,
    step_fn: Box<NodeStepFn>,
    children: RefCell<Vec<Rc<NodeCell>>>,
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

    pub fn run(&mut self) -> () {
        let step_fn = &self.step_fn;

        /**
         * Put this process in a block to avoid borrowing param_manager twice
         */
        {
            let param_manager = &mut *self.param_manager.borrow_mut();
            let mut mutation_bridge = NodeMutationBridge { node: self };
            let mut param_manager_bridge: ParamManagerBridge = From::from(param_manager);

            while param_manager_bridge.param_manager.consume_queue() {
                step_fn((&mut param_manager_bridge, &mut mutation_bridge));
            }

            {
                // TODO: if worker is not done working
                // Queue another patch
                for worker_cell in self.workers.values() {
                    let worker = worker_cell.borrow();
                    worker.run();
                }
            }
        }

        // TODO: determine children

        self.run_children();
    }

    pub fn run_children(&self) {
        let mut vec_ref_mut = self.children.borrow_mut();

        vec_ref_mut.iter_mut().for_each(|node_rc| {
            let node = &mut *(**node_rc).borrow_mut();
            node.run();
        });
    }

    pub fn merge_seed(&mut self, seed: NodeSeed) {
        let param_manager = &mut *self.param_manager.borrow_mut();
        param_manager.push(seed.param);
    }
}

impl From<NodeSeed> for Node {
    fn from(seed: NodeSeed) -> Self {
        let NodeSeed {
            seed_fn,
            param,
            type_id: _,
        } = seed;
        let node = seed_fn();
        node.param_manager.borrow_mut().push(param);
        node
    }
}

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
    param: Box<HeapDataCell>,
    seed_fn: Box<dyn Fn() -> Node>,
}
