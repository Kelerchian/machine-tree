use crate::{typedef::Effect, WorkItemKind, WorkItemNotifier};
use std::collections::VecDeque;

use super::{
    input_manager::{InputBridge, InputManager},
    state_manager::{StateBridge, StateManager},
};

pub struct EffectOperationBridge<'a> {
    state: StateBridge<'a>,
    input: InputBridge<'a>,
}

impl<'a> EffectOperationBridge<'a> {
    pub(crate) fn new(input: &'a mut InputManager, state: &'a mut StateManager) -> Self {
        EffectOperationBridge {
            input: input.into(),
            state: state.into(),
        }
    }
}

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

    pub(crate) fn run_all(&mut self, effect_bridge: &mut EffectOperationBridge) {
        let mut is_executed_at_least_once = false;
        while let Some(effect) = self.current.pop_front() {
            effect(effect_bridge);
            is_executed_at_least_once = true;
        }

        if is_executed_at_least_once {
            self.notify_work(WorkItemKind::EffectExecuted);
        }

        self.load_next();
    }

    pub(crate) fn load_next(&mut self) -> () {
        self.current.append(&mut self.next);
        if self.current.len() > 0 {
            self.notify_work(WorkItemKind::EffectAvailable);
        }
    }

    pub(crate) fn notify_work(&self, work_item_kind: WorkItemKind) {
        if let Some(work_item_sender) = &self.work_item_notifier {
            work_item_sender.notify(work_item_kind, false);
        }
    }
}

pub struct EffectBridge<'a> {
    pub(crate) is_pushed_at_least_once: bool,
    pub(crate) effect_manager: &'a mut EffectManager,
}

impl<'a> EffectBridge<'a> {
    pub fn push(&mut self, effect: Effect) -> () {
        self.effect_manager.push(effect);
        self.is_pushed_at_least_once = true;
    }
}

impl<'a> Into<EffectBridge<'a>> for &'a mut EffectManager {
    fn into(self) -> EffectBridge<'a> {
        EffectBridge {
            is_pushed_at_least_once: Default::default(),
            effect_manager: self,
        }
    }
}
