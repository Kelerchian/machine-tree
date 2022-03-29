use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use crate::{
    embeddable::{
        effect_manager::{EffectExecutionBridge, EffectManager, EffectBridge},
        input_manager::{InputManager, InputBridge},
        state_manager::{StateManager, StateBridge},
    },
    typedef::{RuntimeError, WorkerStepFn},
};

pub struct WorkerOperationBridge<'a> {
    pub input: InputBridge<'a>,
    pub effect: EffectBridge<'a>,
    pub state: StateBridge<'a>,
}

impl<'a> WorkerOperationBridge<'a> {
    fn new(
        input: &'a mut InputManager,
        state: &'a mut StateManager,
        effect: &'a mut EffectManager,
    ) -> Self {
        Self {
            state: state.into(),
            input: input.into(),
            effect: effect.into(),
        }
    }
}
pub struct Worker {
    pub(crate) input_manager: RefCell<InputManager>,
    pub(crate) state_manager: RefCell<StateManager>,
    pub(crate) effect_manager: Arc<RwLock<EffectManager>>,
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

    pub fn run_step(&self) -> Result<(), RuntimeError> {
        let step_fn = &self.step_fn;
        let mut input_ref = self.input_manager.borrow_mut();
        let mut state_ref = self.state_manager.borrow_mut();
        let effect_manager_write_lock = self.effect_manager.write();

        match effect_manager_write_lock {
            Ok(mut effect_ref) => {
                let mut operation_bridge =
                    WorkerOperationBridge::new(&mut input_ref, &mut state_ref, &mut effect_ref);
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

    pub fn run_effect(&self) -> Result<(), RuntimeError> {
        let mut input_ref = self.input_manager.borrow_mut();
        let mut state_ref = self.state_manager.borrow_mut();
        let effect_manager_write_lock = self.effect_manager.write();

        match effect_manager_write_lock {
            Ok(mut effect_ref) => {
                let mut effect_execution_bridge =
                    EffectExecutionBridge::new(&mut input_ref, &mut state_ref);
                effect_ref.run_all(&mut effect_execution_bridge);
                Ok(())
            }
            Err(x) => {
                // TODO: signal error better
                Err(RuntimeError)
            }
        }
    }
}
