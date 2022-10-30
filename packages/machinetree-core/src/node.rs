use crate::node_host::NodeControl;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard, PoisonError, Weak},
};

mod component_utils {
    use crate::node_host::NodeControl;

    use super::{AbstractInputBox, AbstractStepFn, Component};

    pub fn downcast_as_input_ref<'a, Input>(input: &'a AbstractInputBox) -> &'a Input
    where
        Input: Sized + Clone + 'static,
    {
        input.downcast_ref::<Input>().unwrap()
    }

    pub fn clone_input_box<Input>(abstract_input_box: &AbstractInputBox) -> AbstractInputBox
    where
        Input: Sized + Clone + 'static,
    {
        Box::new(downcast_as_input_ref::<Input>(&abstract_input_box).clone())
    }

    pub fn generate_abstract_step_fn<Machine, Input>(input: &Input) -> AbstractStepFn
    where
        Input: Sized + Clone + 'static,
        Machine: Sized + Component<Input = Input> + 'static,
    {
        let mut self_state = Machine::construct(input);
        Box::new(move |control: &mut NodeControl, input: &AbstractInputBox| {
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

    fn step(&mut self, control: &mut NodeControl, input: &Self::Input) -> Vec<NodeSeed>;

    fn seed(input: Self::Input, key: String) -> NodeSeed {
        let type_id = TypeId::of::<Self>();
        let step_fn: AbstractStepFnBrc =
            Box::new(RefCell::new(component_utils::generate_abstract_step_fn::<
                Self,
                Self::Input,
            >(&input)));
        let input: AbstractInputBox = Box::new(input);
        let inherit_input_fn_box: CloneInputFnBox =
            Box::new(component_utils::clone_input_box::<Self::Input>);
        let self_render_signaler = Default::default();

        NodeSeed {
            type_id,
            key,
            input,
            inherit_input_fn_box,
            step_fn,
            self_render_signaler,
        }
    }
}

#[derive(Clone)]
pub struct NodeSelfRenderSignalerSet {
    pub(crate) sender: crossbeam::channel::Sender<NodeKey>,
    pub(crate) self_key: NodeKey,
}

#[derive(Clone)]
pub enum NodeSelfRenderSignaler {
    Empty,
    Set(NodeSelfRenderSignalerSet),
}

impl Default for NodeSelfRenderSignaler {
    fn default() -> Self {
        NodeSelfRenderSignaler::Empty
    }
}

impl NodeSelfRenderSignaler {
    pub(crate) fn set_self(
        &mut self,
        node_key: &NodeKey,
        sender: &crossbeam::channel::Sender<NodeKey>,
    ) {
        if let NodeSelfRenderSignaler::Set(_) = &self {
            return;
        }

        *self = NodeSelfRenderSignaler::Set(NodeSelfRenderSignalerSet {
            sender: sender.clone(),
            self_key: node_key.clone(),
        });
    }

    pub fn rerender(&self) -> Result<(), ()> {
        match self {
            NodeSelfRenderSignaler::Set(signaler) => signaler
                .sender
                .send(signaler.self_key.clone())
                .map_err(|_| {}),
            _ => Ok(()),
        }
    }
}

pub struct NodeSeed {
    // Goes to Node
    pub(crate) type_id: TypeId,
    pub(crate) key: String,
    pub(crate) self_render_signaler: NodeSelfRenderSignaler,

    // Goes to NodeData
    pub(crate) input: AbstractInputBox,
    pub(crate) inherit_input_fn_box: CloneInputFnBox,
    pub(crate) step_fn: AbstractStepFnBrc,
}

impl NodeSeed {
    pub(crate) fn clone_input(&self) -> AbstractInputBox {
        (self.inherit_input_fn_box)(&self.input)
    }
}

// NodeData is !Sync + !Send
pub struct NodeDataRaw {
    pub(crate) input: AbstractInputBox,
    pub(crate) step_fn: AbstractStepFnBrc,
}

impl Into<NodeDataRc> for NodeDataRaw {
    fn into(self) -> NodeDataRc {
        Rc::new(RefCell::new(self))
    }
}

// Node is Sync + Send
pub struct NodeKeyRaw {
    pub(crate) type_id: TypeId,
    pub(crate) key: String,
    pub(crate) self_render_signaler: NodeSelfRenderSignaler,
}

impl NodeKeyRaw {
    // pub(crate) fn equal_as_node_reference(node_a: &NodeKeyRaw, node_b: &NodeKeyRaw) -> bool {
    //     node_a.type_id == node_b.type_id && node_a.key == node_b.key
    // }

    pub(crate) fn get_type_id_string(&self) -> String {
        format!("{:#?}", &self.type_id)
    }

    pub(crate) fn get_key_string(&self) -> String {
        format!("{:#?}", &self.key)
    }
}

// TODO: make StepFn more accomodating toward Self Struct, and not relying on FnMut
pub type StepFn<Input> = Box<dyn FnMut(&mut NodeControl, &Input) -> Vec<NodeSeed>>;
pub(crate) type InputBox<Input> = Box<Input>;
pub(crate) type AbstractInputBox = InputBox<dyn Any>;
pub(crate) type CloneInputFn = fn(&AbstractInputBox) -> AbstractInputBox;
pub(crate) type CloneInputFnBox = Box<CloneInputFn>;
pub(crate) type AbstractStepFn = StepFn<AbstractInputBox>;
pub(crate) type AbstractStepFnBrc = Box<RefCell<AbstractStepFn>>;
pub(crate) type NodeMutex = Mutex<NodeKeyRaw>;
pub(crate) type NodeKeyArc = Arc<NodeMutex>;
pub(crate) type NodeKeyWeak = Weak<NodeMutex>;
pub(crate) type NodeDataRc = Rc<RefCell<NodeDataRaw>>;

#[derive(Clone)]
pub struct NodeKey(pub(crate) NodeKeyArc);

impl Hash for NodeKey {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.read_ptr_as_usize());
    }
}

