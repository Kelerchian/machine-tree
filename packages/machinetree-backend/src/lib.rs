mod typedef;
use typedef::*;

use std::any::TypeId;
use std::cell::RefMut;
use std::ops::FnOnce;
use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    mem,
    rc::Rc,
    sync::{Arc, RwLock},
};

pub struct NodeHost {
    // nodes: Vec<Weak<NodeCell>>,
    root: Rc<NodeCell>,
}

impl NodeHost {
    // fn clean_unused_nodes(&mut self) {
    //     self.nodes.retain(|weak| weak.upgrade().is_some())
    // }

    fn run_root(&self) {
        Node::step(&self.root);
    }
}

struct PatchManager {
    current: VecDeque<Effect>,
    next: VecDeque<Effect>,
}

impl PatchManager {
    pub fn swap_patch(&mut self) -> () {
        // Throw away current_patches
        // Replace current_patches with next_pathces
        // Replace next_patches with new VecDeque
        let mut temp_vec_deque: VecDeque<Effect> = Default::default();
        mem::swap(&mut temp_vec_deque, &mut self.next);
        mem::swap(&mut temp_vec_deque, &mut self.current);
    }

    pub fn push_patch(&mut self, patch: Effect) -> () {
        self.next.push_back(patch);
    }
}

pub struct WorkerMutationBridge<'a> {
    worker: &'a Worker,
}

impl<'a> WorkerMutationBridge<'a> {
    fn read_state_with<Res, ReadFunction: Fn(&HeapData) -> Res>(
        &self,
        read_function: ReadFunction,
    ) -> Res {
        read_function(&*self.worker.state.borrow())
    }

    fn mutate_state_later_with<MutateFunction>(
        &mut self,
        mutate_function: Box<dyn FnOnce(&mut HeapData) -> ()>,
    ) -> Result<(), ()> {
        let state_rc = self.worker.state.clone();
        let effect: Effect = Box::new(move || {
            let state_rc = state_rc.clone();
            mutate_function(&mut *state_rc.borrow_mut());
        });
        match self.worker.patch_manager.write() {
            Ok(mut patch_manager) => {
                patch_manager.push_patch(effect);
                Ok(())
            }
            Err(x) => Err(()),
        }
    }
}

pub struct Worker {
    next_param_queue: RefCell<VecDeque<Rc<HeapDataCell>>>,
    patch_manager: Arc<RwLock<PatchManager>>,
    state: Rc<HeapDataCell>,
    step_fn: Box<WorkerStepFn>,
    destroy_fn: Box<WorkerStepFn>,
}

impl Worker {
    pub fn step(&self) {
        (self.step_fn)(&mut WorkerMutationBridge { worker: self });
    }

    pub fn destroy(&self) {
        (self.destroy_fn)(&mut WorkerMutationBridge { worker: self });
    }

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
}

pub struct ParamManager {
    param_queue: VecDeque<Box<HeapDataCell>>,
    current_param: Box<HeapDataCell>,
}

pub struct ParamManagerBridge<'a> {
    param_manager: &'a mut ParamManager,
}

impl<'a> ParamManagerBridge<'a> {
    pub fn mutate_queue<
        ReturnType,
        MutateFunction: Fn(&mut VecDeque<Box<HeapDataCell>>) -> ReturnType,
    >(
        &mut self,
        mutate_fn: MutateFunction,
    ) -> ReturnType {
        mutate_fn(&mut self.param_manager.param_queue)
    }

    pub fn len(&self) -> usize {
        self.param_manager.param_queue.len()
    }

    pub fn peek_current_param(&self) -> &Box<RefCell<Box<dyn Any>>> {
        &self.param_manager.current_param
    }
}

pub struct NodeMutationBridge {
    node: Rc<NodeCell>,
}

impl NodeMutationBridge {
    fn get_prop(&self) -> Rc<RefCell<Box<dyn Any>>> {
        (*self.node).borrow().param.clone()
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

pub struct Node {
    /**
     * Used to match with NodeSeed
     */
    type_id: TypeId,
    param_manager: RefCell<ParamManager>,
    patch_manager: Arc<RwLock<PatchManager>>,
    param: Rc<HeapDataCell>,
    workers: HashMap<String, Worker>,
    step_fn: Box<NodeStepFn>,
    children: Vec<Rc<NodeCell>>,
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

    pub fn step(node_rc: &Rc<NodeCell>) {
        let mutation_bridge = NodeMutationBridge {
            node: node_rc.clone(),
        };
        // let step_fn = (*node_rc).borrow().step_fn;
        // step_fn(&mutation_bridge);
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
