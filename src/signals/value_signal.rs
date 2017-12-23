use std::marker::PhantomData;

use signals::signals::*;
use signals::runtime::SignalRuntimeRef;


///////////////////////////////////////////////////////////////////////////////////////////////////
// VALUE SIGNAL
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Signal carrying a value.
#[derive(Clone)]
pub struct ValueSignal<V, E> {
  runtime_ref: SignalRuntimeRef<V, E>
}


impl<V, E> ValueSignal<V, E>
where
  V: Clone + 'static,
  E: Clone + 'static
{
  /// Create a new `ValueSignal`, and its inner `SignalRuntimeRef`,
  /// using the given default value and gather function.
  /// See `new` method of `SignalRuntimeRef` for more details.
  pub fn new_with_gather_function(default_value: V, gather_value_function: Box<FnMut(E, &mut V)>) -> Self {
    ValueSignal { runtime_ref: SignalRuntimeRef::new(default_value, gather_value_function) }
  }
}


impl<E> ValueSignal<Vec<E>, E>
where
  E: Clone + 'static
{
  /// Create a new `ValueSignal` and its inner `SignalRuntimeRef`,
  /// using an empty vector as default value,
  /// and a gather function which pushes the given value into the vector.
  pub fn new() -> Self
  {
    ValueSignal { runtime_ref: SignalRuntimeRef::new(Vec::new(), Box::new(|e, v| { v.push(e); })) }
  }
}


impl<V, E> Signal<V, E> for ValueSignal<V, E>
where
  V: Clone,
  E: Clone
{
  fn runtime(self) -> SignalRuntimeRef<V, E> {
    self.runtime_ref.clone()
  }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use std::rc::Rc;
  use std::cell::{Cell, RefCell};

  use processes::*;
  use super::*;


  #[test]
  fn count_using_signal_values()
  {
    let gather_function = |e: u32, v: &mut u32| { *v = e };
    let value_signal_1  = ValueSignal::new_with_gather_function(0, Box::new(gather_function));
    let value_signal_2  = value_signal_1.clone();

    let emit_counter_1 = Rc::new(Cell::new(0));
    let emit_counter_2 = emit_counter_1.clone();

    let signal_value_sum_1 = Rc::new(Cell::new(0));
    let signal_value_sum_2 = signal_value_sum_1.clone();

    let update_signal_value = move |precedent_signal_value| {
      let current_value = signal_value_sum_1.get();
      signal_value_sum_1.set(current_value + precedent_signal_value);
    };

    let emit_loop_process  = value_signal_1.emit_value(3).pause();
    let await_loop_process = value_signal_2.await().map(update_signal_value);

    let loop_map = move |_| {
      let iteration = emit_counter_1.get() + 1;
      emit_counter_1.set(iteration);

      match iteration {
        14 => LoopStatus::Exit(()),
        _  => LoopStatus::Continue
      }
    };
    let join_process = await_loop_process.join(emit_loop_process).map(loop_map).while_loop();

    execute_process(join_process);
    assert_eq!(signal_value_sum_2.get(), 42);
  }
}
