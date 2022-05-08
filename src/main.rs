use machinetree_backend::{
    self,
    node::NodeOperationBridge,
    node_host::NodeHost,
    node_seed::NodeSeed,
    typedef::{HeapDataCell, NodeStepFn, WorkerSeedMap},
    worker::{Worker, WorkerSeed},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct TreeWorkerHostExample {}

impl TreeWorkerHostExample {
    pub fn create_seed() -> NodeSeed {
        let type_id = std::any::TypeId::of::<Self>();

        let generate_workers: Box<dyn FnOnce() -> WorkerSeedMap> = Box::new(|| {
            let mut map: WorkerSeedMap = Default::default();
            // TODO: Worker seed
            map.insert(
                "1".into(),
                WorkerSeed {
                    step_fn: Box::new(|worker_operation_bridge| {}),
                },
            );
            map
        });

        let step_fn: Box<NodeStepFn> = Box::new(|bridge| vec![]);

        NodeSeed::create(
            type_id,
            None,
            RefCell::new(Box::new(())),
            Some(generate_workers),
            step_fn,
        )
    }
}

struct TreeExampleConstructor {}

impl TreeExampleConstructor {
    pub fn step(bridge: &mut NodeOperationBridge) -> Vec<NodeSeed> {
        let count_res = bridge
            .input
            .peek::<u8, u8>(Box::new(|input_queue| -> u8 { input_queue.clone() }));

        let count = match count_res {
            Ok(x) => x,
            Err(_) => 0,
        };

        println!("key: {:?} count: {}", bridge.key, &count);

        if count > 0 {
            (0..=1)
                .into_iter()
                .map(|x| {
                    TreeExampleConstructor::create_seed(
                        count - 1,
                        Some(String::from(format!(
                            "{}{}",
                            match bridge.key {
                                Some(x) => format!("{}-", x),
                                None => String::from(""),
                            },
                            x
                        ))),
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn create_seed(input: u8, key: Option<String>) -> NodeSeed {
        let type_id = std::any::TypeId::of::<Self>();
        let param: HeapDataCell = RefCell::new(Box::new(input));
        let step_fn: Box<NodeStepFn> = Box::new(|bridge| Self::step(bridge));
        NodeSeed::create(type_id, key.clone(), param, None, step_fn)
    }
}

fn main() {
    let mut host = NodeHost::create_root(TreeExampleConstructor::create_seed(3, None));
    loop {
        let host = &mut host;
        if let Err(error) = host.receive_work() {
            if error == crossbeam::channel::TryRecvError::Empty {
                break;
            }
        }
        host.run_work();
    }
}
