use crate::{typedef::Effect, WorkItemNotifier};
use std::{collections::VecDeque, mem};

#[derive(Default)]
pub struct PatchManager {
    pub(crate) current: VecDeque<Effect>,
    pub(crate) next: VecDeque<Effect>,
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
}

impl PatchManager {
    pub fn swap_patch(&mut self) -> () {
        // Throw away current_patches
        // Replace current_patches with next_pathces
        // Replace next_patches with new VecDeque
        let mut temp_vec_deque: VecDeque<Effect> = Default::default();
        mem::swap(&mut temp_vec_deque, &mut self.next);
        mem::swap(&mut temp_vec_deque, &mut self.current);
    }

    pub fn consume_patch(&mut self) -> () {
        while let Some(patch) = self.current.pop_front() {
            patch();
        }
    }

    pub fn push_patch(&mut self, patch: Effect) -> () {
        self.next.push_back(patch);
        self.notify_change_to_host();
    }

    pub fn notify_change_to_host(&self) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify();
        }
    }
}
