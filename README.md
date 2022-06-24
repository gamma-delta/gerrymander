# Gerrymander

Push-down state machine for Rust, intended for game states.

Here's a fairly exhaustive example: 

```rust
# use gerrymander::*;
// Create a state machine with the given state on top.
let mut sm = StateMachine::new("loading");

// Apply a `Transition` to the state machine.
// You might return a `Transition` from your gamestates' `update` function, for example.
// This transition is the simplest: it just does nothing.
let res = sm.apply(Transition::None).unwrap();
assert_eq!(res, TransitionOutcome::None);

// Swap the top state for a different state.
let res = sm.apply(Transition::Swap("playing")).unwrap();
// The topmost state of the stack is considered the "active" state.
// This is what you should be calling your `update` or `draw` or what-have-you functions on.
assert_eq!(*sm.active(), "playing");
// Applying a transition also returns a little bit of information about 
// what the transition did.
// This is for if you want your states to react to being revealed, or whatever.
// In this case, we removed the state `loading`, and added 0 other states besides 
// the new `title_screen` state.
assert_eq!(res, TransitionOutcome::SwappedIn(vec!["loading"], 0));

// The power of push-down state machines comes from, well, pushing down.
// We push a new state on *top* of the old `playing` state; it's still there, just hidden...
let res = sm.apply(Transition::Push("inventory")).unwrap();
assert_eq!(res, TransitionOutcome::Pushed);
// and now the `inventory` state is what's happening.
assert_eq!(*sm.active(), "inventory");

// And then we go back to playing.
let res = sm.apply(Transition::Pop).unwrap();
// The new topmost state, `playing` was revealed/resumed, 
// and we popped off `inventory` to get there.
assert_eq!(res, TransitionOutcome::Revealed(vec!["inventory"]));

// Push a state, again.
let res = sm.apply(Transition::Push("pause")).unwrap();
assert_eq!(res, TransitionOutcome::Pushed);
// In case you want to, for example, render things under the topmost state, 
// you can split the stack into the topmost state and any states under it easily.
// No unwrap is needed because the state machine will always have at least one state in it.
let (under, top) = sm.split_last();
assert_eq!(*top, "pause");
assert_eq!(under, &["playing"]);

// For more power, you can use `PopNAndPush`.
// In this case, we are popping 0 states, and pushing 3.
let res = sm
    .apply(Transition::PopNAndPush(
        0,
        vec!["menu", "submenu", "subsubmenu"],
    ))
    .unwrap();
// We didn't reveal any states, so the outcome is still like we pushed.
// Just like `Transition::Push`!
assert_eq!(res, TransitionOutcome::Pushed);
assert_eq!(sm.get_stack(), &["playing", "pause", "menu", "submenu", "subsubmenu"]);

// Here we pop two states and push 0.
let res = sm.apply(Transition::PopNAndPush(2, vec![])).unwrap();
assert_eq!(
    res,
    TransitionOutcome::Revealed(vec!["submenu", "subsubmenu"])
);

// Here, we both pop and push.
// We pop the `menu` state, and one other state was pushed besides the topmost one
// (which is now `other_submenu`);
let res = sm
    .apply(Transition::PopNAndPush(
        1,
        vec!["other_menu", "other_submenu"],
    ))
    .unwrap();
assert_eq!(res, TransitionOutcome::SwappedIn(vec!["menu"], 1));

// And pop all the menus ...
let res = sm.apply(Transition::Pop).unwrap();
assert_eq!(res, TransitionOutcome::Revealed(vec!["other_submenu"]));
let res = sm.apply(Transition::Pop).unwrap();
assert_eq!(res, TransitionOutcome::Revealed(vec!["other_menu"]));
let res = sm.apply(Transition::Pop).unwrap();
assert_eq!(res, TransitionOutcome::Revealed(vec!["pause"]));
// ... back down to playing.
assert_eq!(sm.get_stack(), &["playing"]);

// And the machine will throw an error if you try to, for example, pop too many states.
// As the stack only has one element in it right now, we can't pop anything.
let err = sm.apply(Transition::Pop);
assert!(matches!(err, Err(TransitionError::PoppedTooMany { available: 0, .. })));
```