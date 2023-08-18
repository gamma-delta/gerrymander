#![doc = include_str!("../README.md")]

use std::{fmt::Display, num::NonZeroUsize};

/// Wrapper for a stack of states.
///
/// The stack will never be empty.
pub struct StateMachine<T> {
  stack: Vec<T>,
}

impl<T> StateMachine<T> {
  /// Create a new `StateMachine` with the given state on top.
  pub fn new(initial: T) -> Self {
    Self {
      stack: vec![initial],
    }
  }

  /// Create a new `StateMachine` with the given states on top. The last element of the vec
  /// will be the topmost state.
  pub fn new_many(stack: Vec<T>) -> Self {
    Self { stack }
  }

  /// Get the last element of the stack, aka the active state.
  pub fn active(&self) -> &T {
    self.stack.last().unwrap()
  }

  /// Get the last element of the stack mutably, aka the active state.
  pub fn active_mut(&mut self) -> &mut T {
    self.stack.last_mut().unwrap()
  }

  /// Get the last element of the stack and all elements under it.
  pub fn split_last(&self) -> (&[T], &T) {
    let (under, last) = self.stack.split_last().unwrap();
    (last, under)
  }

  /// Get the last element of the stack and all elements under it, mutably.
  pub fn split_last_mut(&mut self) -> (&mut [T], &mut T) {
    let (under, last) = self.stack.split_last_mut().unwrap();
    (last, under)
  }

  /// Apply the given transition. See [`Transition::apply`] for more detail.
  pub fn apply(
    &mut self,
    transition: Transition<T>,
  ) -> Result<TransitionOutcome<T>, TransitionError> {
    transition.apply(&mut self.stack)
  }

  /// Borrow the stack.
  pub fn get_stack(&self) -> &[T] {
    &self.stack
  }

  /// Mutably borrow the stack.
  pub fn get_stack_mut(&mut self) -> &mut [T] {
    &mut self.stack
  }

  /// Mutably borrow the stack vector itself.
  ///
  /// ## Safety
  ///
  /// You MUST leave at least one element in the stack. Not doing so won't cause UB, but it will cause panics,
  /// so this method is marked `unsafe`.
  pub unsafe fn get_stack_direct(&mut self) -> &mut Vec<T> {
    &mut self.stack
  }

  /// Iterate over the states from topmost (active) to bottommost.
  pub fn iter(&self) -> std::slice::Iter<T> {
    self.stack.iter()
  }

  /// Mutably iterate over the states from topmost (active) to bottommost.
  pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
    self.stack.iter_mut()
  }

  /// Consume this and return the internal stack of states.
  pub fn consume(self) -> Vec<T> {
    self.stack
  }

  /// Get how many states are in the stack.
  pub fn len(&self) -> NonZeroUsize {
    NonZeroUsize::new(self.stack.len()).unwrap()
  }

  /// To make clippy stop yelling at me.
  #[doc(hidden)]
  pub fn is_empty(&self) -> bool {
    false
  }
}

/// Iterate over the states from topmost to bottommost.
impl<T> IntoIterator for StateMachine<T> {
  type Item = T;
  type IntoIter = std::vec::IntoIter<T>;

  fn into_iter(self) -> Self::IntoIter {
    self.stack.into_iter()
  }
}

/// A transition between states.
pub enum Transition<T> {
  /// Don't do anything
  None,
  /// Push this state on top.
  Push(T),
  /// Pop the current state.
  Pop,
  /// Replace the current state with a new one.
  Swap(T),
  /// The most generic version: pop N states off the stack, then push these new ones.
  /// The last element in the vec will be the new active state.
  PopNAndPush(usize, Vec<T>),
}

impl<T> Transition<T> {
  /// Apply the transition to the given stack.
  ///
  /// If an error is returned, the stack will not be modified.
  pub fn apply(
    self,
    stack: &mut Vec<T>,
  ) -> Result<TransitionOutcome<T>, TransitionError> {
    let (pop_count, mut to_push) = match self {
      Transition::None => return Ok(TransitionOutcome::None),
      Transition::Push(s) => (0, vec![s]),
      Transition::Pop => (1, vec![]),
      Transition::Swap(s) => (1, vec![s]),
      Transition::PopNAndPush(count, states) => (count, states),
    };

    // We need to always leave at least one thing on top
    let allowed_popcnt = if to_push.is_empty() {
      stack.len() - 1
    } else {
      stack.len()
    };
    if pop_count > allowed_popcnt {
      Err(TransitionError::PoppedTooMany {
        popcnt: pop_count,
        available: allowed_popcnt,
      })?
    }

    let len = stack.len();
    let removed: Vec<T> = stack.drain(len - pop_count..).collect();

    Ok(if to_push.is_empty() {
      TransitionOutcome::Revealed(removed)
    } else {
      let len = to_push.len();
      stack.append(&mut to_push);
      if removed.is_empty() {
        TransitionOutcome::Pushed
      } else {
        TransitionOutcome::SwappedIn(removed, len - 1)
      }
    })
  }
}

/// What happened to the state stack after applying a transition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TransitionOutcome<T> {
  /// Nothing happened!
  None,
  /// A state was newly pushed to the stack.
  Pushed,
  /// The top state on the stack was revealed after things were removed from on top of it.
  /// The `Vec` has all the states that used to be on top, with the last element being the previous
  /// top of the stack.
  Revealed(Vec<T>),
  /// Things were removed from the stack, and then the new state got pushed on top along with N things below it.
  SwappedIn(Vec<T>, usize),
  // MTF
  // FTM
}

/// Something went wrong when applying a transition.
#[derive(Debug, Clone, Copy)]
pub enum TransitionError {
  /// Tried to pop too many things off the stack.
  PoppedTooMany {
    /// How many states you tried to pop
    popcnt: usize,
    /// How many states you were *allowed* to pop.
    ///
    /// If you are pushing more onto the stack, this is the length of the state machine.
    /// Otherwise, this is the length minus 1.
    available: usize,
  },
}

impl Display for TransitionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TransitionError::PoppedTooMany { popcnt, available } => write!(
        f,
        "Tried to pop {} states, but coult only pop {}",
        popcnt, available
      ),
    }
  }
}

impl std::error::Error for TransitionError {}
