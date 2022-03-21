use crate::{typedef::Effect, WorkItemNotifier};
use std::{collections::VecDeque, mem};

#[derive(Default)]
pub struct EffectManager {
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
    pub(crate) current: VecDeque<Effect>,
    pub(crate) next: VecDeque<Effect>,
}

impl EffectManager {
    pub(crate) fn push(&mut self, effect: Effect) -> () {
        self.next.push_back(effect);
        self.notify_change_to_host();
    }

    pub(crate) fn consume_and_run(&mut self) -> Option<()> {
        match self.current.pop_front() {
            Some(effect) => {
                effect();
                Some(())
            }
            None => None,
        }
    }

    pub(crate) fn swap_queue(&mut self) -> () {
        // Throw away current_patches
        // Replace current_patches with next_pathces
        // Replace next_patches with new VecDeque
        let mut temp_vec_deque: VecDeque<Effect> = Default::default();
        mem::swap(&mut temp_vec_deque, &mut self.next);
        mem::swap(&mut temp_vec_deque, &mut self.current);
    }

    pub(crate) fn notify_change_to_host(&self) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify();
        }
    }
}

pub struct EffectManagerBridge<'a> {
    pub(crate) effect_manager: &'a mut EffectManager,
}

impl<'a> EffectManagerBridge<'a> {
    pub fn push(&mut self, effect: Effect) -> () {
        self.effect_manager.push(effect);
    }
}

impl<'a> From<&'a mut EffectManager> for EffectManagerBridge<'a> {
    fn from(effect_manager: &'a mut EffectManager) -> Self {
        EffectManagerBridge { effect_manager }
    }
}
