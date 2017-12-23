use std::rc::Rc;
use std::cell::Cell;

use continuations::Continuation;
use runtime::Runtime;


///////////////////////////////////////////////////////////////////////////////////////////////////
// PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A reactive process.
pub trait Process: 'static {
  /// The value created by the process.
  type Value;

  /// Executes the reactive process in the runtime, calls `next` with the resulting value.
  fn call<C>(self, runtime: &mut Runtime, next: C)
  where
    C: Continuation<Self::Value>;

  /// Returns a process which waits an instant before running.
  fn pause(self) -> PauseProcess<Self>
  where
    Self: Sized
  {
    PauseProcess { process: self }
  }

  /// Returns a process which applies the given function to its value
  /// before passing the result to the continuation.
  fn map<F, O>(self, function: F) -> MapProcess<Self, F>
  where
    Self: Sized,
    F: FnOnce(Self::Value) -> O + 'static
  {
    MapProcess { process: self, function: function }
  }

  /// Returns a process which run the process returned by itself.
  fn flatten(self) -> FlattenProcess<Self>
  where
    Self: Sized,
    Self::Value: Process
  {
    FlattenProcess { process: self }
  }

  /// Successively applies `map` and `flatten`.
  fn and_then<F, O>(self, function: F) -> FlattenProcess<MapProcess<Self, F>>
  where
    Self: Sized,
    F: FnOnce(Self::Value) -> O + 'static,
    O: Process
  {
    self.map(function).flatten()
  }

  /// Return a process which run two sub-processes and waits for both to terminate,
  /// so it can give a couple formed by both results to the continuation it is given.
  fn join<P, V>(self, process: P) -> JoinProcess<Self, P>
  where
    Self: Sized,
    P: Process<Value = V>
  {
    JoinProcess { process_1: self, process_2: process }
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// MUTABLE PROCESSES
///////////////////////////////////////////////////////////////////////////////////////////////////

// Hack by Mathieu Fehr, for testing Value field equality on Process
pub trait Is {
    type Value;
}

impl<T> Is for T {
    type Value = T;
}

/// A process that can be executed multiple times, modifying its environement each time.
pub trait ProcessMut: Process {
  /// Executes the mutable process in the runtime, then calls `next` with the process and the
  /// process's return value.
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where
    Self: Sized,
    C: Continuation<(Self, Self::Value)>;

  fn while_loop<T>(self) -> WhileProcess<Self> where
    Self: Sized,
    Self::Value: Is<Value = LoopStatus<T>>
  {
    WhileProcess { process: self }
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// UTILITY FUNCTIONS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Execute the given process in a freshly created `Runtime`,
/// and return the result value.
pub fn execute_process<P, V>(process: P) -> V
where
  P: Process<Value = V>,
  V: 'static // + ::std::fmt::Debug
{
  let mut runtime = Runtime::new();

  let mut return_value = Rc::new(Cell::new(None));
  let mut return_value_clone = return_value.clone();

  let main_continuation = move |r: &mut Runtime, v: ()| {
    process.call(r, move |r: &mut Runtime, v: V| {
      // println!("Return value has been computed: {:?}", v);
      return_value.set(Some(v));
    });
  };

  runtime.on_current_instant(Box::new(main_continuation));
  runtime.execute();

  return_value_clone.take().unwrap()
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// VALUE PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A process returning a single value it holds.
pub struct ValueProcess<V> {
  value: V
}


/// Returns a new `ValueProcess` containing the given value.
pub fn value<V> (value: V) -> ValueProcess<V> {
  ValueProcess { value: value }
}


impl<V> Process for ValueProcess<V>
where
  V: 'static
{
  type Value = V;

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    next.call(runtime, self.value);
  }
}


impl<V> ProcessMut for ValueProcess<V>
where
  V: Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    let value = self.value.clone();
    next.call(runtime, (self, value));
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// PAUSE PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A process pausing one instant before calling itself.
pub struct PauseProcess<P> {
  process: P
}

impl<P> Process for PauseProcess<P>
where
  P: Process + 'static
{
  type Value = P::Value;

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    self.process.call(runtime, next.pause());
  }
}


impl<P, V> ProcessMut for PauseProcess<P>
where
  P: ProcessMut<Value = V>,
  V: 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    self.process.call_mut(runtime, |r: &mut Runtime, (p, v): (P, V)| {
      next.pause().call(r, (p.pause(), v))
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// MAP PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A process applying a function to its output value.
pub struct MapProcess<P, F> {
  process: P,
  function: F
}

impl<P, F, I, O> Process for MapProcess<P, F>
where
  P: Process<Value = I>,
  F: FnOnce(I) -> O + 'static
{
  type Value = O;

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    self.process.call(runtime, next.map(self.function));
  }
}


impl<P, F, I, O> ProcessMut for MapProcess<P, F>
where
  P: ProcessMut<Value = I>,
  F: FnMut(I) -> O + 'static,
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    let mut f = self.function;

    self.process.call_mut(runtime, move |r: &mut Runtime, (p, v): (P, I)| {
      let value = f(v);
      next.call(r, (p.map(f), value));
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// FLATTEN PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A process calling the process it contains, and giving the resulting value to itself.
pub struct FlattenProcess<PP> {
  process: PP
}

impl<PP, P> Process for FlattenProcess<PP>
where
  PP: Process<Value = P>,
  P: Process
{
  type Value = P::Value;

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    self.process.call(runtime, |runtime: &mut Runtime, value: P| {
      value.call(runtime, next);
    });
  }
}


impl<PP, P, V> ProcessMut for FlattenProcess<PP>
where
  PP: ProcessMut<Value = P>,
  P:  ProcessMut<Value = V>
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    self.process.call_mut(runtime, |runtime: &mut Runtime, (pp, p): (PP, P)| {
      p.call_mut(runtime, |r: &mut Runtime, (p, v): (P, V)| {
        next.call(r, (pp.flatten(), v));
      });
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// JOIN PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A process calling two sub-processes before calling the next process with both returned values.

/// A helper structure, used by `JoinProcess` to synchronize the call of two processes.
///
/// This version is specific to the implementaion of `Process`.
struct JoinPoint<V1, V2, C>
where
  C: Continuation<(V1, V2)>
{
  P1_result: Rc<Cell<Option<V1>>>,
  P2_result: Rc<Cell<Option<V2>>>,
  next     : Rc<Cell<Option<C>>>
}

impl<V1, V2, C> JoinPoint<V1, V2, C>
where
  C: Continuation<(V1, V2)> + 'static
{
  /// Create a new `JoinPoint` with the given `next` continuation.
  fn new(next: C) -> JoinPoint<V1, V2, C> {
    JoinPoint {
      P1_result: Rc::new(Cell::new(None)),
      P2_result: Rc::new(Cell::new(None)),
      next:      Rc::new(Cell::new(Some(next)))
    }
  }
}


/// A process calling two sub-processes in a *synchronized* way,
/// i.e. waiting for both to finnish running before running the given `next` continuation.
pub struct JoinProcess<P1, P2>
where
  P1: Process + 'static,
  P2: Process + 'static
{
  process_1: P1,
  process_2: P2
}


impl<P1, P2> Process for JoinProcess<P1, P2>
where
  P1: Process + 'static,
  P2: Process + 'static
{
  type Value = (P1::Value, P2::Value);

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    let join_point_1 = Rc::new(JoinPoint::new(next));
    let join_point_2 = join_point_1.clone();

    self.process_1.call(runtime, move |runtime: &mut Runtime, P1_result: P1::Value| {
      println!("Running process 1 in JoinProcess");
      let P2_result = join_point_1.P2_result.take();

      if P2_result.is_some() {
        let next_input = (P1_result, P2_result.unwrap());
        let next = join_point_1.next.take().unwrap();
        next.call(runtime, next_input);
      }
      else {
        join_point_1.P1_result.set(Some(P1_result));
      }
    });

    self.process_2.call(runtime, move |runtime: &mut Runtime, P2_result: P2::Value| {
      println!("Running process 2 in JoinProcess");
      let P1_result = join_point_2.P1_result.take();

      if P1_result.is_some() {
        let next_input = (P1_result.unwrap(), P2_result);
        let next = join_point_2.next.take().unwrap();
        next.call(runtime, next_input);
      }
      else {
        join_point_2.P2_result.set(Some(P2_result));
      }
    });
  }
}


// Mutable process version below

/// A helper structure, used by `JoinProcess` to synchronize the call of two processes.
///
/// This version is specific to the implementaion of `ProcessMut`.
struct JoinPointMut<C, P1, P2, V1, V2>
where
  C: Continuation<(JoinProcess<P1, P2>, (V1, V2))>,
  P1: ProcessMut<Value = V1>,
  P2: ProcessMut<Value = V2>
{
  P1_result: Rc<Cell<Option<V1>>>,
  P2_result: Rc<Cell<Option<V2>>>,
  next     : Rc<Cell<Option<C>>>,
  p1       : Rc<Cell<Option<P1>>>,
  p2       : Rc<Cell<Option<P2>>>
}


impl<C, P1, P2, V1, V2> JoinPointMut<C, P1, P2, V1, V2>
where
  C: Continuation<(JoinProcess<P1, P2>, (V1, V2))> + 'static,
  P1: ProcessMut<Value = V1>,
  P2: ProcessMut<Value = V2>
{
  fn new(p1: P1, p2: P2, next: C) -> JoinPointMut<C, P1, P2, V1, V2> {
    JoinPointMut {
      P1_result: Rc::new(Cell::new(None)),
      P2_result: Rc::new(Cell::new(None)),
      next:      Rc::new(Cell::new(Some(next))),
      p1:   Rc::new(Cell::new(Some(p1))),
      p2:   Rc::new(Cell::new(Some(p2)))
    }
  }
}


impl<P1, P2> ProcessMut for JoinProcess<P1, P2>
where
  P1: ProcessMut + 'static,
  P2: ProcessMut + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    let join_point_1 = Rc::new(JoinPointMut::new(self.process_1, self.process_2, next));
    let join_point_2 = join_point_1.clone();
    let join_point_3 = join_point_1.clone();

    join_point_3.p1.take().unwrap().call_mut(runtime, move |runtime: &mut Runtime, (p1, v1): (P1, P1::Value)| {
      join_point_1.p1.set(Some(p1));
      let P2_result = join_point_1.P2_result.take();

      if P2_result.is_some() {
        let p1 = join_point_1.p1.take().unwrap();
        let p2 = join_point_1.p2.take().unwrap();

        let next_input = (v1, P2_result.unwrap());
        let next = join_point_1.next.take().unwrap();

        next.call(runtime, (p1.join(p2), next_input));
      }
      else {
        join_point_1.P1_result.set(Some(v1));
      }
    });

    join_point_3.p2.take().unwrap().call_mut(runtime, move |runtime: &mut Runtime, (p2, v2): (P2, P2::Value)| {
      join_point_2.p2.set(Some(p2));
      let P1_result = join_point_2.P1_result.take();

      if P1_result.is_some() {
        let p1 = join_point_2.p1.take().unwrap();
        let p2 = join_point_2.p2.take().unwrap();

        let next_input = (P1_result.unwrap(), v2);
        let next = join_point_2.next.take().unwrap();
        next.call(runtime, (p1.join(p2), next_input));
      }
      else {
        join_point_2.P2_result.set(Some(v2));
      }
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// WHILE PROCESS
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Indicates if a loop is finished.
#[derive(Debug)]
pub enum LoopStatus<V> { Continue, Exit(V) }

/// A process calling itself in a *while* loop fashion, until it returns `LoopStatus::Exit<V>`,
/// which contains the value to give to the `next` continuation.
pub struct WhileProcess<P>
where
  P: Process + 'static
{
  process: P
}


impl<P, V> Process for WhileProcess<P>
where
  P: ProcessMut<Value = LoopStatus<V>>
{
  type Value = V;

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self::Value)> {
    self.process.call_mut(runtime, |r: &mut Runtime, (p, v): (P, LoopStatus<V>)| {
      match v {
        LoopStatus::Continue     => p.while_loop().call(r, next),
        LoopStatus::Exit(output) => next.call(r, output)
      };
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use std::rc::Rc;
  use std::cell::{Cell, RefCell};

  use super::*;


  #[test]
  fn wait_two_instants () {
    let mut runtime = Runtime::new();

    let flag_ref      = Rc::new(RefCell::new(0));
    let flag_ref_copy = flag_ref.clone();

    runtime.on_current_instant(Box::new((move |_r: &mut Runtime, ()| {
      *flag_ref_copy.borrow_mut() = 42;
    }).pause().pause()));

    let mut work_remains = runtime.instant();
    assert_eq!(*flag_ref.borrow_mut(), 0);
    assert_eq!(work_remains, true);

    work_remains = runtime.instant();
    assert_eq!(*flag_ref.borrow_mut(), 0);
    assert_eq!(work_remains, true);

    work_remains = runtime.instant();
    assert_eq!(*flag_ref.borrow_mut(), 42);
    assert_eq!(work_remains, false);

    // There should not be any task left
    work_remains = runtime.instant();
    assert_eq!(work_remains, false);
  }


  #[test]
  fn map_to_multiply () {
    let process = value(21).map(|v| { 2*v });
    let return_value = execute_process(process);

    assert_eq!(42, return_value);
  }


  #[test]
  fn map_and_pause_to_multiply () {
    let process = value(21).pause().map(|v| { 2*v }).pause();

    let return_value = execute_process(process);
    assert_eq!(42, return_value);
  }


  #[test]
  fn join_sum_with_delay () {
    let immediate_process = value(10);
    let paused_process    = value(32).pause().pause().pause();

    let join_and_pause_process = immediate_process.join(paused_process)
      .map(|(v1, v2)| { v1 + v2 });

    let return_value = execute_process(join_and_pause_process);
    assert_eq!(42, return_value);
  }

  #[test]
  fn count_using_while () {
    let counter_1 = Rc::new(RefCell::new(0));
    let counter_2 = counter_1.clone();
    let counter_3 = counter_1.clone();

    let sum = move |v| { *counter_1.borrow_mut() += 1 };
    let test_loop_end = move |v| {
      match *counter_2.borrow() {
        42 => LoopStatus::Exit(42),
        _  => LoopStatus::Continue
      }
    };

    let sum = value(()).map(sum).map(test_loop_end).while_loop();

    execute_process(sum);
    assert_eq!(42, *counter_3.borrow());
  }
}
