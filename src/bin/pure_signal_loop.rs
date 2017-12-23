extern crate reactrust;

use reactrust::processes::*;
use reactrust::signals::signals::Signal;
use reactrust::signals::pure_signal::PureSignal;


/// Example program, which has the same behaviour of the the ReactiveML code snippet
/// given in question 2, part 3 of the project instructions.
///
/// It infinitely loops over three joined processes, which respectively:
/// * emit a signal S, and pause;
/// * test whether the signal S is present or absent, and pause;
/// * await (immediately) for the signal S, and pause.
fn main () {
  let pure_signal_1 = PureSignal::new();
  let pure_signal_2 = pure_signal_1.clone();
  let pure_signal_3 = pure_signal_1.clone();

  let continue_status_1 = |_| -> LoopStatus<()> { LoopStatus::Continue };
  let continue_status_2 = |_| -> LoopStatus<()> { LoopStatus::Continue };
  let continue_status_3 = |_| -> LoopStatus<()> { LoopStatus::Continue };

  // First process: emit and pause in a loop
  let emit_and_pause_process = pure_signal_1.emit()
    .pause()
    .map(continue_status_1)
    .while_loop();

  // Second process: print whether the signal is present or not
  let print_present = |_| { println!("Present"); };
  let print_absent  = |_| { println!("Absent"); };

  let present_or_absent_process = pure_signal_2.present(
    value(()).map(print_present).pause(),
    value(()).map(print_absent)
  )
  .map(continue_status_2)
  .while_loop();

  // Third process: await (immediately) the signal to print a message
  let print_signal_received = |_| { println!("Signal received"); };

  let await_process = pure_signal_3.await_immediate()
  .map(print_signal_received)
  .pause()
  .map(continue_status_3)
  .while_loop();

  // Final process: join all above processes
  let main_process = emit_and_pause_process.join(
    present_or_absent_process.join(
      await_process
    )
  );

  execute_process(main_process);
}
