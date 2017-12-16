use std::rc::Rc;

/// A shared pointer to a signal runtime.
#[derive(Clone)]
pub struct SignalRuntimeRef {
    runtime: Rc<SignalRuntime>,
}

/// Runtime for pure signals.
struct SignalRuntime {
    // TODO: implement
}

impl SignalRuntimeRef {
    /// Sets the signal as emitted for the current instant.
    fn emit(self, runtime: &mut Runtime) {
        unimplemented!() // TODO
    }

    /// Calls `c` at the first cycle where the signal is present.
    fn on_signal<C>(self, runtime: &mut Runtime, c: C) where C: Continuation<()> {
        unimplemented!() // TODO
    }

    // TODO: add other methods when needed.
}
