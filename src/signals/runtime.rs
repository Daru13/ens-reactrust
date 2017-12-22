use std::rc::Rc;
use std::cell::{Cell, RefCell};

use runtime::Runtime;
use continuations::Continuation;


///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL RUNTIME
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Runtime for pure signals.
struct SignalRuntime {
  is_currently_emitted  : Cell<bool>,
  call_on_present: RefCell<Vec<Box<Continuation<()>>>>,
  call_later_on_present: RefCell<Vec<Box<Continuation<()>>>>,
  call_later_on_absent: RefCell<Vec<Box<Continuation<()>>>>
}


impl SignalRuntime {
  pub fn new() -> Self {
    SignalRuntime {
      is_currently_emitted  : Cell::new(false),
      call_on_present: RefCell::new(Vec::new()),
      call_later_on_present: RefCell::new(Vec::new()),
      call_later_on_absent: RefCell::new(Vec::new())
    }
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL RUNTIME REFERENCE
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A shared pointer to a signal runtime.
#[derive(Clone)]
pub struct SignalRuntimeRef {
  runtime: Rc<SignalRuntime>,
}


impl SignalRuntimeRef {

  pub fn new() -> Self {
    SignalRuntimeRef { runtime: Rc::new(SignalRuntime::new()) }
  }


  fn reset_on_end_of_instant(&self, runtime: &mut Runtime) {
    let signal_runtime = self.runtime.clone();

    runtime.on_end_of_instant(Box::new(move |r: &mut Runtime, v: ()| {
      signal_runtime.is_currently_emitted.set(false);
      signal_runtime.call_on_present.borrow_mut().clear();
      signal_runtime.call_later_on_present.borrow_mut().clear();
      signal_runtime.call_later_on_absent.borrow_mut().clear();
    }));
  }


  fn run_continuations_on_next_instant(&self, runtime: &mut Runtime) {
    let signal_runtime = self.runtime.clone();

    runtime.on_next_instant(Box::new(move |r: &mut Runtime, v: ()| {
      let mut later_on_present_continuations = signal_runtime.call_later_on_present.borrow_mut();
      for boxed_continuation in later_on_present_continuations.drain(..) {
        r.on_current_instant(boxed_continuation);
      }

      let mut later_on_absent_continuations  = signal_runtime.call_later_on_absent.borrow_mut();
      for boxed_continuation in later_on_absent_continuations.drain(..) {
        r.on_current_instant(boxed_continuation);
      }
    }));
  }


  /// Sets the signal as emitted for the current instant.
  pub fn emit(self, mut runtime: &mut Runtime) {
    if self.runtime.is_currently_emitted.get() {
      return;
    }

    self.runtime.is_currently_emitted.set(true);
    self.reset_on_end_of_instant(runtime);

    // Empty the list of continuations to execute during next instant if there is *no* signal
    self.runtime.call_later_on_absent.borrow_mut().clear();

    // Add awaiting continuations to current instant
    let mut on_present_continuations = self.runtime.call_on_present.borrow_mut();
    for boxed_continuation in on_present_continuations.drain(..) {
      runtime.on_current_instant(boxed_continuation);
    }
  }


  pub fn on_present<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    if self.runtime.is_currently_emitted.get() {
      runtime.on_current_instant(Box::new(c));
    }
    else {
      self.runtime.call_on_present.borrow_mut().push(Box::new(c));
    }
  }


  pub fn later_on_present<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    if self.runtime.is_currently_emitted.get() {
      runtime.on_next_instant(Box::new(c));
    }
    else {
      self.runtime.call_later_on_present.borrow_mut().push(Box::new(c));
    }
  }


  pub fn later_on_absent<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    if self.runtime.is_currently_emitted.get() {
      return;
    }
    else {
      if self.runtime.call_later_on_absent.borrow_mut().is_empty() {
        self.run_continuations_on_next_instant(runtime);
      }

      self.runtime.call_later_on_absent.borrow_mut().push(Box::new(c));
    }
  }
}
