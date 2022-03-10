use crate::{
    node::{Node, NodeMutationBridge, NodeSeed},
    worker::WorkerMutationBridge,
    ParamManagerBridge,
};
use std::{any::Any, cell::RefCell};

// TODO: explain why they need to be here

pub(crate) type HeapData = Box<dyn Any>;
pub(crate) type NodeStepFn =
    dyn Fn((&mut ParamManagerBridge, &mut NodeMutationBridge)) -> Vec<NodeSeed>;
pub(crate) type WorkerStepFn =
    dyn Fn((&mut ParamManagerBridge, &mut WorkerMutationBridge)) -> Box<dyn Any>;
pub(crate) type NodeCell = RefCell<Node>;
pub(crate) type HeapDataCell = RefCell<HeapData>;
pub(crate) type Effect = Box<dyn FnOnce() -> ()>;
