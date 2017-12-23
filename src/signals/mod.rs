/// **The core of signals, used internally only.**
///
/// This sub-module contains all the data required by a signal to work,
/// as well as various methods used to interface a more abstract, actual signal, with
/// lower level mechanisms such as the runtime and continuations (see related modules for details).
///
/// There is only one type of core, used by both signals:
/// pure signals actually are signals emitting and gathering unit `()` typed values.
///
pub mod runtime;

/// **Generic signal and signal processes.**
///
/// This sub-module contains the definition of the generic `Signal` trait, which must be implemented
/// by any structure which represents and actual signal. This trait exposes several methods
/// for building processes based on signals, which are also defined in this module.
pub mod signals;

/// **Pure signals.**
///
/// This sub-module contains the implementation of pure signals (i.e. holding no value).
///
pub mod pure_signal;

/// **Value signals.**
///
/// This sub-module contains the implementation of value signals.
///
pub mod value_signal;
