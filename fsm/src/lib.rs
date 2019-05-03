//! # fsm
//! a set of simple traits, and derive for enums that turns them into finite-state machines
//!
//! simply `#[derive(State)]` on your unit enums
//!
//! the enums are required to be `Clone + Copy + PartialEq + PartialOrd`
//!
//! this will provide some useful methods
//!
//! such as:
//!
//! name | description | returns
//! --- | --- | ---
//! `T::start()` | gets the first variant (e.g. start state) | the **start** state
//! `T::end()` | gets the last variant (e.g. end state) | the **end** state
//! `T::len()` | gets the number of variants | --
//! `next(&mut self)` | advances to the **next** state, wrapping around. | the previous state
//! `previous` | advances to the **previous** state, wrapping around. | the previous state
//! `goto` | goto a specific state | the previous state
//! `into_iter` | gets an iterator starting at the current state and looping infinitely | ---
//! `into_iter_once` | gets an iterator from this, starting at the current state. but only loops until the last state | ---
//! `flip` | on enums with just **two variants**, this'll toggle to the next state | the previous state
//!
//!
//! ## usage
//! ```
//! use fsm::*;
//! #[derive(Debug, State, Clone, Copy, PartialEq, PartialOrd)]
//! enum Foo {
//!     Start,
//!     Next,
//!     Rollback,
//!     Error,
//!     End,
//! }
//!
//! // you can start at arbitary positions
//! let mut foo = Foo::Rollback;
//! // the iterators are also double-ended, so you can iterate backwards
//! assert_eq!(
//!     foo.into_iter_once().rev().collect::<Vec<_>>(),
//!     vec![  
//!         Foo::Rollback,
//!         Foo::Next,
//!         Foo::Start,    
//!     ]
//! );
//!
//! // start at the end and then wrap around, forwards
//! let mut foo = Foo::end();
//! foo.next(); // will return Foo::end() (the current state before advancing)
//! assert_eq!(foo, Foo::start());
//!
//! // start at the beginning and then wrap around, backwards
//! let mut foo = Foo::start();
//! foo.previous(); // will return Foo::start() (the current state before advancing)
//! assert_eq!(foo, Foo::end());
//!
//! // jump to a specific state
//! let mut foo = Foo::Error;
//! foo.goto(Foo::Next); // will return Foo::Error
//! assert_eq!(foo, Foo::Next)
//! ```
//! ## using the auto-derived `StateFlip` trait on an enum with 2 variants
//! ```
//! use fsm::*;
//! #[derive(Debug, State, Clone, Copy, PartialEq, PartialOrd)]
//! enum Coin {
//!     Heads, Tails
//! }
//! assert_eq!(Coin::len(), 2);
//!
//! let mut state = Coin::start();
//! assert_eq!(state.flip(), Coin::Heads);
//! assert_eq!(state, Coin::Tails);
//! assert_eq!(state.flip(), Coin::Tails);
//! assert_eq!(state, Coin::Heads);
//!
//! assert_eq!(
//!     Coin::start().into_iter().take(Coin::len() * 4).collect::<Vec<_>>(),
//!     vec![
//!         Coin::Heads,
//!         Coin::Tails,
//!         Coin::Heads,
//!         Coin::Tails,
//!         Coin::Heads,
//!         Coin::Tails,
//!         Coin::Heads,
//!         Coin::Tails,
//!     ]
//! );
//!
//! assert_eq!(
//!     Coin::start().into_iter_once().collect::<Vec<_>>(),
//!     vec![Coin::Heads, Coin::Tails]
//! );
//! ```

pub use derive::*;

/// State allows an type to be used as a finite state machine
#[allow(clippy::len_without_is_empty)]
pub trait State: Sized
where
    Self: Clone + Copy + PartialEq + PartialOrd,
    Self: Into<u8> + std::convert::TryFrom<u8>,
{
    /// The maximum number of states available
    const MAX: u8;

    /// The first 'state' in the enum
    fn start() -> Self {
        Self::try_from(0).ok().unwrap()
    }

    /// The last state in the enum
    fn end() -> Self {
        Self::try_from(Self::MAX - 1).ok().unwrap()
    }

    /// Returns how many states exist
    fn len() -> usize {
        Self::MAX as usize
    }

    /// Advances to the next state, wrapping around
    fn next(&mut self) -> Self {
        let t = (*self).into();
        let next = (t + 1) % Self::MAX;
        std::mem::replace(self, Self::try_from(next).ok().unwrap())
    }

    /// Advanced to the previous state, wrapping around
    fn previous(&mut self) -> Self {
        let t = (*self).into();
        let prev = if t == 0 { Self::MAX } else { t } - 1;
        std::mem::replace(self, Self::try_from(prev).ok().unwrap())
    }

    /// Goto a specific state
    fn goto(&mut self, other: Self) -> Self {
        std::mem::replace(self, other)
    }

    /// Create an (infinite) iterator from this
    ///
    /// Will advance to each state, wrapping start -- starting from the current state
    fn into_iter(self) -> StateIter<Self> {
        let pos: u8 = self.into();
        StateIter {
            item: self,
            pos: pos as usize,
            infinite: true,
            done: false,
        }
    }

    /// Create an (fixed point) iterator from this
    ///
    /// Will advance until the last state is reached, starting from the current state.
    fn into_iter_once(self) -> StateIter<Self> {
        let pos: u8 = self.into();
        StateIter {
            item: self,
            pos: pos as usize,
            infinite: false,
            done: false,
        }
    }
}

/// This is useful for enums of two states (a coin flip, as a prime example)
pub trait StateFlip: State {
    /// Toggle to the next state
    fn flip(&mut self) -> Self {
        self.next()
    }
}

/// An iterator over the state
pub struct StateIter<T>
where
    T: State,
{
    item: T,
    pos: usize,
    infinite: bool,
    done: bool,
}

impl<T> Iterator for StateIter<T>
where
    T: State,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if !self.infinite && self.item == T::end() {
            self.done = true;
        }
        let next = self.item.next();
        let p: u8 = next.into();
        self.pos = p as usize;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.pos, Some(T::MAX as usize))
    }
}

impl<T> ExactSizeIterator for StateIter<T> where T: State {}

impl<T> DoubleEndedIterator for StateIter<T>
where
    T: State,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if !self.infinite && self.item == T::start() {
            self.done = true;
        }
        let next = self.item.previous();
        let p: u8 = next.into();
        self.pos = p as usize;
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_state() {
        #[derive(Debug, State, Clone, Copy, PartialEq, PartialOrd)]
        enum Coin {
            Heads,
            Tails,
        }
        assert_eq!(Coin::len(), 2);

        let mut state = Coin::start();
        assert_eq!(state, Coin::Heads);
        assert_eq!(state.flip(), Coin::Heads);
        assert_eq!(state, Coin::Tails);
        assert_eq!(state.flip(), Coin::Tails);
        assert_eq!(state, Coin::Heads);
    }

    #[test]
    fn many() {
        #[derive(Debug, State, Clone, Copy, PartialEq, PartialOrd)]
        enum Foo {
            Start,
            Do,
            Something,
            Else,
            End,
        }

        assert_eq!(Foo::len(), 5);
        assert_eq!(Foo::start().into_iter_once().count(), Foo::len());
        assert_eq!(Foo::Something.into_iter_once().count(), 3);
        assert_eq!(Foo::End.into_iter_once().rev().count(), Foo::len());
    }
}
