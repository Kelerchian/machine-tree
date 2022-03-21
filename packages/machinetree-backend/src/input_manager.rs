use crate::{
    typedef::{HeapDataCell, MutateFn, PeekFn, RuntimeError, TypedHeapdataCell},
    WorkItemNotifier,
};
use std::{collections::VecDeque, intrinsics::transmute};

#[derive(Default)]
pub struct InputManager {
    pub(crate) input_queue: VecDeque<Box<HeapDataCell>>,
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
}

impl InputManager {
    pub fn mutate<AssumedParamType: 'static, ReturnType>(
        &mut self,
        mutate_fn: MutateFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        let mut errs: VecDeque<RuntimeError> = Default::default();

        // Validation for later unsafe transmute below
        self.input_queue.iter().for_each(|boxed_refcell| {
            let borrowed = boxed_refcell.as_ref().borrow();

            if let None = borrowed.as_ref().downcast_ref::<AssumedParamType>() {
                &errs.push_back(RuntimeError);
            }
        });

        if errs.len() > 0 {
            Err(RuntimeError)
        } else {
            // Swap->Transmute->Mutate->Transmute->Swap starts
            let mut swappable_temp = Default::default();
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            let mut transmuted: VecDeque<Box<TypedHeapdataCell<AssumedParamType>>> =
                unsafe { transmute(swappable_temp) };
            let result = mutate_fn(&mut transmuted);
            let mut swappable_temp: VecDeque<Box<HeapDataCell>> = unsafe { transmute(transmuted) };
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            // Swap->Transmute->Mutate->Transmute->Swap ends

            self.notify_change_to_host();

            Ok(result)
        }
    }

    pub fn peek<AssumedParamType: 'static, ReturnType>(
        &mut self,
        peek_fn: PeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        let mut errs: VecDeque<RuntimeError> = Default::default();

        // Validation for later unsafe transmute below
        self.input_queue
            .iter()
            .for_each(|boxed_refcell: &Box<HeapDataCell>| {
                let borrowed_refcell = boxed_refcell.as_ref().borrow();

                if let None = borrowed_refcell.as_ref().downcast_ref::<AssumedParamType>() {
                    &errs.push_back(RuntimeError);
                }
            });

        if errs.len() > 0 {
            Err(RuntimeError)
        } else {
            // Swap->Transmute->Mutate->Transmute->Swap starts
            let mut swappable_temp = Default::default();
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            let transmuted: VecDeque<Box<TypedHeapdataCell<AssumedParamType>>> =
                unsafe { transmute(swappable_temp) };
            let result = peek_fn(&transmuted);
            let mut swappable_temp: VecDeque<Box<HeapDataCell>> = unsafe { transmute(transmuted) };
            std::mem::swap(&mut swappable_temp, &mut self.input_queue);
            // Swap->Transmute->Mutate->Transmute->Swap ends

            Ok(result)
        }
    }

    pub fn notify_change_to_host(&self) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify();
        }
    }

    pub fn push(&mut self, data: Box<HeapDataCell>) -> () {
        // TODO: determine EQ
        self.input_queue.push_back(data);
        self.notify_change_to_host();
    }
}

// TODO: explain why bridges, what does it serve
pub struct InputManagerBridge<'a> {
    pub(crate) input_manager: &'a mut InputManager,
}

impl<'a> InputManagerBridge<'a> {
    pub fn mutate<AssumedParamType: 'static, ReturnType>(
        &mut self,
        mutate_fn: MutateFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.input_manager.mutate(mutate_fn)
    }

    pub fn peek<AssumedParamType: 'static, ReturnType>(
        &mut self,
        peek_fn: PeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.input_manager.peek(peek_fn)
    }

    pub fn queue_len(&self) -> usize {
        self.input_manager.input_queue.len()
    }
}

impl<'a> From<&'a mut InputManager> for InputManagerBridge<'a> {
    fn from(input_manager: &'a mut InputManager) -> Self {
        InputManagerBridge { input_manager }
    }
}
