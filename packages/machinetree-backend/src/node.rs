use crate::{
    embeddable::{
        effect_manager::{EffectBridge, EffectManager},
        input_manager::{InputBridge, InputManager},
        state_manager::{StateBridge, StateManager},
    },
    node_seed::NodeSeed,
    typedef::{HeapDataCell, NodeStepFn, RuntimeError},
    worker::Worker,
};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

/*
 * Node should work like pipe:
 * Triggers are:
 * 1. Param push
 * 2. Worker notification
 */

pub struct Node {
    /**
     * Used to match with NodeSeed
     */
    pub(crate) type_id: TypeId,
    pub(crate) key: Option<String>,
    pub(crate) input_manager: RefCell<InputManager>,
    pub(crate) state_manager: RefCell<StateManager>,
    pub(crate) effect_manager: Arc<RwLock<EffectManager>>,
    pub(crate) workers: RefCell<HashMap<String, Rc<RefCell<Worker>>>>,
    pub(crate) step_fn: Box<NodeStepFn>,
}

impl Node {
    // TODO: move logic to host
    // pub fn swap_patches(&self) {
    //     match self.effect_manager.write() {
    //         Ok(mut effect_manager) => {
    //             (*effect_manager).swap_patch();
    //         }
    //         Err(_poison_error) => {
    //             // TODO: tell the host there's a poison error
    //         }
    //     }
    // }

    pub fn run<'a>(&'a mut self) -> Result<Vec<NodeSeed>, RuntimeError> {
        // These codes are put in a bloc kto avoid borrowing param manager twice
        let step_fn = &self.step_fn;
        let mut input_manager_mut_ref = self.input_manager.borrow_mut();
        let mut state_manager_mut_ref = self.state_manager.borrow_mut();
        let effect_manager_write_lock = self.effect_manager.write();

        match effect_manager_write_lock {
            Ok(mut effect_manager_mut_ref) => {
                let mut operation_bridge = NodeOperationBridge::new(
                    &self.key,
                    &mut input_manager_mut_ref,
                    &mut state_manager_mut_ref,
                    &mut effect_manager_mut_ref,
                );
                let seeds = step_fn(&mut operation_bridge);
                Ok(seeds)
            }
            Err(_) => {
                Err(RuntimeError)
                // TODO: signal runtime error
            }
        }
    }

    pub fn consume_input(&self, input: Box<HeapDataCell>) {
        let mut input_manager = self.input_manager.borrow_mut();
        input_manager.push(input);
    }
}
// TODO: explain capabilities
pub struct NodeOperationBridge<'a> {
    pub input: InputBridge<'a>,
    pub effect: EffectBridge<'a>,
    pub state: StateBridge<'a>,
    pub key: &'a Option<String>,
}

impl<'a> NodeOperationBridge<'a> {
    fn new(
        key: &'a Option<String>,
        input: &'a mut InputManager,
        state: &'a mut StateManager,
        effect: &'a mut EffectManager,
    ) -> Self {
        Self {
            key,
            input: input.into(),
            state: state.into(),
            effect: effect.into(),
        }
    }

    // TODO: provide a way to access worker
    // fn use_worker<ReturnType>(
    //     &'a self,
    //     id: &String,
    //     operation_fn: Box<dyn Fn(&mut WorkerOperationBridge) -> ReturnType>,
    // ) -> Option<ReturnType> {
    //     let workers = self.node.workers.borrow_mut();
    //     if let Some(worker) = (workers).get(id) {
    //         let return_value = {
    //             let borrowed = (worker).borrow_mut();
    //             let mut operation_bridge = WorkerOperationBridge::from(&*borrowed);
    //             let return_value = operation_fn(&mut operation_bridge);

    //             // TODO: signal to patch manager, or write it in worker.rs instead

    //             return_value
    //         };
    //         Some(return_value)
    //     } else {
    //         None
    //     }
    // }
}
