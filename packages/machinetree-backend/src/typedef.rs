use crate::{Node, NodeMutationBridge, NodeSeed, WorkerMutationBridge};
use std::{any::Any, cell::RefCell};

// TODO: explain why they need to be here

pub(crate) type HeapData = Box<dyn Any>;
pub(crate) type NodeStepFn = dyn Fn(&mut NodeMutationBridge) -> Vec<NodeSeed>;
pub(crate) type WorkerStepFn = dyn Fn(&mut WorkerMutationBridge) -> Box<dyn Any>;
pub(crate) type NodeCell = RefCell<Node>;
pub(crate) type HeapDataCell = RefCell<HeapData>;
pub(crate) type Effect = Box<dyn FnOnce() -> ()>;
