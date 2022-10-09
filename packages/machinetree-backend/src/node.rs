use std::{
    any::{Any, TypeId},
    cell::RefCell,
    rc::Rc,
};

pub trait Control {
    fn rerender(&mut self) -> ();
}

pub trait Component
where
    Self: Sized + 'static,
{
    type Input: Sized + Clone + 'static;

    fn construct() -> StepFn<Self::Input>;

    fn inherit_input(node: &Node) -> AbstractizedInputBox
    where
        Self: Sized,
    {
        Box::new(Self::get_input_box_ref(&node.input).clone())
    }

    fn create_inherit_input_fn_box() -> InheritInputFnBox
    where
        Self: Sized,
    {
        Box::new(Self::inherit_input)
    }

    fn get_input_box_ref<'a>(input: &'a AbstractizedInputBox) -> &'a Self::Input {
        input.downcast_ref::<Self::Input>().unwrap()
    }

    fn construct_abstract() -> AbstractizedStepFn {
        // Reverse Box<dyn Any>
        // to Box<RefCell<Box<dyn FnMut(Self::Input) -> Vec<NodeSeed>>>>

        let mut step_fn = Self::construct();
        Box::new(
            move |control: &mut dyn Control, input: &AbstractizedInputBox| {
                step_fn(control, &Self::get_input_box_ref(input))
            },
        )
    }

    fn seed(input: Self::Input, key: String) -> Node {
        let type_id = TypeId::of::<Self>();
        let step_fn: AbstractizedStepFnBrc = Box::new(RefCell::new(Self::construct_abstract()));
        let input: AbstractizedInputBox = Box::new(input);
        let inherit_input_fn_box: InheritInputFnBox = Box::new(Self::inherit_input);
        Node {
            type_id,
            key,
            input,
            inherit_input_fn_box,
            step_fn,
        }
    }
}

pub struct ExampleNodeFac;
impl Component for ExampleNodeFac {
    type Input = u32;

    fn construct() -> StepFn<Self::Input> {
        Box::new(|_, _| vec![])
    }
}

/**
 * Used to create a new Node or append param into a Node
 */
pub struct Node {
    /**
     * Used to match with Node.
     * If a NodeSeed's type_id matches Node's type_id,
     * instead of creating a new Node
     * It appends the param into the Node's param
     */
    pub(crate) type_id: TypeId,
    pub(crate) key: String,
    pub(crate) input: AbstractizedInputBox,
    pub(crate) inherit_input_fn_box: InheritInputFnBox,
    pub(crate) step_fn: AbstractizedStepFnBrc,
}

// TODO: make StepFn more accomodating toward Self Struct, and not relying on FnMut
pub type StepFn<Input> = Box<dyn FnMut(&mut dyn Control, &Input) -> Vec<Node>>;
pub(crate) type InputBox<Input> = Box<Input>;
pub(crate) type AbstractizedInputBox = InputBox<dyn Any>;
pub(crate) type InheritInputFn = fn(&Node) -> AbstractizedInputBox;
pub(crate) type InheritInputFnBox = Box<InheritInputFn>;
pub(crate) type AbstractizedStepFn = StepFn<AbstractizedInputBox>;
pub(crate) type AbstractizedStepFnBrc = Box<RefCell<AbstractizedStepFn>>;
pub(crate) type NodeC = RefCell<Node>;
pub(crate) type NodeRcc = Rc<NodeC>;

impl Node {
    pub(crate) fn equal_as_node_reference(node_a: &Node, node_b: &Node) -> bool {
        node_a.type_id == node_b.type_id && node_a.key == node_b.key
    }

    pub fn inherit_input(&self) -> AbstractizedInputBox {
        (&self.inherit_input_fn_box)(self)
    }
}

impl Into<NodeRcc> for Node {
    fn into(self) -> NodeRcc {
        Rc::new(RefCell::new(self))
    }
}
