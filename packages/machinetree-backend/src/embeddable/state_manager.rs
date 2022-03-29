use crate::{
    typedef::{HeapData, MutateFn, PeekFn, RuntimeError, TypedHeapData},
    WorkItemKind, WorkItemNotifier,
};
use std::collections::HashMap;

type StateMutateFn<AssumedParamType, ReturnType> =
    MutateFn<TypedHeapData<AssumedParamType>, ReturnType>;
type StatePeekFn<AssumedParamType, ReturnType> =
    PeekFn<TypedHeapData<AssumedParamType>, ReturnType>;

#[derive(Default)]
pub struct StateManager {
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
    pub(crate) state_map: HashMap<String, HeapData>,
}

impl StateManager {
    pub(crate) fn peek<AssumedParamType: Default + 'static, ReturnType>(
        &mut self,
        key: &String,
        peek_fn: StatePeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        let state = match self.state_map.remove(key) {
            Some(stored_state) => match stored_state.downcast::<AssumedParamType>() {
                Ok(downcasted_box) => downcasted_box,
                Err(_) => return Err(RuntimeError),
            },
            None => Box::new(AssumedParamType::default()),
        };

        let result = peek_fn(&state);
        self.state_map.insert(key.clone(), state);
        Ok(result)
    }

    pub(crate) fn mutate<AssumedParamType: Default + 'static, ReturnType>(
        &mut self,
        key: &String,
        mutate_fn: StateMutateFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        let mut state = match self.state_map.remove(key) {
            Some(stored_state) => match stored_state.downcast::<AssumedParamType>() {
                Ok(downcasted_box) => downcasted_box,
                Err(_) => return Err(RuntimeError),
            },
            None => Box::new(AssumedParamType::default()),
        };

        let result = mutate_fn(&mut state);
        self.state_map.insert(key.clone(), state);
        Ok(result)
    }

    pub(crate) fn notify_work(&self) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify(WorkItemKind::StepIssued, false);
        }
    }
}

// TODO: explain why bridges, what does it serve
pub struct StateBridge<'a> {
    pub(crate) state_manager: &'a mut StateManager,
}

impl<'a> StateBridge<'a> {
    pub fn peek<AssumedParamType: Default + 'static, ReturnType>(
        &mut self,
        key: &String,
        peek_fn: StatePeekFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.state_manager.peek(key, peek_fn)
    }

    pub fn mutate<AssumedParamType: Default + 'static, ReturnType>(
        &mut self,
        key: &String,
        mutate_fn: StateMutateFn<AssumedParamType, ReturnType>,
    ) -> Result<ReturnType, RuntimeError> {
        self.state_manager.mutate(key, mutate_fn)
    }
}

impl<'a> Into<StateBridge<'a>> for &'a mut StateManager {
    fn into(self) -> StateBridge<'a> {
        StateBridge {
            state_manager: self,
        }
    }
}
