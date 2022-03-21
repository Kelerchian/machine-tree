use std::{cell::RefCell, time::Duration};

use machinetree_backend::{
    self,
    node::NodeSeed,
    typedef::{HeapDataCell, NodeStepFn, PeekFn},
    NodeHost,
};

struct TreeExampleConstructor {}

impl TreeExampleConstructor {
    pub fn create_seed(input: u8, key: Option<String>) -> NodeSeed {
        let type_id = std::any::TypeId::of::<TreeExampleConstructor>();

        let generate_step_fn: Box<dyn Fn() -> Box<NodeStepFn>> = Box::new(|| {
            // TODO: consider if step_fn should return Result<_, RuntimeError>
            // Emphasis on the RuntimeError
            let step_fn: Box<NodeStepFn> = Box::new(|(input_manager, _y)| {
                // TODO: macroify the ReturnType
                let x = input_manager.peek::<u8, Option<u8>>(Box::new(|x| -> Option<u8> {
                    let a = x.front();
                    let b = match a {
                        Some(x) => {
                            let x = x.borrow();
                            let x = (**x).clone();
                            Some(x)
                        }
                        None => None,
                    };
                    b
                }));

                let count: u8 = match x {
                    Ok(x) => match x {
                        Some(x) => x,
                        None => 0u8,
                    },
                    Err(_) => 0u8,
                };

                println!("count: {}", &count);

                let mut seeds = vec![];

                for x in 0..count {
                    seeds.push(TreeExampleConstructor::create_seed(
                        x,
                        Some(String::from(format!("{}", x))),
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
    let mut host = NodeHost::create_root(TreeExampleConstructor::create_seed(3, None));

    loop {
        let host = &mut host;
        println!("received work: {:?}", host.receive_work());
        host.run_work();
        std::thread::sleep(Duration::from_millis(1000));
    }

    // let mut controls = vec![[Control {
    //     param: 3u8,
    //     data: RefCell::new(Box::new(SampleCounter { count: 10 })),
    //     mutation_queue: RefCell::new(VecDeque::new()),
    // }]];

    // loop {
    //     controls.iter_mut().for_each(|[control]| {
    //         control.step();
    //     });
    // }
}
