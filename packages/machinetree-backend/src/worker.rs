use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    param_manager::{ParamManager, ParamManagerBridge},
    patch_manager::PatchManager,
    typedef::{Effect, HeapData, HeapDataCell, WorkerStepFn},
};

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
            let borrowed_state = &mut *(*state_rc).borrow_mut();
            mutate_function(borrowed_state);
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
    param_manager: RefCell<ParamManager>,
    patch_manager: Arc<RwLock<PatchManager>>,
    state: Rc<HeapDataCell>,
    step_fn: Box<WorkerStepFn>,
    destroy_fn: Box<WorkerStepFn>,
}

impl Worker {
    pub fn destroy(&self) {
        // TODO: implement
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

    pub fn run(&self) {
        let step_fn = &self.step_fn;
        let param_manager = &mut (*self.param_manager.borrow_mut());
        let mut mutation_bridge = WorkerMutationBridge { worker: self };
        let mut param_manager_bridge: ParamManagerBridge = From::from(param_manager);

        while param_manager_bridge.param_manager.consume_queue() {
            step_fn((&mut param_manager_bridge, &mut mutation_bridge));
        }
    }
}
