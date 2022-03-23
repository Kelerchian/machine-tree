use crate::{
    node::{Node, NodeOperationBridge},
    node_seed::NodeSeed,
    worker::{Worker, WorkerOperationBridge},
};
use std::{any::Any, cell::RefCell, collections::VecDeque, rc::Rc};

// TODO: explain why they need to be here

pub(crate) type HeapData = Box<dyn Any>;
pub type HeapDataCell = RefCell<HeapData>;

pub(crate) type TypedHeapData<T> = Box<T>;
pub type TypedHeapdataCell<T> = RefCell<TypedHeapData<T>>;

pub type NodeStepFn = dyn Fn(&mut NodeOperationBridge) -> Vec<NodeSeed>;
pub(crate) type WorkerStepFn = dyn Fn(&mut WorkerOperationBridge) -> Box<dyn Any>;

pub(crate) type NodeCell = RefCell<Node>;
pub(crate) type NodeCellRc = Rc<NodeCell>;
pub(crate) type WorkerCell = RefCell<Worker>;
pub(crate) type WorkerCellRc = Rc<WorkerCell>;

pub(crate) type Effect = Box<dyn FnOnce() -> ()>;

pub type PeekFn<AssumedHeapdataType, ReturnType> =
    Box<dyn Fn(&VecDeque<Box<TypedHeapdataCell<AssumedHeapdataType>>>) -> ReturnType>;

pub type MutateFn<AssumedHeapdataType, ReturnType> =
    Box<dyn Fn(&mut VecDeque<Box<TypedHeapdataCell<AssumedHeapdataType>>>) -> ReturnType>;

pub struct RuntimeError;
