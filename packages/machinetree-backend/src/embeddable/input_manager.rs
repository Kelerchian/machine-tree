use std::cell::RefCell;

use crate::{
    typedef::{HeapData, HeapDataCell, PeekFn, RuntimeError},
    WorkItemKind, WorkItemNotifier,
};

pub struct InputManager {
    pub(crate) data: HeapDataCell,
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
}

pub type InputPeekFn<AssumedHeapdataType, ReturnType> = PeekFn<AssumedHeapdataType, ReturnType>;
impl InputManager {
    pub(crate) fn new(data: HeapDataCell) -> InputManager {
        let new_input = InputManager {
            data,
            work_item_notifier: None,
        };
        new_input
    }

    pub(crate) fn peek<AssumedParamType: 'static, ReturnType>(
        &self,
        peek_fn: InputPeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        let input = self.data.borrow();
        match input.downcast_ref::<AssumedParamType>() {
            Some(x) => Ok(peek_fn(x)),
            None => Err(RuntimeError),
        }
    }

    pub(crate) fn set(&mut self, data: HeapDataCell) {
        self.data = data;
        self.notify_work();
    }

    pub(crate) fn notify_work(&self) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify(WorkItemKind::StepIssued, true);
        }
    }
}

pub struct InputBridge<'a> {
    pub(crate) input_manager: &'a mut InputManager,
}

impl<'a> InputBridge<'a> {
    pub fn peek<AssumedParamType: 'static, ReturnType>(
        &mut self,
        peek_fn: InputPeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.input_manager.peek(peek_fn)
    }
}

impl<'a> Into<InputBridge<'a>> for &'a mut InputManager {
    fn into(self) -> InputBridge<'a> {
        InputBridge {
            input_manager: self,
        }
    }
}

// impl<'a> From<&'a mut InputManager> for InputManagerBridge<'a> {
//     fn from(input_manager: &'a mut InputManager) -> Self {
//         InputManagerBridge { input_manager }
//     }
// }
