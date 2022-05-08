use crate::{
    embeddable::effect_manager::EffectOperationBridge,
    node::{Node, NodeOperationBridge},
    node_seed::NodeSeed,
    worker::{Worker, WorkerOperationBridge, WorkerSeed},
};
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

// TODO: explain why they need to be here

pub(crate) type HeapData = Box<dyn Any>;
pub type HeapDataCell = RefCell<HeapData>;

pub(crate) type TypedHeapData<T> = Box<T>;
pub type TypedHeapDataCell<T> = RefCell<TypedHeapData<T>>;

pub type NodeStepFn = dyn Fn(&mut NodeOperationBridge) -> Vec<NodeSeed>;
pub(crate) type WorkerStepFn = dyn Fn(&mut WorkerOperationBridge) -> ();

pub(crate) type NodeCell = RefCell<Node>;
pub(crate) type NodeCellRc = Rc<NodeCell>;
pub(crate) type WorkerCell = RefCell<Worker>;
pub(crate) type WorkerCellRc = Rc<WorkerCell>;
pub(crate) type WorkerMap = HashMap<String, Rc<RefCell<Worker>>>;
pub type WorkerSeedMap = HashMap<String, WorkerSeed>;

pub(crate) type Effect = Box<dyn FnOnce(&mut EffectOperationBridge) -> ()>;

pub type PeekFn<AssumedHeapdataType, ReturnType> = Box<dyn Fn(&AssumedHeapdataType) -> ReturnType>;
pub type MutateFn<AssumedHeapdataType, ReturnType> =
    Box<dyn Fn(&mut AssumedHeapdataType) -> ReturnType>;

pub struct RuntimeError;
