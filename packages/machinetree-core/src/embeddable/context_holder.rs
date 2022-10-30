use crate::node::Component;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
};

#[derive(Default)]
pub(crate) struct ContextHolder {
    pub type_map: HashMap<TypeIdOfContextContainer, Rc<dyn Any>>,
}

impl ContextHolder {
    pub(crate) fn get<'f, Container>(&'f self) -> Option<Rc<Container::Inner>>
    where
        Container: ContextContainer + 'static,
    {
        let type_id_of_container = TypeId::of::<Container>();
        let inner_any = self.type_map.get(&type_id_of_container)?;
        let inner_typed = Rc::downcast::<Container::Inner>(inner_any.clone()).unwrap();
        Some(inner_typed)
    }

    pub(crate) fn set<'f, Container>(
        &mut self,
        value: Container::Inner,
    ) -> Option<Rc<Container::Inner>>
    where
        Container: ContextContainer + 'static,
    {
        let type_id_of_container = TypeId::of::<Container>();
        let inner_any = self.type_map.insert(type_id_of_container, Rc::new(value))?;
        let inner_typed = Rc::downcast::<Container::Inner>(inner_any).unwrap();
        Some(inner_typed)
    }
}

pub trait ContextContainer: Component<Input = Self::Inner> {
    type Inner: Sized + 'static;
}

type TypeIdOfContextContainer = TypeId;

// pub struct ContextAccess<'a> {
//     pub(crate) context_holder_ref: &'a mut ContextHolder,
// }

// impl<'a> ContextAccess<'a> {
//     pub fn get_context<'f, Container>(&'a mut self) -> _
//     where
//         Container: ContextContainer + 'static,
//         'a: 'f,
//     {
//         self.context_holder_ref.borrow()
//         let type_id_of_container = TypeId::of::<Container>();
//         let value = self
//             .context_holder_ref
//             .type_map
//             .get_mut(&type_id_of_container);
//         match value {
//             None => None,
//             Some(inner_any) => {
//                 let inner_typed = inner_any.downcast_mut::<Container::Inner>().unwrap();
//                 Some(inner_typed)
//             }
//         }
//     }

//     pub fn set_context<'f, Container>(&'a mut self, inner: Container::Inner)
//     where
//         Container: ContextContainer + 'static,
//         'a: 'f,
//     {
//         let type_id_of_container = TypeId::of::<Container>();
//         self.context_holder_ref
//             .type_map
//             .insert(type_id_of_container, Box::new(inner));
//     }

//     pub fn has<'f, Container>(&'a mut self) -> bool
//     where
//         Container: ContextContainer + 'static,
//         'a: 'f,
//     {
//         let type_id_of_container = TypeId::of::<Container>();
//         self.context_holder_ref
//             .type_map
//             .contains_key(&type_id_of_container)
//     }
// }
