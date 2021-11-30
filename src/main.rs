use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::time::Duration;
// When Node imports Hook it must explicitly call that it is a hook

pub struct Control<Param: 'static + Sized + Clone, State: Machine<Param> + ?Sized> {
    param: Param,
    data: RefCell<Box<State>>,
    mutation_queue: RefCell<VecDeque<Box<dyn FnMut(RefMut<Box<State>>)>>>,
}

impl<Param: 'static + Sized + Clone, State: Machine<Param>> Control<Param, State> {
    fn peek(&self) -> Ref<Box<State>> {
        self.data.borrow()
    }

    fn mutate(&self, mutation_function: Box<dyn FnMut(RefMut<Box<State>>)>) {
        self.mutation_queue
            .borrow_mut()
            .push_back(mutation_function);
    }
}

pub trait Machine<Param: 'static + Sized + Clone> {
    fn render(param: &Param) -> Box<dyn FnMut() -> Box<Self>> {
        let cloned_param = param.clone();
        Box::new(move || Self::new(&cloned_param))
    }
    fn new(params: &Param) -> Box<Self>;
    fn step(control: &Control<Param, Self>) -> ();
}

struct SampleCounter {
    count: u8,
}

impl Machine<u8> for SampleCounter {
    fn new(_params: &u8) -> Box<SampleCounter> {
        Box::new(SampleCounter { count: 10 })
    }

    fn step(control: &Control<u8, SampleCounter>) {
        if control.peek().count > 0 {
            println!("Depth: {}, Count: {}", control.param, control.peek().count);
            control.mutate(Box::new(|mut state| {
                state.count -= 1;
            }));
        }
    }
}

fn main() {
    let control = Control {
        param: 3u8,
        data: RefCell::new(Box::new(SampleCounter { count: 10 })),
        mutation_queue: RefCell::new(VecDeque::new()),
    };

    loop {
        println!("Loop");
        std::thread::sleep(Duration::from_millis(1000));
        Machine::step(&control);

        loop {
            let maybe_mutation = control.mutation_queue.borrow_mut().pop_front();
            match maybe_mutation {
                Some(mut mutation) => {
                    mutation(control.data.borrow_mut());
                },
                None => break,
            }
        }
    }
}
