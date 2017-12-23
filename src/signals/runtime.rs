use std::rc::Rc;
use std::cell::{Cell, RefCell};

use runtime::Runtime;
use continuations::Continuation;


///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL RUNTIME
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Runtime of a signal.
///
/// It is unique to each signal, and contains all the information concerning the signal:
/// its emit state, registered continuations, and value-related fields.
struct SignalRuntime<V, E> {
  // Emit state
  is_currently_emitted  : Cell<bool>,

  // Registered continuations
  call_on_present: RefCell<Vec<Box<Continuation<()>>>>,
  call_later_on_present: RefCell<Vec<Box<Continuation<V>>>>,
  call_later_on_absent: RefCell<Vec<Box<Continuation<()>>>>,

  // FLag indicating whether a continuation to run later_on_absent continuations
  // has been added to the runtime
  call_later_on_absent_registered: Cell<bool>,

  // Default, current, previous value and their gather function
  default_value: V,
  current_value: Cell<Option<V>>,
  previous_value: Cell<Option<V>>,
  gather_value_function: Cell<Option<Box<FnMut(E, &mut V)>>>
}


impl<V, E> SignalRuntime<V, E>
where
  E: Clone,
  V: Clone
{
  /// Create a new `SignalRuntime`, with a default value of type `V`, and a gather function
  /// receiving an element `E` and a mutable reference to the current value (of type `V`).
  pub fn new(default_value: V, gather_value_function: Box<FnMut(E, &mut V)>) -> Self {
    SignalRuntime {
      is_currently_emitted  : Cell::new(false),

      call_on_present: RefCell::new(Vec::new()),
      call_later_on_present: RefCell::new(Vec::new()),
      call_later_on_absent: RefCell::new(Vec::new()),

      call_later_on_absent_registered: Cell::new(false),

      default_value: default_value.clone(),
      current_value: Cell::new(Some(default_value.clone())),
      previous_value: Cell::new(None),
      gather_value_function: Cell::new(Some(gather_value_function))
    }
  }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// SIGNAL RUNTIME REFERENCE
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Shared pointer to a signal runtime.
///
/// It is meant to be shared and manipulated by all references to the same signal.
#[derive(Clone)]
pub struct SignalRuntimeRef<V, E> {
  runtime: Rc<SignalRuntime<V, E>>
}


impl<V, E> SignalRuntimeRef<V, E>
where
  V: Clone + 'static,
  E: Clone + 'static
{
  /// Create a new `SignalRuntimeRef`, and its inner `SignalRuntime`,
  /// using the given default value and gather function.
  /// See `new` method of `SignalRuntime` for more details.
  pub fn new(default_value: V, gather_value_function: Box<FnMut(E, &mut V)>) -> Self {
    SignalRuntimeRef { runtime: Rc::new(SignalRuntime::new(default_value, gather_value_function)) }
  }

  /// Modify the current value of the signal runtime,
  /// by applying the gather function to the given value.
  fn gather_value(&self, value: E) {
    let mut current_value         = self.runtime.current_value.take().unwrap();
    let mut gather_value_function = self.runtime.gather_value_function.take().unwrap();

    gather_value_function(value, &mut current_value);

    self.runtime.gather_value_function.set(Some(gather_value_function));
    self.runtime.current_value.set(Some(current_value));
  }

  /// Add a continuation to the end of current instant for updating the signal.
  /// It resets various fields and update the precedent and current value of this signal.
  fn add_update_on_end_of_instant(&self, runtime: &mut Runtime) {
    let signal_runtime_ref = self.clone();
    let signal_runtime     = self.runtime.clone();

    runtime.on_end_of_instant(Box::new(move |r: &mut Runtime, v: ()| {
      let signal_has_been_emitted = signal_runtime_ref.runtime.is_currently_emitted.get();
      signal_runtime_ref.runtime.is_currently_emitted.set(false);

      // Those continuations now are useless
      signal_runtime_ref.runtime.call_on_present.borrow_mut().clear();
      signal_runtime_ref.runtime.call_later_on_present.borrow_mut().clear();

      signal_runtime.previous_value.replace(signal_runtime.current_value.take());
      signal_runtime.current_value.set(Some(signal_runtime.default_value.clone()));
    }));
  }

  /// Add all continuations stored in the `on_present_continuations` field of the signal runtime
  /// to current instant.
  fn add_on_present_continuations_to_runtime(&self, runtime: &mut Runtime) {
    let mut on_present_continuations = self.runtime.call_on_present.borrow_mut();
    for boxed_continuation in on_present_continuations.drain(..) {
      runtime.on_current_instant(boxed_continuation);
    }
  }

  /// Add all continuations stored in the `later_on_present_continuations`
  /// field of the signal runtime to next instant.
  ///
  /// They are enclosed in continuations which feed them the *precedent value* of the signal,
  /// (*precedent* at the moment of the call, i.e. during next instant).
  fn add_later_on_present_continuations_to_runtime(&self, runtime: &mut Runtime) {
    let mut later_on_present_continuations = self.runtime.call_later_on_present.borrow_mut();
    for boxed_continuation in later_on_present_continuations.drain(..) {
      let signal_runtime_ref = self.clone();

      runtime.on_next_instant(Box::new(move |r: &mut Runtime, v: ()| {
        let previous_value = signal_runtime_ref.runtime.previous_value.take().unwrap();
        signal_runtime_ref.runtime.previous_value.set(Some(previous_value.clone()));

        boxed_continuation.call_box(r, previous_value.clone());
      }));
    }
  }

  /// Add all continuations stored in the `later_on_absent_continuations` field of the signal runtime
  /// to next instant.
  fn add_later_on_absent_continuations_to_runtime(&self, runtime: &mut Runtime) {
    let mut later_on_absent_continuations = self.runtime.call_later_on_absent.borrow_mut();
    for boxed_continuation in later_on_absent_continuations.drain(..) {
      runtime.on_next_instant(boxed_continuation);
    }
  }


  /// Emit the signal during current instant.
  ///
  /// It updates the state of the signal runtime, gather the given value,
  /// drop any pending continuations to run if the signal was absent,
  /// and add all pending continuations to run if the signal is present to the runtime.
  pub fn emit(self, mut runtime: &mut Runtime, value: E) {
    if self.runtime.is_currently_emitted.get() {
      return;
    }

    self.runtime.is_currently_emitted.set(true);
    self.add_update_on_end_of_instant(runtime);

    self.gather_value(value);

    // Empty the list of continuations to execute during next instant if there is *no* signal
    self.runtime.call_later_on_absent.borrow_mut().clear();
    self.runtime.call_later_on_absent_registered.set(false);

    // Add awaiting continuations to current instant
    self.add_on_present_continuations_to_runtime(runtime);
    self.add_later_on_present_continuations_to_runtime(runtime);
  }

  /// Register a continuation to run during current instant
  /// if the signal is present during current instant.
  pub fn on_present<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    if self.runtime.is_currently_emitted.get() {
      runtime.on_current_instant(Box::new(c));
    }
    else {
      self.runtime.call_on_present.borrow_mut().push(Box::new(c));
    }
  }

  /// Register a continuation to run during next instant
  /// if the signal is present during current instant.
  ///
  /// If executed, the continuation will be given the previous value of the signal.
  pub fn later_on_present<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<V> {
    if self.runtime.is_currently_emitted.get() {
      runtime.on_next_instant(Box::new(move |r: &mut Runtime, v: ()| {
        let previous_value = self.runtime.previous_value.take().unwrap();
        self.runtime.previous_value.set(Some(previous_value.clone()));

        c.call(r, previous_value.clone());
      }));
    }
    else {
      self.runtime.call_later_on_present.borrow_mut().push(Box::new(c));
    }
  }

  /// Register a continuation to run during current instant
  /// if the signal is absent during current instant.
  pub fn later_on_absent<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
    if self.runtime.is_currently_emitted.get() {
      return;
    }
    else {
      self.runtime.call_later_on_absent.borrow_mut().push(Box::new(c));

      if ! self.runtime.call_later_on_absent_registered.get() {
        let signal_runtime_ref = self.clone();
        runtime.on_next_instant(Box::new(move |r: &mut Runtime, v: ()| {
          signal_runtime_ref.add_later_on_absent_continuations_to_runtime(r);
        }));

        self.runtime.call_later_on_absent_registered.set(true);
      }
    }
  }

}
