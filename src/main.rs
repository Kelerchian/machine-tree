use std::time::Duration;

use machinetree_backend::{self, node::Component, node::Node, node_host::NodeHost};

struct ExampleComponent {
    child_count: u32,
}

#[derive(Clone)]
struct Param {
    parent_index_chain: String,
    self_index: u32,
    child_count: u32,
}

impl Component for ExampleComponent {
    type Input = Param;

    fn construct(input: &Self::Input) -> Self
    where
        Self: Sized + 'static,
    {
        Self {
            child_count: input.child_count,
        }
    }

    fn step(
        &mut self,
        control: &mut dyn machinetree_backend::node::Control,
        param: &Self::Input,
    ) -> Vec<Node> {
        let self_index = param.self_index;
        let parent_index_chain = &param.parent_index_chain;
        let self_index_chain = format!("{parent_index_chain}/{self_index}");
        let current_child_count = self.child_count;

        if self.child_count > 0 {
            self.child_count = self.child_count - 1;
            control.rerender();
        }

        println!("{self_index_chain}");
        (0..current_child_count)
            .into_iter()
            .map(|index| {
                Self::seed(
                    Param {
                        self_index: index,
                        child_count: current_child_count - 1,
                        parent_index_chain: self_index_chain.clone(),
                    },
                    format!("{self_index_chain}/{index}"),
                )
            })
            .collect()
    }
}

fn main() {
    let mut host = NodeHost::create_with_root(ExampleComponent::seed(
        Param {
            parent_index_chain: String::from("root"),
            self_index: 0,
            child_count: 3,
        },
        format!(""),
    ));
    let mut i = 0;
    while host.step().render_count > 0 {
        println!("===end-of-iteration:{i}");
        i += 1;
    }
}
