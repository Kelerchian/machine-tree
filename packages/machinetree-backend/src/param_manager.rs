use crate::typedef::HeapDataCell;
use std::{any::Any, cell::RefCell, collections::VecDeque};

pub struct ParamManager {
    pub(crate) param_queue: VecDeque<Box<HeapDataCell>>,
    pub(crate) current_param: Box<HeapDataCell>,
}

impl ParamManager {
    pub(crate) fn consume_queue(&mut self) -> bool {
        let first = self.param_queue.pop_front();
        if let Some(first) = first {
            self.current_param = first;
            true
        } else {
            false
        }
    }

    pub(crate) fn push(&mut self, data: Box<HeapDataCell>) {
        self.param_queue.push_back(data);
    }
}

// TODO: explain why bridges, what does it serve
pub struct ParamManagerBridge<'a> {
    pub(crate) param_manager: &'a mut ParamManager,
}

impl<'a> ParamManagerBridge<'a> {
    pub fn mutate_queue<
        ReturnType,
        MutateFunction: Fn(&mut VecDeque<Box<HeapDataCell>>) -> ReturnType,
    >(
        &mut self,
        mutate_fn: MutateFunction,
    ) -> ReturnType {
        let result = mutate_fn(&mut self.param_manager.param_queue);
        result
    }

    pub fn queue_len(&self) -> usize {
        self.param_manager.param_queue.len()
    }

    pub fn peek_current_param(&self) -> &Box<RefCell<Box<dyn Any>>> {
        &self.param_manager.current_param
    }
}

impl<'a> From<&'a mut ParamManager> for ParamManagerBridge<'a> {
    fn from(param_manager: &'a mut ParamManager) -> Self {
        ParamManagerBridge { param_manager }
    }
}
