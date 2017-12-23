use runtime::Runtime;


///////////////////////////////////////////////////////////////////////////////////////////////////
// CONTINUATION
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A reactive continuation awaiting a value of type `V`.
/// For the sake of simplicity, continuations must be valid on the `static` lifetime.
pub trait Continuation<V>: 'static {
  /// Calls the continuation.
  fn call(self, runtime: &mut Runtime, value: V);

  /// Calls the continuation. Works even if the continuation is boxed.
  ///
  /// This is necessary because the size of a value must be known to unbox it. It is
  /// thus impossible to take the ownership of a `Box<Continuation>` whitout knowing the
  /// underlying type of the `Continuation`.
  fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V);

  /// Creates a new continuation that applies a function to the input value before calling `Self`.
  fn map<F, V2>(self, map: F) -> Map<Self, F>
  where
    Self: Sized,
    F: FnOnce(V2) -> V + 'static
  {
    Map {
      continuation: self,
      map: map
    }
  }

  /// Creates a new continuation that waits for the next instant before running a continuation
  fn pause(self) -> Pause<Self>
  where
    Self: Sized
  {
    Pause {
      continuation: self
    }
  }
}


/// Functions of type `FnOnce` are considered to be continuations.
///
/// This is used in order to make continuations out of Rust closures.
impl<V, F> Continuation<V> for F
where
  F: FnOnce(&mut Runtime, V) + 'static
{
  fn call(self, runtime: &mut Runtime, value: V) {
    self(runtime, value);
  }

  fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V) {
    (*self).call(runtime, value);
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// MAP
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A continuation that applies a function to the value it receives,
/// before calling another continuation.
pub struct Map<C, F> {
  continuation: C,
  map: F
}

impl<C, F, V1, V2> Continuation<V1> for Map<C, F>
where
  C: Continuation<V2>,
  F: FnOnce(V1) -> V2 + 'static
{
  fn call(self, runtime: &mut Runtime, value: V1) {
    let result = (self.map)(value);
    self.continuation.call(runtime, result);
  }

  fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V1) {
    (*self).call(runtime, value);
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// PAUSE
///////////////////////////////////////////////////////////////////////////////////////////////////

/// A continuation that postpones its execution to the next instant.
pub struct Pause<C> {
  continuation: C
}

impl<C, V> Continuation<V> for Pause<C>
where
  C: Continuation<V>, V: 'static
{
  fn call(self, runtime: &mut Runtime, value: V) {
    runtime.on_next_instant(Box::new(move |r: &mut Runtime, ()| {
      self.continuation.call(r, value);
    }));
  }

  fn call_box(self: Box<Self>, runtime: &mut Runtime, value: V) {
    (*self).call(runtime, value);
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use std::rc::Rc;
  use std::cell::RefCell;

  use super::*;


  #[test]
  fn wait_two_instants () {
    let mut runtime = Runtime::new();

    let flag_ref      = Rc::new(RefCell::new(0));
    let flag_ref_copy = flag_ref.clone();

    runtime.on_current_instant(Box::new(move |r: &mut Runtime, ()| {
      println!("Waiting instant 1...");
      r.on_next_instant(Box::new(move |r: &mut Runtime, ()| {
        println!("Waiting instant 2...");
        r.on_next_instant(Box::new(move |_r: &mut Runtime, ()| {
          *flag_ref_copy.borrow_mut() = 42;
        }));
      }));
    }));

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
}
