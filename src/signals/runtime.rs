use std::rc::Rc;
use std::cell::{Cell, RefCell};

use runtime::Runtime;
use continuations::Continuation;


///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL RUNTIME
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Runtime for pure signals.
struct SignalRuntime {
  is_currently_emitted    : Cell<bool>,
  registered_continuations: RefCell<Vec<Box<Continuation<()>>>>
}


impl SignalRuntime {
  pub fn new() -> Self {
    SignalRuntime {
      is_currently_emitted    : Cell::new(false),
      registered_continuations: RefCell::new(Vec::new())
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
      signal_runtime.registered_continuations.borrow_mut().clear();
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

    let mut registered_continuations  = self.runtime.registered_continuations.borrow_mut();
    let registered_continuations_iter = registered_continuations.drain(..);

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
      self.runtime.registered_continuations.borrow_mut().push(Box::new(c));
    }
  }

  pub fn on_signal_mut<C>(&mut self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    println!("On signal mut");
    self.runtime.registered_continuations.borrow_mut().push(Box::new(c));
  }

  // TODO: add other methods when needed.
}
