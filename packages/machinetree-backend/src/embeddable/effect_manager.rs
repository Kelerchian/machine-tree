use crate::{typedef::Effect, WorkItemKind, WorkItemNotifier};
use std::collections::VecDeque;

#[derive(Default)]
pub struct EffectManager {
    pub(crate) work_item_notifier: Option<WorkItemNotifier>,
    pub(crate) current: VecDeque<Effect>,
    pub(crate) next: VecDeque<Effect>,
}

impl EffectManager {
    pub(crate) fn push(&mut self, effect: Effect) -> () {
        self.next.push_back(effect);
    }

    pub(crate) fn run_all(&mut self) {
        let mut is_executed_at_least_once = false;
        while let Some(effect) = self.current.pop_front() {
            effect();
            is_executed_at_least_once = true;
        }

        if is_executed_at_least_once {
            self.notify_change_to_host(WorkItemKind::Step);
        }

        self.load_next();
    }

    pub(crate) fn load_next(&mut self) -> () {
        self.current.append(&mut self.next);
        if self.current.len() > 0 {
            self.notify_change_to_host(WorkItemKind::Effect);
        }
    }

    pub(crate) fn notify_change_to_host(&self, work_item_kind: WorkItemKind) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify(work_item_kind);
        }
    }
}

pub struct EffectManagerBridge<'a> {
    pub(crate) is_pushed_at_least_once: bool,
    pub(crate) effect_manager: &'a mut EffectManager,
}

impl<'a> Drop for EffectManagerBridge<'a> {
    fn drop(&mut self) {
        self.effect_manager.load_next();
    }
}

impl<'a> EffectManagerBridge<'a> {
    pub fn push(&mut self, effect: Effect) -> () {
        self.effect_manager.push(effect);
        self.is_pushed_at_least_once = true;
    }
}

impl<'a> From<&'a mut EffectManager> for EffectManagerBridge<'a> {
    fn from(effect_manager: &'a mut EffectManager) -> Self {
        EffectManagerBridge {
            is_pushed_at_least_once: Default::default(),
            effect_manager,
        }
    }
}
