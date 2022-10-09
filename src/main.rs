use std::time::Duration;

use machinetree_backend::{self, node::Component, node::StepFn, node_host::NodeHost};

struct ExampleComponent;

#[derive(Clone)]
struct Param {
    parent_index_chain: String,
    self_index: u32,
    parent_child_count: u32,
}

impl Component for ExampleComponent {
    type Input = Param;

    fn construct() -> StepFn<Self::Input> {
        // define state
        let mut child_count_opt = None;

        // step_fn
        Box::new(move |control, param| {
            let self_index = param.self_index;
            let initial_child_count = param.parent_child_count;
            let parent_index_chain = &param.parent_index_chain;
            let self_index_chain = format!("{parent_index_chain}/{self_index}");

            match child_count_opt {
                None => {
                    child_count_opt = Some(initial_child_count);
                    control.rerender();
                    println!("{self_index_chain}: initializing");
                    Default::default()
                }
                Some(child_count) => {
                    if child_count > 0 {
                        child_count_opt = Some(child_count - 1);
                        control.rerender();
                    }

                    println!("{self_index_chain}");
                    (0..child_count)
                        .into_iter()
                        .map(|index| {
                            Self::seed(
                                Param {
                                    self_index: index,
                                    parent_child_count: child_count - 1,
                                    parent_index_chain: self_index_chain.clone(),
                                },
                                format!("{self_index_chain}/{index}"),
                            )
                        })
                        .collect()
                }
            }
        })
    }
}

fn main() {
    let mut host = NodeHost::create_with_root(ExampleComponent::seed(
        Param {
            parent_index_chain: String::from("root"),
            self_index: 0,
            parent_child_count: 3,
        },
        format!(""),
    ));
    let mut i = 0;
    while host.step().render_count > 0 {
        println!("===end-of-iteration:{i}");
        std::thread::sleep(Duration::from_millis(10));
        i += 1;
    }
}
