use machinetree_backend::{
    self,
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
            Box::new(RefCell::new(Box::new(()))),
            Some(generate_workers),
            step_fn,
        )
    }
}

struct TreeExampleConstructor {}

impl TreeExampleConstructor {
    pub fn create_seed(input: u8, key: Option<String>) -> NodeSeed {
        let type_id = std::any::TypeId::of::<Self>();
        let param: Box<HeapDataCell> = Box::new(RefCell::new(Box::new(input)));
        let step_fn: Box<NodeStepFn> = Box::new(|bridge| {
            let input_res =
                bridge
                    .input
                    .peek::<u8, Option<u8>>(Box::new(|input_queue| -> Option<u8> {
                        let input_front = input_queue.front();
                        let input_res = match input_front {
                            Some(x) => {
                                let x = x.borrow();
                                let x = (**x).clone();
                                Some(x)
                            }
                            None => None,
                        };
                        input_res
                    }));

            let count: u8 = match input_res {
                Ok(x) => match x {
                    Some(x) => x,
                    None => 0u8,
                },
                Err(_) => 0u8,
            };
            println!("key: {:?} count: {}", bridge.key, count);

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
        });

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
