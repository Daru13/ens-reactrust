use signals::signals::Signal;
use signals::runtime::SignalRuntimeRef;


///////////////////////////////////////////////////////////////////////////////////////////////////
// PURE SIGNAL
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct PureSignal {
  runtime_ref: SignalRuntimeRef
}


impl PureSignal {
  pub fn new() -> Self {
    PureSignal { runtime_ref: SignalRuntimeRef::new() }
  }
}


impl Signal for PureSignal {
  fn runtime(self) -> SignalRuntimeRef {
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

  use runtime::Runtime;
  use processes::*;
  use super::*;

  // This function joins and emitting process and an awaiting process, in both possible orders.
  // This allows tests to try both orders without duplicating most of the code.
  fn general_emit_and_await_immediate(await_first: bool) {
    let pure_signal_1 = PureSignal::new();
    let pure_signal_2 = pure_signal_1.clone();

    let pure_signal_emitted_1  = Rc::new(Cell::new(false));
    let pure_signal_emitted_2  = pure_signal_emitted_1.clone();
    let pure_signal_emitted_3  = pure_signal_emitted_1.clone();
    let pure_signal_received_1 = Rc::new(Cell::new(false));
    let pure_signal_received_2 = pure_signal_received_1.clone();
    let pure_signal_received_3 = pure_signal_received_1.clone();

    // First process: check that the signal has *not* been received before, and emit the signal
    let process_emit = pure_signal_1.emit().map(move |signal| {
      // println!("Signal has been emitted from process_emit");
      assert_eq!(pure_signal_received_1.get(), false);
      pure_signal_emitted_1.set(true);
    });

    // Second process: check that the signal has been emitted before, once received
    let process_await = pure_signal_2.await_immediate().map(move |v| {
      // println!("Signal has been received in process_await");
      assert_eq!(pure_signal_emitted_2.get(), true);
      pure_signal_received_2.set(true);
    });

    // Third (main) process: run both processes and makes sure the signal is emitted and received
    // The order of the join operation is decided at this point, according to the given parameter
    let check_joined_processes = move |v| {
      assert_eq!(pure_signal_emitted_3.get(), true);
      assert_eq!(pure_signal_received_3.get(), true);
    };

    if await_first {
      execute_process(process_await.join(process_emit).map(check_joined_processes));
    }
    else {
      execute_process(process_emit.join(process_await).map(check_joined_processes));
    }
  }

  #[test]
  fn emit_and_await_immediate () {
    general_emit_and_await_immediate(false);
  }

  #[test]
  fn await_immediate_and_emit () {
    general_emit_and_await_immediate(true);
  }


  fn general_present_or_absent_signal (emit_signal: bool) {
    let pure_signal_1 = PureSignal::new();
    let pure_signal_2 = pure_signal_1.clone();

    let present_signal_1  = Rc::new(Cell::new(false));
    let present_signal_2  = present_signal_1.clone();
    let present_signal_3  = present_signal_1.clone();

    // First process: ran if the signal is present (during current instant)
    let process_if = value(()).map(move |v| {
      // println!("Signal is present");
      assert_eq!(present_signal_1.get(), true);
    });

    // Second process: ran if the signal was absent (during previous instant)
    let process_else = value(()).map(move |v| {
      // println!("Signal is absent");
      assert_eq!(present_signal_2.get(), false);
    });

    // Run a present process, and possibly emit the signal during the same instant (using join construct)
    // Whether the signal shall be emitted or not is decided at this point, according to the given parameter
    if emit_signal {
      let emit_process = pure_signal_1.emit().map(move |v| { present_signal_3.set(true); });
      execute_process(pure_signal_2.present(process_if, process_else).join(emit_process));
    }
    else {
      execute_process(pure_signal_1.present(process_if, process_else));
    }
  }

  #[test]
  fn signal_is_present () {
    general_present_or_absent_signal(true);
  }

  #[test]
  fn signal_is_absent () {
    general_present_or_absent_signal(false);
  }
}
