use crate::{
    typedef::{HeapDataCell, MutateFn, PeekFn, RuntimeError, TypedHeapDataCell},
    WorkItemKind, WorkItemNotifier,
};
use std::{collections::VecDeque, intrinsics::transmute};

#[derive(Default)]
pub struct InputManager {
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
    pub(crate) input_queue: VecDeque<Box<HeapDataCell>>,
}

pub type InputPeekFn<AssumedHeapdataType, ReturnType> =
    PeekFn<VecDeque<Box<TypedHeapDataCell<AssumedHeapdataType>>>, ReturnType>;
pub type InputMutateFn<AssumedHeapdataType, ReturnType> =
    MutateFn<VecDeque<Box<TypedHeapDataCell<AssumedHeapdataType>>>, ReturnType>;

impl InputManager {
    fn validate_payload_type<AssumedType: 'static>(&self) -> VecDeque<RuntimeError> {
        let mut errs: VecDeque<RuntimeError> = Default::default();

        self.input_queue.iter().for_each(|boxed_refcell| {
            let borrowed = boxed_refcell.as_ref().borrow();
            if let None = borrowed.as_ref().downcast_ref::<AssumedType>() {
                errs.push_back(RuntimeError);
            }
        });

        errs
    }

    pub(crate) fn peek<AssumedParamType: 'static, ReturnType>(
        &mut self,
        peek_fn: InputPeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        if self.validate_payload_type::<AssumedParamType>().len() > 0 {
            Err(RuntimeError)
        } else {
            // Swap->Transmute->Mutate->Transmute->Swap starts
            let mut swappable_temp = Default::default();
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            let transmuted: VecDeque<Box<TypedHeapDataCell<AssumedParamType>>> =
                unsafe { transmute(swappable_temp) };
            let result = peek_fn(&transmuted);
            let mut swappable_temp: VecDeque<Box<HeapDataCell>> = unsafe { transmute(transmuted) };
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            // Swap->Transmute->Mutate->Transmute->Swap ends

            Ok(result)
        }
    }

    pub(crate) fn mutate<AssumedParamType: 'static, ReturnType>(
        &mut self,
        mutate_fn: InputMutateFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        if self.validate_payload_type::<AssumedParamType>().len() > 0 {
            Err(RuntimeError)
        } else {
            // Swap->Transmute->Mutate->Transmute->Swap starts
            let mut swappable_temp = Default::default();
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            let mut transmuted: VecDeque<Box<TypedHeapDataCell<AssumedParamType>>> =
                unsafe { transmute(swappable_temp) };
            let result = mutate_fn(&mut transmuted);
            let mut swappable_temp: VecDeque<Box<HeapDataCell>> = unsafe { transmute(transmuted) };
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            // Swap->Transmute->Mutate->Transmute->Swap ends

            self.notify_change_to_host();

            Ok(result)
        }
    }

    pub(crate) fn notify_change_to_host(&self) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify(WorkItemKind::Step);
        }
    }

    pub(crate) fn push(&mut self, data: Box<HeapDataCell>) -> () {
        self.input_queue.push_back(data);
        self.notify_change_to_host();
    }

    pub(crate) fn push_many(&mut self, mut data: VecDeque<Box<HeapDataCell>>) -> () {
        self.input_queue.append(&mut data);
        self.notify_change_to_host();
    }
}

// TODO: explain why bridges, what does it serve
pub struct InputManagerBridge<'a> {
    pub(crate) input_manager: &'a mut InputManager,
}

impl<'a> InputManagerBridge<'a> {
    pub fn peek<AssumedParamType: 'static, ReturnType>(
        &mut self,
        peek_fn: InputPeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.input_manager.peek(peek_fn)
    }

    pub fn mutate<AssumedParamType: 'static, ReturnType>(
        &mut self,
        mutate_fn: InputMutateFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.input_manager.mutate(mutate_fn)
    }

    pub fn push(&mut self, data: Box<HeapDataCell>) -> () {
        self.input_manager.input_queue.push_back(data);
    }

    pub fn push_many(&mut self, mut data: VecDeque<Box<HeapDataCell>>) -> () {
        self.input_manager.input_queue.append(&mut data);
    }
}

impl<'a> From<&'a mut InputManager> for InputManagerBridge<'a> {
    fn from(input_manager: &'a mut InputManager) -> Self {
        InputManagerBridge { input_manager }
    }
}
