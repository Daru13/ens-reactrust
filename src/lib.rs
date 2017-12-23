//! `reactrust` is a simple reactive library written for a course project at ENS.
//!
//! It was inspired by the ReactiveML language, and provides related concepts, such as processes
//! and signals, and methods to interact with them to build reactive programs.
//!
//!
//! **Note:** even though this library works, I haven't had time to do everything in the project,
//! which could be greatly improved. It mainly lacks more process types,
//! specialized signals (e.g. single- and multiple- consumer(s) versions),
//! as well as more testing.
//!
//! There also is no proper program using this library, nor parallelization attempts.
//!


/// **A continuation is a part of program, of a computation, which await a value of some type.**
///
/// It allows to cut a large program into smaller parts,
/// which can be chained and ordered in logical time units.
///
/// Implementing `Continuation` requires two things:
///
/// * Implementing the `call` method,
///   for running a continuation in a given runtime, with a given value.
/// * Implementing the `call_box` method,
///   for running a *boxed* continuation in a given runtime, with a given value.
///
/// Continuations are the inner mechanism used by this reactive library, but they are not
/// meant to be directly manipulated by end users. Instead, they are used by higher-level concepts,
/// such a processes (see module `processes`).
///
/// They thus provide little methods and concepts for working with them,
/// as this is done by other parts of the library.
pub mod continuations;

/// **A runtime is an environment for running continuations.**
///
/// A runtime discretizes the time in so-called *instants*, which are logical units of time where
/// a certain number of continuations (*tasks*) are ran (using their `call` methods). It only accepts
/// continuations expecting a unit `()` typed input, but those can call other continuations in chain,
/// though it always happen in the same instant.
///
/// It contains three lists of continuations:
///
/// * `current_instant_tasks`, containing continuations to run during current instant.
/// * `end_of_instant_tasks`, containing special continuations to run at the end of current instant.
///   This list is mainly meant to be used for internal mechanisms, such as signals
///   (see module `signals` for details).
/// * `next_instant_tasks`, containing continuations to run during next instant.
///   When all continuations of the two other lists have been ran, the content of this list becomes
///   the new content of `current_instant_tasks`.
///
/// A runtime run continuations contained in those lists in this very order, until they are empty.
/// The `instant` method is designed to do this over one instant, while the `execute` method does it
/// until there is no more work to do.
///
pub mod runtime;

/// **Processes are abstractions over continuations,
/// which allow a simpler manipulation of reactive concepts.**
///
/// They exist in two flavours: `Process` and `ProcessMut`.
/// The difference between those two traits relies in how many times they can be ran.
///
/// # Processes
///
/// Simple processes (implementing `Process`) can be ran only one time in their lifetime.
///
/// Implementing `Process` requires two things:
///
/// * Defining `Value`, the type of the value produced by the process;
/// * Implementing the `call` method: given a continuation `c` and a runtime,
///   it should run the process (whatever it does), and finally call `c` over the given runtime,
///   by giving it the value it produced.
///
/// Moreover, any struct implementing `Process` can use various additional methods,
/// for creating other kinds of processes, such as process pausing before calling
/// the continuation (`PausedProcess`), or a process applying a function
/// to its output (`MappedProcess`).
///
/// # Mutable processes
///
/// Mutable processes (implementing `ProcessMut`) can be ran multiple times in their lifetime,
/// and thus can modify their inner environment. A mutable process also is a simple process.
///
/// Implementing `ProcessMut` requires one thing:
///
/// * Implementing the `call_mut` method: given a continuation `c` and a runtime,
///   it should run the process (whatever it does), and finally call `c` over the given runtime,
///   by giving it a couple formed by (1) itself, and (2) the value it produced.
///
pub mod processes;

/// **Signals are a communication mechanisms available for processes.**
///
/// A signal has a unique core (`SignalRuntime`), shared as an inner reference
/// (`SignalRuntimeRef`), manipulated by a more abstract, eponymous trait.
///
/// Processes can be created from structs implementing `Signal`, so that one can, for instante,
/// *await* for a signal, or *emit* one. Those signal-related concepts thus can be mixed with other
/// process concepts (see module `processes`). The semantics used by this module is inspired
/// by the semantics of ReactiveML's signals.
///
/// This module actually is a super-module, which contains both signals' inner mechanisms,
/// as well as two different types of signals (though they all use the same core):
///
/// * *Pure* signals (`PureSignal`), with no value;
/// * *Value* signals (`ValueSignal`), which can hold and gather values during each instant.
///
/// # Pure signals
///
/// Since pure signals do not contain any value, they can simply indicate whether they are
/// emitted or not, during each instant (see module `runtime` for more on instants).
/// They provide basic mechanisms to this extent, for emitting them and awaiting
/// for their presence or absence during an instant.
///
/// # Value signals:
///
/// Value signals, on the other hand, are able to hold values, and even accumulate them if need be.
/// They must be provided with (1) a *default value*, and (2) a *gather function*,
/// used to update the internal value when the same signal is emitted
/// more than one time in a single instant.
///
/// They can also use all the mechanisms available for pure signals.
///
pub mod signals;
