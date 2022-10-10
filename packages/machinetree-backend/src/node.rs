use std::{
    any::{Any, TypeId},
    cell::RefCell,
    rc::Rc,
};

use crate::embeddable::context_holder::{ContextAccess, ContextHolder};

mod component_utils {
    use super::{AbstractInputBox, AbstractStepFn, Component, Control, Node};

    pub fn downcast_as_input_ref<'a, Input>(input: &'a AbstractInputBox) -> &'a Input
    where
        Input: Sized + Clone + 'static,
    {
        input.downcast_ref::<Input>().unwrap()
    }

    pub fn clone_input_box<Input>(node: &Node) -> AbstractInputBox
    where
        Input: Sized + Clone + 'static,
    {
        Box::new(downcast_as_input_ref::<Input>(&node.input).clone())
    }

    pub fn generate_abstract_step_fn<Machine, Input>(input: &Input) -> AbstractStepFn
    where
        Input: Sized + Clone + 'static,
        Machine: Sized + Component<Input = Input> + 'static,
    {
        let mut self_state = Machine::construct(input);
        Box::new(move |control: &mut dyn Control, input: &AbstractInputBox| {
            let input_ref = downcast_as_input_ref::<Input>(input);
            self_state.step(control, input_ref)
        })
    }
}

pub trait Control<'a> {
    fn access_context<'f>(&'a mut self) -> &'f mut ContextAccess;
    fn rerender(&mut self) -> ();
}

pub trait Component
where
    Self: Sized + 'static,
{
    type Input: Sized + Clone + 'static;

    fn construct(input: &Self::Input) -> Self
    where
        Self: Sized + 'static;

    fn step(&mut self, control: &mut dyn Control, input: &Self::Input) -> Vec<Node>;

    fn seed(input: Self::Input, key: String) -> Node {
        let type_id = TypeId::of::<Self>();
        let step_fn: AbstractStepFnBrc =
            Box::new(RefCell::new(component_utils::generate_abstract_step_fn::<
                Self,
                Self::Input,
            >(&input)));
        let input: AbstractInputBox = Box::new(input);
        let inherit_input_fn_box: InheritInputFnBox =
            Box::new(component_utils::clone_input_box::<Self::Input>);
        let context_holder = Default::default();

        Node {
            type_id,
            key,
            input,
            inherit_input_fn_box,
            step_fn,
            context_holder,
        }
    }
}

pub struct Node {
    /**
     * Used to match with Node.
     * If a NodeSeed's type_id matches Node's type_id,
     * instead of creating a new Node
     * It appends the param into the Node's param
     */
    pub(crate) type_id: TypeId,
    pub(crate) key: String,
    pub(crate) input: AbstractInputBox,
    pub(crate) inherit_input_fn_box: InheritInputFnBox,
    pub(crate) step_fn: AbstractStepFnBrc,
    pub(crate) context_holder: ContextHolder,
}

// TODO: make StepFn more accomodating toward Self Struct, and not relying on FnMut
pub type StepFn<Input> = Box<dyn FnMut(&mut dyn Control, &Input) -> Vec<Node>>;
pub(crate) type InputBox<Input> = Box<Input>;
pub(crate) type AbstractInputBox = InputBox<dyn Any>;
pub(crate) type InheritInputFn = fn(&Node) -> AbstractInputBox;
pub(crate) type InheritInputFnBox = Box<InheritInputFn>;
pub(crate) type AbstractStepFn = StepFn<AbstractInputBox>;
pub(crate) type AbstractStepFnBrc = Box<RefCell<AbstractStepFn>>;
pub(crate) type NodeC = RefCell<Node>;
pub(crate) type NodeRcc = Rc<NodeC>;

impl Node {
    pub(crate) fn equal_as_node_reference(node_a: &Node, node_b: &Node) -> bool {
        node_a.type_id == node_b.type_id && node_a.key == node_b.key
    }

    pub fn clone_input(&self) -> AbstractInputBox {
        (&self.inherit_input_fn_box)(self)
    }
}

impl Into<NodeRcc> for Node {
    fn into(self) -> NodeRcc {
        Rc::new(RefCell::new(self))
    }
}
