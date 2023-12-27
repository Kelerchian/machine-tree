use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard, PoisonError, Weak},
};

use crate::{node::SelfRender, node_host::NodeControl};

pub struct SeedData {
    pub(crate) input: AnyBox,
    pub(crate) inherit_input_fn_box: CloneInputBox,
    pub(crate) step_fn: BoxedAbsStep,
}

pub struct Seed {
    // Goes to NodeKey
    pub(crate) key: RawKey,
    // Goes to NodeData
    pub(crate) data: SeedData,
}

impl Seed {
    pub(crate) fn clone_input(&self) -> AnyBox {
        (self.data.inherit_input_fn_box)(&self.data.input)
    }

    pub(crate) fn sprout(self) -> (RawKey, RawData) {
        let Seed {
            key,
            data: SeedData { input, step_fn, .. },
        } = self;
        (
            key,
            RawData {
                input: input,
                step_fn: step_fn,
            },
        )
    }
}

// NodeData is !Sync + !Send
pub struct RawData {
    pub(crate) input: AnyBox,
    pub(crate) step_fn: BoxedAbsStep,
}

impl Into<DataRc> for RawData {
    fn into(self) -> DataRc {
        Rc::new(RefCell::new(self))
    }
}

// Node is Sync + Send
pub struct RawKey {
    pub(crate) type_id: TypeId,
    pub(crate) key: Option<String>,
    pub(crate) self_render: SelfRender,
}

impl Hash for RawKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
        self.key.hash(state);
    }
}

impl RawKey {
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

#[derive(Clone)]
pub struct Key(pub(crate) KeyArc);

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.read_ptr_as_usize().hash(state);
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Key {}

impl From<KeyArc> for Key {
    fn from(node_rc: KeyArc) -> Self {
        Key(node_rc)
    }
}

impl From<&KeyArc> for Key {
    fn from(node_rc: &KeyArc) -> Self {
        Key(node_rc.clone())
    }
}

impl TryFrom<&KeyWeak> for Key {
    type Error = ();

    fn try_from(value: &KeyWeak) -> Result<Self, Self::Error> {
        value.upgrade().map(|x| Key::from(x)).ok_or(())
    }
}

impl Into<KeyWeak> for &Key {
    fn into(self) -> KeyWeak {
        Arc::downgrade(&self.0)
    }
}

impl Key {
    pub(crate) fn new_from_raw(raw: RawKey) -> Key {
        Key(Arc::new(Mutex::new(raw)))
    }

    pub(crate) fn read_ptr_as_usize(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    pub(crate) fn lock<'a>(
        &'a self,
    ) -> Result<MutexGuard<'a, RawKey>, PoisonError<MutexGuard<'a, RawKey>>> {
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

impl Into<KeyArc> for RawKey {
    fn into(self) -> KeyArc {
        Arc::new(Mutex::new(self))
    }
}

// TODO: make StepFn more accomodating toward Self Struct, and not relying on FnMut
pub(crate) type AnyBox = Box<dyn Any>;
pub(crate) type CloneInput = fn(&AnyBox) -> AnyBox;
pub(crate) type CloneInputBox = Box<CloneInput>;
pub(crate) type Step<Input> = Box<dyn FnMut(&mut NodeControl, &Input) -> Vec<Seed>>;
pub(crate) type AbsStep = Step<AnyBox>;
pub(crate) type BoxedAbsStep = Box<RefCell<AbsStep>>;
pub(crate) type KeyMutex = Mutex<RawKey>;
pub(crate) type KeyArc = Arc<KeyMutex>;
pub(crate) type KeyWeak = Weak<KeyMutex>;
pub(crate) type DataRc = Rc<RefCell<RawData>>;
