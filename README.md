# fsm

a set of simple traits, and derive for enums that turns them into finite-state machines

simply `#[derive(State)]` on your unit enums

the enums are required to be `Clone + Copy + PartialEq + PartialOrd`

this will provide some useful methods

such as:

name | description | returns
--- | --- | ---
`T::start()` | gets the first variant (e.g. start state) | the **start** state
`T::end()` | gets the last variant (e.g. end state) | the **end** state
`T::len()` | gets the number of variants | --
`next(&mut self)` | advances to the **next** state, wrapping around. | the previous state
`previous` | advances to the **previous** state, wrapping around. | the previous state
`goto` | goto a specific state | the previous state
`into_iter` | gets an iterator starting at the current state and looping infinitely | ---
`into_iter_once` | gets an iterator from this, starting at the current state. but only loops until the last state | ---
`flip` | on enums with just **two variants**, this'll toggle to the next state | the previous state


### usage
```rust
use fsm::*;
#[derive(Debug, State, Clone, Copy, PartialEq, PartialOrd)]
enum Foo {
    Start,
    Next,
    Rollback,
    Error,
    End,
}

// you can start at arbitary positions
let mut foo = Foo::Rollback;
// the iterators are also double-ended, so you can iterate backwards
assert_eq!(
    foo.into_iter_once().rev().collect::<Vec<_>>(),
    vec![
        Foo::Rollback,
        Foo::Next,
        Foo::Start,
    ]
);

// start at the end and then wrap around, forwards
let mut foo = Foo::end();
foo.next(); // will return Foo::end() (the current state before advancing)
assert_eq!(foo, Foo::start());

// start at the beginning and then wrap around, backwards
let mut foo = Foo::start();
foo.previous(); // will return Foo::start() (the current state before advancing)
assert_eq!(foo, Foo::end());

// jump to a specific state
let mut foo = Foo::Error;
foo.goto(Foo::Next); // will return Foo::Error
assert_eq!(foo, Foo::Next)
```
### using the auto-derived `StateFlip` trait on an enum with 2 variants
```rust
use fsm::*;
#[derive(Debug, State, Clone, Copy, PartialEq, PartialOrd)]
enum Coin {
    Heads, Tails
}
assert_eq!(Coin::len(), 2);

let mut state = Coin::start();
assert_eq!(state.flip(), Coin::Heads);
assert_eq!(state, Coin::Tails);
assert_eq!(state.flip(), Coin::Tails);
assert_eq!(state, Coin::Heads);

assert_eq!(
    Coin::start().into_iter().take(Coin::len() * 4).collect::<Vec<_>>(),
    vec![
        Coin::Heads,
        Coin::Tails,
        Coin::Heads,
        Coin::Tails,
        Coin::Heads,
        Coin::Tails,
        Coin::Heads,
        Coin::Tails,
    ]
);

assert_eq!(
    Coin::start().into_iter_once().collect::<Vec<_>>(),
    vec![Coin::Heads, Coin::Tails]
);
```
