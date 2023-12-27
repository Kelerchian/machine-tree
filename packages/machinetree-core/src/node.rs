use crate::key::AnyBox;
use crate::key::BoxedAbsStep;
use crate::key::CloneInputBox;
use crate::key::Key;
use crate::key::RawKey;
use crate::key::Seed;
use crate::key::SeedData;
use crate::node_host::NodeControl;
use std::{any::TypeId, cell::RefCell};

mod component_utils {
    use crate::node_host::NodeControl;

    use super::Component;
    use crate::key::{AbsStep, AnyBox};

    pub fn downcast_as_input_ref<'a, Input>(input: &'a AnyBox) -> &'a Input
    where
        Input: Sized + Clone + 'static,
    {
        input.downcast_ref::<Input>().unwrap()
    }

    pub fn clone_input_box<Input>(abstract_input_box: &AnyBox) -> AnyBox
    where
        Input: Sized + Clone + 'static,
    {
        Box::new(downcast_as_input_ref::<Input>(&abstract_input_box).clone())
    }

    pub fn generate_abstract_step_fn<Machine, Input>(input: &Input) -> AbsStep
    where
        Input: Sized + Clone + 'static,
        Machine: Sized + Component<Input = Input> + 'static,
    {
        let mut self_state = Machine::construct(input);
        Box::new(move |control: &mut NodeControl, input: &AnyBox| {
            let input_ref = downcast_as_input_ref::<Input>(input);
            self_state.step(control, input_ref)
        })
    }
}

pub trait Component
where
    Self: Sized + 'static,
{
    type Input: Sized + Clone + 'static;

    fn construct(input: &Self::Input) -> Self
    where
        Self: Sized + 'static;

    fn seed(input: Self::Input, key: String) -> Seed {
        let type_id = TypeId::of::<Self>();
        let step_fn: BoxedAbsStep =
            Box::new(RefCell::new(component_utils::generate_abstract_step_fn::<
                Self,
                Self::Input,
            >(&input)));
        let input: AnyBox = Box::new(input);
        let inherit_input_fn_box: CloneInputBox =
            Box::new(component_utils::clone_input_box::<Self::Input>);
        let self_render_signaler = Default::default();

        Seed {
            key: RawKey {
                type_id,
                key,
                self_render: self_render_signaler,
            },
            data: SeedData {
                input,
                inherit_input_fn_box,
                step_fn,
            },
        }
    }

    fn step(&mut self, control: &mut NodeControl, input: &Self::Input) -> Vec<Seed>;
}

#[derive(Clone)]
pub struct SelfRenderSet {
    pub(crate) sender: crossbeam::channel::Sender<Key>,
    pub(crate) self_key: Key,
}

#[derive(Clone)]
pub enum SelfRender {
    Unset,
    Set(SelfRenderSet),
}

impl Default for SelfRender {
    fn default() -> Self {
        SelfRender::Unset
    }
}

impl SelfRender {
    pub(crate) fn set_self(&mut self, node_key: &Key, sender: &crossbeam::channel::Sender<Key>) {
        if let SelfRender::Set(_) = &self {
            return;
        }

        *self = SelfRender::Set(SelfRenderSet {
            sender: sender.clone(),
            self_key: node_key.clone(),
        });
    }

    pub fn rerender(&self) -> Result<(), ()> {
        match self {
            SelfRender::Set(signaler) => signaler
                .sender
                .send(signaler.self_key.clone())
                .map_err(|_| {}),
            _ => Ok(()),
        }
    }
}

pub(crate) enum WorkItem {
    Render(Key),
}
