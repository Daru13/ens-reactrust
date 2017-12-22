use std::rc::Rc;
use std::cell::*;
use std::marker::PhantomData;

use runtime::Runtime;
use continuations::Continuation;
use processes::{Process, ProcessMut};
use signals::runtime::SignalRuntimeRef;


///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A reactive signal.
pub trait Signal<V, E>
where
  Self: Clone,
  V: Clone,
  E: Clone
{
  /// Returns a reference to the signal's runtime.
  fn runtime(self) -> SignalRuntimeRef<V, E>;

  fn emit(self) -> Emit<Self, V, E>
  where
    Self: Sized + 'static
  {
    Emit { signal: Box::new(self), phantom: PhantomData }
  }

  fn await(self) -> Await<Self, V, E>
  where
    Self: Sized + 'static
  {
    Await { signal: Box::new(self), phantom: PhantomData }
  }

  /// Returns a process that waits for the next emission of the signal, current instant
  /// included.
  fn await_immediate(self) -> AwaitImmediate<Self, V, E>
  where
    Self: Sized + 'static
  {
    AwaitImmediate { signal: Box::new(self), phantom: PhantomData }
  }

  fn present<P1, P2, PV>(self, process_if: P1, process_else: P2) -> Present<Self, P1, P2, PV, V, E>
  where
    Self: Sized + 'static,
    P1: Process<Value = PV>,
    P2: Process<Value = PV>,
    V: 'static
  {
    Present {
      signal      : Box::new(self),
      process_if  : process_if,
      process_else: process_else,
      phantom: PhantomData
    }
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// AWAIT
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Await<S, V, E>
where
  S: Signal<V, E> + Sized + Clone,
  V: Clone + 'static,
  E: Clone + 'static
{
  signal: Box<S>,
  phantom: PhantomData<(V, E)>
}


impl<S, V, E> Process for Await<S, V, E>
where
  S: Signal<V, E> + Sized + 'static,
  V: Clone + 'static,
  E: Clone + 'static
{
  type Value = ();

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    self.signal.runtime().later_on_present(runtime, next);
  }
}


impl<S, V, E> ProcessMut for Await<S, V, E>
where
  S: Signal<V, E> + Sized + Clone + 'static,
  V: Clone + 'static,
  E: Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    let s1 = *self.signal;
    let s2 = s1.clone();

    s1.runtime().later_on_present(runtime, move |r: &mut Runtime, v: ()| {
      next.call(r, (s2.await(), ()));
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// AWAIT IMMEDIATE
///////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Clone)]
pub struct AwaitImmediate<S, V, E>
where
  S: Signal<V, E> + Sized + Clone,
  V: Clone + 'static,
  E: Clone + 'static
{
  signal: Box<S>,
  phantom: PhantomData<(V, E)>
}


impl<S, V, E> Process for AwaitImmediate<S, V, E>
where
  S: Signal<V, E> + Sized + 'static,
  V: Clone + 'static,
  E: Clone + 'static
{
  type Value = ();

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    self.signal.runtime().on_present(runtime, next);
  }
}


impl<S, V, E> ProcessMut for AwaitImmediate<S, V, E>
where
  S: Signal<V, E> + Sized + Clone + 'static,
  V: Clone + 'static,
  E: Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    let s1 = *self.signal;
    let s2 = s1.clone();

    s1.runtime().on_present(runtime, move |r: &mut Runtime, v: ()| {
      next.call(r, (s2.await_immediate(), ()));
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// EMIT
///////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Clone)]
pub struct Emit<S, V, E>
where
  S: Signal<V, E> + Sized + Clone,
  V: Clone + 'static,
  E: Clone + 'static
{
  signal: Box<S>,
  phantom: PhantomData<(V, E)>
}


impl<S, V, E> Process for Emit<S, V, E>
where
  S: Signal<V, E> + Sized + 'static,
  V: Clone + 'static,
  E: Clone + 'static
{
  type Value = ();

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    //println!("Call in Emit");

    self.signal.runtime().emit(runtime);
    next.call(runtime, ());
  }
}


impl<S, V, E> ProcessMut for Emit<S, V, E>
where
  S: Signal<V, E> + Sized + Clone + 'static,
  V: Clone + 'static,
  E: Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    //println!("Call mut in Emit");

    let signal_1 = self.signal;
    let signal_2 = signal_1.clone();

    signal_1.runtime().emit(runtime);
    next.call(runtime, (signal_2.emit(), ()));
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// PRESENT
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Present<S, P1, P2, PV, SV, E>
where
  S: Signal<SV, E> + Sized + Clone,
  P1: Process<Value = PV>,
  P2: Process<Value = PV>,
  PV: 'static,
  SV:Clone +  'static,
  E: Clone + 'static
{
  signal      : Box<S>,
  process_if  : P1,
  process_else: P2,
  phantom     : PhantomData<(SV, E)>
}


impl<S, P1, P2, PV, SV, E> Process for Present<S, P1, P2, PV, SV, E>
where
  S: Signal<SV, E> + Sized + 'static,
  P1: Process<Value = PV>,
  P2: Process<Value = PV>,
  PV: 'static,
  SV:Clone +  'static,
  E: Clone + 'static
{
  type Value = PV;

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    //println!("Call in Present");

    let signal_1   = self.signal;
    let signal_2   = signal_1.clone();

    let next_1 = Rc::new(Cell::new(Some(next)));
    let next_2 = next_1.clone();

    // Case 1: the signal is present during current instant
    let process_if = self.process_if;

    signal_1.runtime().on_present(runtime, move |r: &mut Runtime, v: ()| {
      process_if.call(r, next_1.take().unwrap());
    });

    // Case 2: the signal is absent during current instant
    let process_else = self.process_else;

    signal_2.runtime().later_on_absent(runtime, move |r: &mut Runtime, v: ()| {
      process_else.call(r, next_2.take().unwrap());
    });
  }
}


impl<S, P1, P2, PV, SV, E> ProcessMut for Present<S, P1, P2, PV, SV, E>
where
  S: Signal<SV, E> + Sized + Clone + 'static,
  P1: ProcessMut<Value = PV>,
  P2: ProcessMut<Value = PV>,
  PV: 'static,
  SV:Clone +  'static,
  E: Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    //println!("Call mut in Present");

    let signal_1 = self.signal;
    let signal_2 = signal_1.clone();
    let signal_3 = signal_1.clone();

    let signal_4 = Rc::new(Cell::new(Some(signal_3)));
    let signal_5 = signal_4.clone();

    let next_1 = Rc::new(Cell::new(Some(next)));
    let next_2 = next_1.clone();

    let process_if_1 = Rc::new(Cell::new(Some(self.process_if)));
    let process_if_2 = process_if_1.clone();

    let process_else_1 = Rc::new(Cell::new(Some(self.process_else)));
    let process_else_2 = process_else_1.clone();

    // Case 1: the signal is present during current instant
    signal_1.runtime().on_present(runtime, move |r: &mut Runtime, v: ()| {
      process_if_1.take().unwrap().call_mut(r, move |r: &mut Runtime, (p, v): (P1, PV)| {
        let present = signal_4.take().unwrap().present(p, process_else_1.take().unwrap());
        next_1.take().unwrap().call(r, (present, v));
      });
    });

    // Case 2: the signal is absent during current instant
    signal_2.runtime().later_on_absent(runtime, move |r: &mut Runtime, v: ()| {
      process_else_2.take().unwrap().call_mut(r, move |r: &mut Runtime, (p, v): (P2, PV)| {
        let present = signal_5.take().unwrap().present(process_if_2.take().unwrap(), p);
        next_2.take().unwrap().call(r, (present, v));
      });
    });
  }
}
