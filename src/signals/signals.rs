/// A reactive signal.
pub trait Signal {
    /// Returns a reference to the signal's runtime.
    fn runtime(self) -> SignalRuntimeRef;

    /// Returns a process that waits for the next emission of the signal, current instant
    /// included.
    fn await_immediate(self) -> AwaitImmediate where Self: Sized {
      unimplemented!() // TODO
    }

    // TODO: add other methods if needed.
}

struct AwaitImmediate {
    // TODO
}

impl Process for AwaitImmediate {
    // TODO
}

impl ProcessMut for AwaitImmediate {
    // TODO
}
