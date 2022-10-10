use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::node::Component;

#[derive(Default)]
pub(crate) struct ContextHolder {
    pub(crate) type_map: HashMap<TypeIdOfContextContainer, Box<dyn Any>>,
}

pub trait ContextContainer: Component<Input = Self::Inner> {
    type Inner: Sized + 'static;
}

type TypeIdOfContextContainer = TypeId;

pub struct ContextAccess<'a> {
    pub(crate) context_holder_ref: &'a mut ContextHolder,
}

impl<'a> ContextAccess<'a> {
    pub fn get_context<'f, Container>(&'a mut self) -> Option<&'f mut Container::Inner>
    where
        Container: ContextContainer + 'static,
        'a: 'f,
    {
        let type_id_of_container = TypeId::of::<Container>();
        let value = self
            .context_holder_ref
            .type_map
            .get_mut(&type_id_of_container);
        match value {
            None => None,
            Some(inner_any) => {
                let inner_typed = inner_any.downcast_mut::<Container::Inner>().unwrap();
                Some(inner_typed)
            }
        }
    }

    pub fn set_context<'f, Container>(&'a mut self, inner: Container::Inner)
    where
        Container: ContextContainer + 'static,
        'a: 'f,
    {
        let type_id_of_container = TypeId::of::<Container>();
        self.context_holder_ref
            .type_map
            .insert(type_id_of_container, Box::new(inner));
    }

    pub fn has<'f, Container>(&'a mut self) -> bool
    where
        Container: ContextContainer + 'static,
        'a: 'f,
    {
        let type_id_of_container = TypeId::of::<Container>();
        self.context_holder_ref
            .type_map
            .contains_key(&type_id_of_container)
    }
}
