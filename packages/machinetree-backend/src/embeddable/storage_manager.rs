use crate::typedef::{HeapDataCell, MutateFn, PeekFn, TypedHeapData};
use std::{cell::RefCell, collections::HashMap};

type StateMutateFn<AssumedParamType, ReturnType> =
    MutateFn<TypedHeapData<AssumedParamType>, ReturnType>;
type StatePeekFn<AssumedParamType, ReturnType> =
    PeekFn<TypedHeapData<AssumedParamType>, ReturnType>;

#[derive(Default)]
pub struct StorageManager {
    // Work item notifier can be copied anytime anywhere by the stored data rather
    // than persisted by the manager
    // pub(crate) work_item_notifier: Option<WorkItemNotifier>,
    pub(crate) state_map: HashMap<String, HeapDataCell>,
}

impl StorageManager {
    fn has<T>(&mut self, key: &String) -> bool {
        self.state_map.contains_key(key)
    }

    fn get<'a, T>(&mut self, key: &String) -> Option<&'a mut Box<T>> {
        self.state_map.get(key).map(|x| {
            let mut borrow = x.borrow_mut();
            // Unwrap because there should not be failed downcast
            // expect when caused by memory corruption
            let mut typed_borrow = borrow.downcast::<T>().unwrap();

            &mut typed_borrow
        })
    }

    fn insert<'a, T>(&mut self, key: String, value: T) -> () {
        self.state_map.insert(key, RefCell::new(Box::new(value)));
    }
}
pub struct StorageBridge<'a> {
    manager: &'a mut StorageManager,
}

impl<'a> StorageBridge<'a> {
    fn get<T, I: Send>(&self, key: &String) -> Option<&'static mut Box<T>> {
        self.manager.get::<T>(key)
    }

    fn has<T, I: Send>(&self, key: &String) -> bool {
        self.manager.has::<T>(key)
    }

    fn insert<T>(&self, key: &String, value: T) -> () {
        self.manager.insert(key.clone(), value)
    }
}
