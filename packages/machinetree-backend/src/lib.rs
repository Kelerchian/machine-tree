pub mod node;
pub mod param_manager;
pub mod patch_manager;
pub mod typedef;
pub mod worker;

use node::{Node, NodeSeed};
use param_manager::{ParamManager, ParamManagerBridge};
use patch_manager::PatchManager;
use typedef::*;

use std::{cell::RefCell, rc::Rc};

pub struct NodeHost {
    // nodes: Vec<Weak<NodeCell>>,
    root: Rc<NodeCell>,
}

impl NodeHost {
    fn init_root(seed: NodeSeed) -> NodeHost {
        let node: Node = seed.into();
        NodeHost {
            root: Rc::new(RefCell::new(node)),
        }
    }

    fn run_root(&self) {
        let borrowed = &mut *self.root.borrow_mut();
        borrowed.run();
    }
}

// Initialize

// 1. Create Node and bind to Root
// 2. Run host worker

// Host Worker

// 1. For each node, recursively from root, create_patch
// 2. Run patch (which will queue more patches)
// 3. Repeat

// Run Patch
// 1. Read dependencies (props, Context (implemented later))
// 2. (Optional) queue for more patches
// 3. Determine and prune children

// Children Determination, memoiozation?
// 1. pikir nanti
