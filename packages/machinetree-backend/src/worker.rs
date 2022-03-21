use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    input_manager::{InputManager, InputManagerBridge},
    patch_manager::PatchManager,
    typedef::{Effect, HeapData, HeapDataCell, WorkerStepFn},
};

pub struct WorkerOperationBridge<'a> {
    worker: &'a Worker,
    is_mutated: bool,
}

impl<'a> WorkerOperationBridge<'a> {
    pub fn read<Res, ReadFunction: Fn(&HeapData) -> Res>(
        &self,
        read_function: ReadFunction,
    ) -> Res {
        read_function(&*self.worker.state.borrow())
    }

    pub fn mutate<MutateFunction>(
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
                self.is_mutated = true;
                Ok(())
            }
            Err(x) => Err(()),
        }
    }
}

impl<'a> From<&'a Worker> for WorkerOperationBridge<'a> {
    fn from(worker: &'a Worker) -> Self {
        WorkerOperationBridge {
            worker,
            is_mutated: false,
        }
    }
}

pub struct Worker {
    input_manager: RefCell<InputManager>,
    patch_manager: Arc<RwLock<PatchManager>>,
    state: Rc<HeapDataCell>,
    step_fn: Box<WorkerStepFn>,
    destroy_fn: Box<WorkerStepFn>,
}

impl Worker {
    pub fn destroy(&self) {
        // TODO: implement
    }

    pub fn append_param(&mut self, data: Box<HeapDataCell>) {
        {
            let mut input_manager = self.input_manager.borrow_mut();
            input_manager.push(data);
        }
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
        let mut input_manager_mut_ref = self.input_manager.borrow_mut();
        let mut operation_bridge = WorkerOperationBridge::from(self);
        let mut input_manager_bridge = InputManagerBridge::from(&mut *input_manager_mut_ref);
        step_fn((&mut input_manager_bridge, &mut operation_bridge));
        // TODO: signal change to node and host
    }
}
