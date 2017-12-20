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
  awaiting_continuations: RefCell<Vec<Box<Continuation<()>>>>,
  on_present_continuations: RefCell<Vec<Box<Continuation<()>>>>,
  on_absent_continuations: RefCell<Vec<Box<Continuation<()>>>>,
}


impl SignalRuntime {
  pub fn new() -> Self {
    SignalRuntime {
      is_currently_emitted  : Cell::new(false),
      awaiting_continuations: RefCell::new(Vec::new()),
      on_present_continuations: RefCell::new(Vec::new()),
      on_absent_continuations: RefCell::new(Vec::new())
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


  /// Register signal runtime for a reset at the end of current instant.
  /// The signal runtime fields are reset to their initial state (the same as the one produced by `new`);
  fn reset_at_end_of_instant(&self, runtime: &mut Runtime) {
    let signal_runtime = self.runtime.clone();
    let reset_continuation = move |r: &mut Runtime, v: ()| {
      signal_runtime.is_currently_emitted.set(false);
      signal_runtime.awaiting_continuations.borrow_mut().clear();
      signal_runtime.on_absent_continuations.borrow_mut().clear();
    };

    runtime.on_end_of_instant(Box::new(reset_continuation));
  }


  /// Sets the signal as emitted for the current instant.
  pub fn emit(self, mut runtime: &mut Runtime) {
    println!("Emit signal");

    if self.runtime.is_currently_emitted.get() {
      println!("Signal has already been emitted, nothing to do");
      return;
    }

    self.runtime.is_currently_emitted.set(true);
    self.reset_at_end_of_instant(runtime);

    let mut awaiting_continuations  = self.runtime.awaiting_continuations.borrow_mut();
    let registered_continuations_iter = awaiting_continuations.drain(..);

    for boxed_continuation in registered_continuations_iter {
      runtime.on_current_instant(boxed_continuation);
      println!("Added continuation to current instant");
    }
  }


  /// Calls `c` at the first cycle where the signal is present.
  pub fn on_signal<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    println!("On signal");

    if self.runtime.is_currently_emitted.get() {
      println!("Signal is already emitted: adding c to current instant");
      runtime.on_current_instant(Box::new(c));
    }
    else {
      println!("Signal is not emitted: saving c for later");
      self.runtime.awaiting_continuations.borrow_mut().push(Box::new(c));
    }
  }

  // TODO: check if this version is actually needed or not...
  pub fn on_signal_mut<C>(&mut self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    println!("On signal mut");

    if self.runtime.is_currently_emitted.get() {
      println!("Signal is already emitted: adding c to current instant");
      runtime.on_current_instant(Box::new(c));
    }
    else {
      println!("Signal is not emitted: saving c for later");
      self.runtime.awaiting_continuations.borrow_mut().push(Box::new(c));
    }
  }


  /// Calls `c` during the next cycle if the signal is not present during current cycle.
  pub fn on_no_signal<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    println!("On no signal");

    if self.runtime.is_currently_emitted.get() {
      println!("Signal is already emitted: ignoring c");
      return;
    }
    else {
      println!("Signal is not emitted: saving c for next instant");

      let signal_runtime = Rc::new(Cell::new(Some(self.runtime.clone())));

      if self.runtime.on_absent_continuations.borrow_mut().is_empty() {
        println!("Adding a continuation to execute potential on absent signals during next instant");

        let run_next_instant_continuations = move |r: &mut Runtime, v: ()| {
          let     signal_runtime               = signal_runtime.take().unwrap();
          let mut on_absent_continuations      = signal_runtime.on_absent_continuations.borrow_mut();
          let     on_absent_continuations_iter = on_absent_continuations.drain(..);

          for boxed_continuation in on_absent_continuations_iter {
            r.on_current_instant(boxed_continuation);
            println!("Added (on absent) continuations during next instant");
          }
        };

        runtime.on_next_instant(Box::new(run_next_instant_continuations));
      }

      self.runtime.on_absent_continuations.borrow_mut().push(Box::new(c));
    }
  }
}
