use std::{
    borrow::BorrowMut,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    embeddable::{
        effect_manager::{self, EffectManager, EffectManagerBridge},
        input_manager::{self, InputManager, InputManagerBridge},
    },
    typedef::{Effect, HeapData, HeapDataCell, RuntimeError, WorkerStepFn},
};

pub struct WorkerOperationBridge<'a> {
    pub input: InputManagerBridge<'a>,
    pub effect: EffectManagerBridge<'a>,
}

impl<'a> WorkerOperationBridge<'a> {
    fn new(input: &'a mut InputManager, effect: &'a mut EffectManager) -> Self {
        Self {
            input: InputManagerBridge::from(input),
            effect: EffectManagerBridge::from(effect),
        }
    }
}
pub struct Worker {
    pub(crate) input_manager: RefCell<InputManager>,
    pub(crate) effect_manager: Arc<RwLock<EffectManager>>,
    pub(crate) state: Rc<HeapDataCell>,
    pub(crate) step_fn: Box<WorkerStepFn>,
    pub(crate) destroy_fn: Box<WorkerStepFn>,
}

impl Worker {
    // pub fn destroy(&self) {
    //     // TODO: implement
    // }

    // pub fn append_param(&mut self, data: Box<HeapDataCell>) {
    //     let mut input_manager = self.input_manager.borrow_mut();
    //     input_manager.push(data);
    // }

    // TODO: move logic to host
    // pub fn swap_patches(&self) {
    //     match self.effect_manager.write() {
    //         Ok(mut effect_manager) => {
    //             (*effect_manager).swap_queue();
    //         }
    //         Err(_poison_error) => {
    //             // TODO: tell the host there's a poison error
    //         }
    //     };
    // }

    pub fn run(&self) -> Result<(), RuntimeError> {
        let step_fn = &self.step_fn;
        let mut input_manager_mut_ref = self.input_manager.borrow_mut();
        let effect_manager_write_lock = self.effect_manager.write();

        match effect_manager_write_lock {
            Ok(mut effect_manager_mut_ref) => {
                let mut operation_bridge = WorkerOperationBridge::new(
                    &mut input_manager_mut_ref,
                    &mut effect_manager_mut_ref,
                );
                step_fn(&mut operation_bridge);
                Ok(())
            }
            Err(x) => {
                // TODO: signal error better
                Err(RuntimeError)
            }
        }
        // TODO: signal change to node and host
    }
}
