use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::{
    embeddable::{
        effect_manager::{EffectBridge, EffectManager, EffectOperationBridge},
        input_manager::{InputBridge, InputManager},
        state_manager::{StateBridge, StateManager},
    },
    typedef::{RuntimeError, WorkerCell, WorkerCellRc, WorkerMap, WorkerSeedMap, WorkerStepFn},
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

pub struct WorkerSeed {
    pub step_fn: Box<WorkerStepFn>,
}

impl Into<Worker> for WorkerSeed {
    fn into(self) -> Worker {
        Worker {
            input_manager: Default::default(),
            state_manager: Default::default(),
            effect_manager: Default::default(),
            step_fn: self.step_fn,
        }
    }
}

pub struct Worker {
    pub(crate) input_manager: RefCell<InputManager>,
    pub(crate) state_manager: RefCell<StateManager>,
    pub(crate) effect_manager: Arc<RwLock<EffectManager>>,
    pub(crate) step_fn: Box<WorkerStepFn>,
    // pub(crate) destroy_fn: Box<WorkerStepFn>,
}

impl Into<WorkerCellRc> for Worker {
    fn into(self) -> WorkerCellRc {
        Rc::new(RefCell::new(self))
    }
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
                    EffectOperationBridge::new(&mut input_ref, &mut state_ref);
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
