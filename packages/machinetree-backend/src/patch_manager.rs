use crate::typedef::Effect;
use std::{collections::VecDeque, mem};

pub struct PatchManager {
    current: VecDeque<Effect>,
    next: VecDeque<Effect>,
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
    }
}
