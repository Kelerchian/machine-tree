use machinetree_backend::{
    self,
    node_host::NodeHost,
    node_seed::NodeSeed,
    typedef::{HeapDataCell, NodeStepFn},
};
use std::cell::RefCell;

struct TreeExampleConstructor {}

impl TreeExampleConstructor {
    pub fn create_seed(input: u8, key: Option<String>) -> NodeSeed {
        let type_id = std::any::TypeId::of::<TreeExampleConstructor>();

        let generate_step_fn: Box<dyn Fn() -> Box<NodeStepFn>> = Box::new(|| {
            // TODO: consider if step_fn should return Result<_, RuntimeError>
            // Emphasis on the RuntimeError
            let step_fn: Box<NodeStepFn> = Box::new(|bridge| {
                // TODO: macroify the ReturnType
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

                let mut seeds = vec![];

                for x in 0..count {
                    seeds.push(TreeExampleConstructor::create_seed(
                        count - 1,
                        Some(String::from(format!("{}-{}", count, x))),
                    ));
                }

                seeds
            });
            step_fn
        });

        let param: Box<HeapDataCell> = Box::new(RefCell::new(Box::new(input)));

        NodeSeed::create(type_id, key, param, generate_step_fn)
    }
}

fn main() {
    let mut host = NodeHost::create_root(TreeExampleConstructor::create_seed(4, None));
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
