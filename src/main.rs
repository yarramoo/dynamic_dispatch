use std::{ptr::NonNull, marker::PhantomData};

trait Speak {
    fn speak(&self);
}

struct Cat;
impl Speak for Cat {
    fn speak(&self) {
        println!("meowww");
    }
}

struct Dog;
impl Speak for Dog {
    fn speak(&self) {
        println!("WOOF");
    }
}

struct SpeakFunctions {
    speak_thunk: unsafe fn(NonNull<()>),
}

struct AnythingSpeak<'a> {
    _p: PhantomData<&'a ()>,
    data: NonNull<()>,
    functions: &'static SpeakFunctions,
}

impl<'a> AnythingSpeak<'a> {
    fn new<T: Speak>(t: &'a T) -> Self {
        Self {
            _p: PhantomData,
            data: NonNull::from(t).cast(),
            functions: &SpeakFunctions { 
                speak_thunk: |data| unsafe { data.cast::<T>().as_ref() }.speak(),
             }
        }
    }

    fn speak(&self) {
        unsafe { (self.functions.speak_thunk)(self.data) }
    }
}

fn main() {
    // Static dispatch
    let cat = Cat;
    cat.speak();
    let dog = Dog;
    dog.speak();
    // Custom dynamic dispatch
    let mut a = AnythingSpeak::new(&Cat);
    a.speak();
    a = AnythingSpeak::new(&Dog);
    a.speak();
    // In-built dynamic dispatch
    let mut b = &cat as &dyn Speak; // At compile-time a vtable is generated and stuck in static mem
    b.speak();
    b = &dog as &dyn Speak;
    b.speak();
}
