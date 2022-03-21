use crate::{
    embeddable::{
        effect_manager::{EffectManager, EffectManagerBridge},
        input_manager::{InputManager, InputManagerBridge},
    },
    node_seed::NodeSeed,
    typedef::{NodeStepFn, RuntimeError},
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
    //     };
    // }

    pub fn run<'a>(&'a mut self) -> Result<Vec<NodeSeed>, RuntimeError> {
        // These codes are put in a bloc kto avoid borrowing param manager twice
        let step_fn = &self.step_fn;
        let mut input_manager_mut_ref = self.input_manager.borrow_mut();
        let mut effect_manager_write_lock = self.effect_manager.write();

        match effect_manager_write_lock {
            Ok(mut effect_manager_mut_ref) => {
                let mut operation_bridge = NodeOperationBridge::new(
                    &mut input_manager_mut_ref,
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

    pub fn consume_seed(&self, seed: NodeSeed) {
        let mut input_manager = self.input_manager.borrow_mut();
        input_manager.push(seed.input);
    }
}
// TODO: explain capabilities
pub struct NodeOperationBridge<'a> {
    pub input: InputManagerBridge<'a>,
    pub effect: EffectManagerBridge<'a>,
}

impl<'a> NodeOperationBridge<'a> {
    fn new(input: &'a mut InputManager, effect: &'a mut EffectManager) -> Self {
        Self {
            input: InputManagerBridge::from(input),
            effect: EffectManagerBridge::from(effect),
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