impl PartialEq for NodeKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NodeKey {}

impl From<NodeKeyArc> for NodeKey {
    fn from(node_rc: NodeKeyArc) -> Self {
        NodeKey(node_rc)
    }
}

impl From<&NodeKeyArc> for NodeKey {
    fn from(node_rc: &NodeKeyArc) -> Self {
        NodeKey(node_rc.clone())
    }
}

impl TryFrom<&NodeKeyWeak> for NodeKey {
    type Error = ();

    fn try_from(value: &NodeKeyWeak) -> Result<Self, Self::Error> {
        value.upgrade().map(|x| NodeKey::from(x)).ok_or(())
    }
}

impl Into<NodeKeyWeak> for &NodeKey {
    fn into(self) -> NodeKeyWeak {
        Arc::downgrade(&self.0)
    }
}

impl NodeKey {
    pub(crate) fn read_ptr_as_usize(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    pub(crate) fn lock<'a>(
        &'a self,
    ) -> Result<MutexGuard<'a, NodeKeyRaw>, PoisonError<MutexGuard<'a, NodeKeyRaw>>> {
        self.0.lock()
    }

    pub fn debug_attempt_get_name(&self) -> String {
        match self.lock() {
            Ok(node_key_raw) => format!(
                "{:#?}:{:#?}",
                &node_key_raw.get_type_id_string(),
                &node_key_raw.get_key_string()
            ),
            Err(_) => format!("unidentifiable"),
        }
    }
}

pub(crate) enum WorkItem {
    Render(NodeKey),
}

impl NodeSeed {
    pub(crate) fn split(self) -> (NodeKeyRaw, NodeDataRaw) {
        let NodeSeed {
            type_id,
            key,
            self_render_signaler,
            input,
            inherit_input_fn_box: _,
            step_fn,
        } = self;

        (
            NodeKeyRaw {
                type_id,
                key,
                self_render_signaler,
            },
            NodeDataRaw { input, step_fn },
        )
    }
}

impl Into<NodeKeyArc> for NodeKeyRaw {
    fn into(self) -> NodeKeyArc {
        Arc::new(Mutex::new(self))
    }
}
