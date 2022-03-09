use lazy_static::lazy_static;
use proc_macro::TokenStream;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

// pub struct Control<Param, State>
// where
//     State: Machine<Param> + ?Sized,
// {
//     param: Param,
//     data: RefCell<Box<State>>,
//     mutation_queue: RefCell<VecDeque<Box<dyn FnMut(RefMut<Box<State>>)>>>,
// }

// impl<Param, State: Machine<Param>> Control<Param, State> {
//     fn peek(&self) -> Ref<Box<State>> {
//         self.data.borrow()
//     }

//     fn mutate(&self, mutation_function: Box<dyn FnMut(RefMut<Box<State>>)>) {
//         self.mutation_queue
//             .borrow_mut()
//             .push_back(mutation_function);
//     }

//     fn step(&mut self) {
//         println!("Loop");
//         std::thread::sleep(Duration::from_millis(1000));
//         Machine::step(self);

//         let children = Machine::spawn::<_, Machine<_>>(&self);

//         while let Some(mut mutation) = self.mutation_queue.borrow_mut().pop_front() {
//             mutation(self.data.borrow_mut());
//         }
//     }
// }

// pub trait Machine<Param> {
//     fn schedule_new(param: Param) -> Box<dyn FnOnce() -> Self>;

//     fn new(params: Param) -> Self;

//     fn step(control: &mut Control<Param, Self>) -> ();

//     fn spawn<ChildParam: Sized, Child: Machine<_>>(
//         control: &Control<u8, SampleCounter>,
//     ) -> Vec<Option<Box<dyn FnOnce() -> Child>>>;
// }

// struct SampleCounter {
//     count: u8,
// }

// impl Machine<u8> for SampleCounter {
//     fn schedule_new(param: u8) -> Box<dyn FnOnce() -> SampleCounter> {
//         Box::new(move || SampleCounter::new(param))
//     }

//     fn new(params: u8) -> SampleCounter {
//         SampleCounter { count: 10 }
//     }

//     fn step(control: &mut Control<u8, SampleCounter>) {
//         if control.peek().count > 0 {
//             println!("Depth: {}, Count: {}", control.param, control.peek().count);
//             control.mutate(Box::new(|mut state| {
//                 state.count -= 1;
//             }));
//         }
//     }

//     fn spawn<ChildParam: Sized, Child: Machine<ChildParam>>(
//         control: &Control<u8, SampleCounter>,
//     ) -> Vec<Option<Box<dyn FnOnce() -> Child>>> {
//         vec![{
//             let param = control.param.clone() - 1;
//             if param > 0 {
//                 Some(SampleCounter::schedule_new(param));
//             }
//             None
//         }]
//     }
// }

// fn into_backend<Param, State>(initial_prop: Param) {
//     let stored_prop: Rc<dyn Any> = Rc::new(initial_prop);
//     let stored_state: Rc<dyn Any> = Rc::new(());

//     let stored_step_fn = || {

//     };
//     let backend = NodeBackend {
//         stored_props: stored_prop,
//         stored_state,
//         stored_step_fn,
//         children: vec![],
//     };
// }

