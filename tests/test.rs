use gerrymander::*;

#[test]
fn testing() {
  let mut machine = StateMachine::<&str>::new("bottom");
  assert_eq!(*machine.active(), "bottom");

  let res = machine.apply(Transition::Push("1"));
  assert_eq!(res, Ok(TransitionOutcome::Pushed));

  let res = machine.apply(Transition::Push("2"));
  assert_eq!(res, Ok(TransitionOutcome::Pushed));

  let res = machine.apply(Transition::Swap("3"));
  assert_eq!(res, Ok(TransitionOutcome::SwappedIn(vec!["2"], 0)));

  let res = machine.apply(Transition::Pop);
  assert_eq!(res, Ok(TransitionOutcome::Revealed(vec!["3"])));

  let res =
    machine.apply(Transition::PopNAndPush(0, vec!["10", "11", "12", "13"]));
  assert_eq!(res, Ok(TransitionOutcome::Pushed));
  assert_eq!(*machine.active(), "13");

  // notably, this transition would have left the machine empty
  // if it didn't add more.
  // this should be allowed.
  let res = machine.apply(Transition::PopNAndPush(6, vec!["a", "b", "c"]));
  assert_eq!(
    res,
    Ok(TransitionOutcome::SwappedIn(
      vec!["bottom", "1", "10", "11", "12", "13"],
      2
    ))
  );
  assert_eq!(machine.get_stack(), &["a", "b", "c"]);

  let res = machine.apply(Transition::PopNAndPush(100, vec![]));
  assert_eq!(
    res,
    Err(TransitionError::PoppedTooMany {
      popcnt: 100,
      // could only pop 2 off a stack of 3 so the bottom is always OK
      available: 2
    })
  );
  // error should not modify the stack
  assert_eq!(machine.get_stack(), &["a", "b", "c"]);
}

#[cfg(feature = "serde")]
#[test]
fn serdeez_nuts() {
  let machine = StateMachine::new_many(
    ["alpha", "beta", "gamma", "delta", "epsilon"]
      .into_iter()
      .map(String::from)
      .collect(),
  );

  let jsonified = serde_json::to_string(&machine).unwrap();
  let unjsonified: StateMachine<String> =
    serde_json::from_str(&jsonified).unwrap();

  let stack1 = machine.consume();
  let stack2 = unjsonified.consume();
  assert_eq!(stack1, stack2);
}
