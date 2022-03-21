use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{
    node::Node,
    typedef::{HeapDataCell, NodeCell, NodeStepFn},
    WorkItem, WorkItemNotifier,
};

/**
 * Used to create a new Node or append param into a Node
 */
pub struct NodeSeed {
    /**
     * Used to match with Node.
     * If a NodeSeed's type_id matches Node's type_id,
     * instead of creating a new Node
     * It appends the param into the Node's param
     */
    pub(crate) type_id: TypeId,
    pub(crate) key: Option<String>,
    pub(crate) input: Box<HeapDataCell>,
    pub(crate) generate_step_fn: Box<dyn Fn() -> Box<NodeStepFn>>,
}

impl NodeSeed {
    pub fn create(
        type_id: TypeId,
        key: Option<String>,
        input: Box<HeapDataCell>,
        generate_step_fn: Box<dyn Fn() -> Box<NodeStepFn>>,
    ) -> Self {
        Self {
            type_id,
            key,
            input,
            generate_step_fn,
        }
    }

    pub(crate) fn try_merge(seed: NodeSeed, node_rc: &mut Rc<NodeCell>) -> Result<(), NodeSeed> {
        let node_borrow = (*node_rc).borrow_mut();
        if seed.type_id == node_borrow.type_id && seed.key == node_borrow.key {
            node_borrow.consume_seed(seed);
            drop(node_borrow);
            Ok(())
        } else {
            Err(seed)
        }
    }

    pub(crate) fn into_node_cell_rc(
        seed: NodeSeed,
        // TODO: rename, it is ugly
        sender: &crossbeam::channel::Sender<WorkItem>,
    ) -> Rc<RefCell<Node>> {
        let step_fn = (seed.generate_step_fn)();

        let node = Node {
            type_id: seed.type_id,
            key: seed.key.clone(),
            input_manager: Default::default(),
            effect_manager: Default::default(),
            workers: Default::default(),
            step_fn,
        };

        let node_rc = Rc::new(RefCell::new(node));
        {
            let node = node_rc.borrow();

            if let Ok(mut effect_manager_write_guard) = node.effect_manager.write() {
                let work_item = WorkItem::from(&node_rc);
                let work_item_notifier = WorkItemNotifier::from_work_item(work_item, &sender);
                effect_manager_write_guard.work_item_notifier = Some(work_item_notifier);
                drop(effect_manager_write_guard);
            }

            {
                let mut input_manager = node.input_manager.borrow_mut();
                let work_item = WorkItem::from(&node_rc);
                (*input_manager).work_item_notifier =
                    Some(WorkItemNotifier::from_work_item(work_item, &sender));
            }

            // IMPORTANT: must be done after patch_manager.on_mutate_listener is installed
            node.consume_seed(seed);
            drop(node);

            // TODO: error handling for "else" block
            // which is a never scenario
        }

        node_rc
    }
}
