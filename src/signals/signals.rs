use std::rc::Rc;
use std::cell::*;

use runtime::Runtime;
use continuations::Continuation;
use processes::{Process, ProcessMut};
use signals::runtime::SignalRuntimeRef;


///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A reactive signal.
pub trait Signal
where
  Self: Clone
{
  /// Returns a reference to the signal's runtime.
  fn runtime(self) -> SignalRuntimeRef;

  /// Returns a process that waits for the next emission of the signal, current instant
  /// included.
  fn await_immediate(self) -> AwaitImmediate<Self>
  where
    Self: Sized + 'static
  {
    AwaitImmediate { signal: Box::new(self) }
  }

  fn emit(self) -> Emit<Self>
  where
    Self: Sized + 'static
  {
    Emit { signal: Box::new(self) }
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// AWAIT IMMEDIATE
///////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Clone)]
pub struct AwaitImmediate<S>
where
  S: Signal + Sized + Clone
{
  signal: Box<S>
}


impl<S> Process for AwaitImmediate<S>
where
  S: Signal + Sized + 'static
{
  type Value = ();

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    self.signal.runtime().on_signal(runtime, next);
  }
}


impl<S> ProcessMut for AwaitImmediate<S>
where
  S: Signal + Sized + Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    let s1 = *self.signal;
    let s2 = s1.clone();

    s1.runtime().on_signal_mut(runtime, move |r: &mut Runtime, v: ()| {
      next.call(r, (s2.await_immediate(), ()));
    });
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// EMIT
///////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Clone)]
pub struct Emit<S>
where
  S: Signal + Sized + Clone
{
  signal: Box<S>
}


impl<S> Process for Emit<S>
where
  S: Signal + Sized + 'static
{
  type Value = ();

  fn call<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<Self::Value> {
    println!("Call in Emit");
    self.signal.runtime().emit(runtime);
    next.call(runtime, ());
  }
}


impl<S> ProcessMut for Emit<S>
where
  S: Signal + Sized + Clone + 'static
{
  fn call_mut<C>(self, runtime: &mut Runtime, next: C) where C: Continuation<(Self, Self::Value)> {
    println!("Call mut in Emit");

    let signal_1 = self.signal;
    let signal_2 = signal_1.clone();

    signal_1.runtime().emit(runtime);
    next.call(runtime, (signal_2.emit(), ()));
  }
}
