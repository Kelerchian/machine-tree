use std::rc::Rc;

use super::lake::NodeLake;
use crate::{embeddable::context_holder::ContextContainer, key::Key};

#[derive(Clone)]
pub struct NodeNavigator {
    pub(crate) current: Key,
}

impl NodeNavigator {
    pub fn get_parent(&self, lake: &NodeLake) -> Option<NodeNavigator> {
        let data = lake.get(&self.current)?;
        let parent_wcc = (&data.borrow_self().borrow_relations().parent).clone()?;
        let parent_rcc = parent_wcc.upgrade()?;
        Some(NodeNavigator {
            current: parent_rcc.into(),
        })
    }
}
pub struct ContextAccess<'a> {
    pub(crate) lake: &'a NodeLake,
    pub(crate) node_key_pointer: Key,
}

impl<'a> ContextAccess<'a> {
    pub fn set_context<Container>(
        &mut self,
        value: Container::Inner,
    ) -> Option<Rc<<Container as ContextContainer>::Inner>>
    where
        Container: ContextContainer,
    {
        let node_data = self.lake.get(&self.node_key_pointer);

        match node_data {
            Some(node_data) => {
                let node_data_point = node_data.borrow_self();
                let mut context_holder = node_data_point.borrow_mut_context();
                (*context_holder).set::<Container>(value)
            }
            None => None,
        }
    }

    pub fn get_context<'b, Container, ReturnValue>(&'a self) -> Option<Rc<Container::Inner>>
    where
        Container: ContextContainer,
        'a: 'b,
    {
        let mut maybe_navigator = Some(NodeNavigator {
            current: self.node_key_pointer.clone(),
        });

        let context_data = loop {
            let next_navigator = {
                if let Some(navigator) = &maybe_navigator {
                    let node_data = self.lake.get(&navigator.current);

                    if let Some(node_data) = node_data {
                        let node_data_point = node_data.borrow_self();
                        let context_holder = node_data_point.borrow_mut_context();
                        if let Some(context_data) = context_holder.get::<Container>() {
                            break Some(context_data.clone());
                        }
                    }

                    navigator.get_parent(self.lake)
                } else {
                    break None;
                }
            };

            maybe_navigator = next_navigator;
        };

        context_data
    }
}
