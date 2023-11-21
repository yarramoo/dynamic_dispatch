Following the guide from [[https://www.youtube.com/watch?v=wU8hQvU8aKM&ab_channel=LoganSmith|this]] video, I wanted to take notes and investigate some other sources to understand it better.
# Rust
## Structure
Experiment using type erasure. Create a struct that holds a reference to something that implements `trait Speak`.

```
struct AnythingSpeak<'a> {
    _p: PhantomData<&'a ()>,
    data: NonNull<()>,
}
```

This is our struct. The data is a NonNull pointer to something that implements Speak. We don't know what it will be because we may introduce structures in the future that implement speak, so we just use the unit `()` to represent this unknown. Since it is a reference we need a lifetime that we bind to the structure via `PhantomData<&'a ()'>` 

However we don't know what kind of thing we are pointing to in any instance of our structure. How can we call a function it implements? We can add the function pointer to our structure!

`speak_thunk: unsafe fn(NonNull<()>),`

Why unsafe? It is unsafe because of the way we are using our structure. We may only invoke this function with the argument in the `data` field, and no other `NonNull<()>`. This property must be enforced by the programmer. 

## Implementation
We can then provide the following implementation...
```
impl<'a> AnythingSpeak<'a> {
    fn new<T: Speak>(t: &'a T) -> Self {
        Self {
            _p: PhantomData,
            data: NonNull::from(t).cast(),
            speak_thunk: |data| unsafe { data.cast::<T>().as_ref() }.speak(),
        }
    }
}
```
The `_p` and `data` fields are fairly straightforward. The `data` field just casts a `NonNull<T>` to a `NonNull<()>`. 
`speak_thunk` is more interesting. The value that we are giving it is a closure, but in Rust closures that do not capture can be cast to function pointers. Since everything here can be evaluated at compile-time, this is okay. 

I tried for a while to get a function pointer direct to `T::speak`, however I was not able to get anything that compiled. It seemed ugly to me to create a new function just to wrap the type's `speak()` function that plugged in a `NonNull<()>` argument. However, `T::speak as fn(NonNull<()>)` was an invalid cast and this makes sense. The stack space needed between `NonNull<()>` and `&self` may be entirely different so we would need a different function.

If I was slightly more patient in the video I would know that thunks are often used for bookkeeping before or after a function. This is what the closure is doing here - we only have a reference to unit, so we need a wrapper to cast to the appropriate type, before finally calling the correct `speak()` method. 

Now we can add the `speak()` method on our generic of `Speak` that just calls `speak_thunk` with `data`. 

```
impl<'a> AnythingSpeak<'a> {
    fn speak(&self) {
        unsafe { (self.speak_thunk)(self.data) }
    }
}
```

Now we have this situation in `main`
```
    Cat.speak();
    Dog.speak();
    let mut a = AnythingSpeak::new(&Cat);
    a.speak();
    a = AnythingSpeak::new(&Dog);
    a.speak();
```
Which results in 
```
meowww
WOOF
meowww
WOOF
```
We can see direct function calls to the implementations of `speak()` on the Dog and Cat-typed values. We can also create a single value of type `AnythingSpeak` that correctly identifies the implementing methods on Cat and Dog when constructed with a reference to values of those types. 

### Problem 
If we add any methods to the `Speak` trait, we must also add a field to the `AnythingSpeak` structure, which is a bit ugly. How can we avoid this?

A solution is to store a pointer to another structure that holds all the relevant function pointers. 
```
struct SpeakFunctions {
    speak_thunk: unsafe fn(NonNull<()>),
}
```
And now our `AnythingSpeak` struct looks like
```
struct AnythingSpeak<'a> {
    _p: PhantomData<&'a ()>,
    data: NonNull<()>,
    functions: &'static SpeakFunctions,
}
```
with implementation
```
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
```
Why can we have a `&'static` reference? That seems surprising since we create instances of `SpeakFunctions` when we need different concrete types that implement `Speak`, right? 

We can see in the implementation that we seem to be creating an instance of the `SpeakFunctions` struct and are just taking a reference to a local variable. This is all kinds of weird...

The term is **constant promotion** or **rvalue static promotion** - if a runtime value is seen to not depend on any runtime variables, instead only consisting of information known *at compile time*, a reference to it may be promoted to whatever lifetime you want and the value is put in static memory. 

This effectively lets us express a *constant* that is associated with a *type*. Pretty.

And now, when we instantiate an `AnythingSpeaks`, the size of the variable on the stack does not scale with the number of methods. It's just a fat pointer. 

# `&dyn Speak`
So it turns out that what we were implementing all along was this type: `&dyn Speak`, which contains a pointer to the base type, and a pointer to a `vtable` that holds references to the implementations of `Speak` functions. 

A `vtable` has some differences however. 
- It holds by default references to `Drop::drop` and drop glue so that we can deallocate the base type unconditionally via the type-erased trait object
- It holds `size` and `alignment`
- Entries in the `vtable` directly reference the base object's functions, rather than referencing indirectly through a thunk. We needed this because we had to cast the object to `T: Speak` to play by the type system rules, but the compiler doesn't worry about that stuff. 

https://articles.bchlr.de/traits-dynamic-dispatch-upcasting

![[imgs/Screenshot 2023-11-21 at 12.49.16 pm.png|400]]
The article here also mentions that the size and alignment are needed to implement `std::mem::size_of_val` and `std::mem::align_of_val`, as well as for codegen. 

There are more implications for upcasting in Rust vs OO languages. If we have a variable of type `&dyn SubTrait`, we cannot pass it to a function that takes a `&dyn SuperTrait` because to create the `vtable` for this super-trait-object, we must know the concrete type, which the compiler does not. 

There are workarounds that involve coercing references to types that implement `Super + Sized` into a `Ref<dyn Super + 'a>`, but this cannot be done for dynamically sized types. 

# C++
## Implementation
C++ uses a different approach that involves declaring methods on a class as `virtual` which indicates the intention to use this method via dynamic dispatch. This way the `vtable` is built when the class is declared, rather than used in a dynamic way. 
```
class Animal:
    int data;
public:
    virtual void speak();
};
```
This is much more intrusive in that you must decide beforehand how you want to call your methods, and each instance of an object also carries the overhead of a pointer to the `vtable` even if you only want the methods in a static context for some instances of the class. 
- We can see this by checking `sizeof()` for types that use virtual functions. Note the size even when we don't use the virtual functions. Pretty cool. 

Additionally in C++ you have to manually implement a virtual destructor but that's a language choice. 

However, there seem to be upsides to this approach. C++ is able to handle upcasting much easier, likely because at the stage where we build the `vtable`, full type information is known so we can easily insert inherited virtual methods directly into the constructed `vtable`.

