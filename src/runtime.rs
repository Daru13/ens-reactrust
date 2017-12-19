use continuations::Continuation;
use signals::runtime::SignalRuntimeRef;


///////////////////////////////////////////////////////////////////////////////////////////////////
// RUNTIME
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Runtime for executing reactive continuations.
pub struct Runtime {
  // Pools of continuations to execute at different points in time
  current_instant_tasks: Vec<Box<Continuation<()>>>,
  next_instant_tasks   : Vec<Box<Continuation<()>>>,
  end_of_instant_tasks : Vec<Box<Continuation<()>>>
}


impl Runtime {
  /// Creates a new `Runtime`.
  pub fn new() -> Self {
    Self {
      current_instant_tasks: Vec::new(),
      next_instant_tasks   : Vec::new(),
      end_of_instant_tasks : Vec::new()
    }
  }

  /// Executes instants until all work is completed.
  pub fn execute(&mut self) {
    let mut remaining_work = true;

    while remaining_work {
      remaining_work = self.instant();
    }
  }

  /// Executes a single instant to completion. Indicates if more work remains to be done.
  pub fn instant(&mut self) -> bool {
    println!("Running instant (cur: {}, endof: {}, next: {})",
      self.current_instant_tasks.len(),
      self.end_of_instant_tasks.len(),
      self.next_instant_tasks.len()
    );

    // Run tasks belonging to the current instant, then tasks belonging to the end of current instant
    while self.current_instant() {}
    while self.end_of_instant() {}

    // Move to the next instant and return whether there are new tasks to run or not
    return self.move_to_next_instant();
  }

  /// Prepare the runtime for moving to the next instant, and update its state accordingly
  /// Returns whether there are next ionstant tasks to run or not
  fn move_to_next_instant(&mut self) -> bool {
    println!("Moving to next instant...");

    // Clear current instant tasks
    self.current_instant_tasks.clear();
    self.end_of_instant_tasks.clear();

    // Next instant tasks now are current instant tasks
    self.current_instant_tasks.append(&mut self.next_instant_tasks);

    return !self.current_instant_tasks.is_empty();
  }

  /// Execute a single task registered as a current instant task
  fn current_instant(&mut self) -> bool {
    if self.current_instant_tasks.is_empty() {
      return false;
    }

    println!("Current instant (cur: {}, endof: {}, next: {})",
      self.current_instant_tasks.len(),
      self.end_of_instant_tasks.len(),
      self.next_instant_tasks.len()
    );

    let task = self.current_instant_tasks.pop();
    match task {
      Some(continuation) => continuation.call_box(self, ()),
      None               => () // Should not happen
    };

    return !self.current_instant_tasks.is_empty();
  }

  /// Execute a single task registered as an end-of-instant task
  fn end_of_instant(&mut self) -> bool {
    if self.end_of_instant_tasks.is_empty() {
      return false;
    }

    println!("End of instant (cur: {}, endof: {}, next: {})",
      self.current_instant_tasks.len(),
      self.end_of_instant_tasks.len(),
      self.next_instant_tasks.len()
    );

    let task = self.end_of_instant_tasks.pop();
    match task {
      Some(continuation) => continuation.call_box(self, ()),
      None               => () // Should not happen
    };

    return !self.end_of_instant_tasks.is_empty();
  }

  /// Registers a continuation to execute on the current instant.
  pub fn on_current_instant(&mut self, c: Box<Continuation<()>>) {
    println!("On current instant (cur: {}, endof: {}, next: {})",
      self.current_instant_tasks.len(),
      self.end_of_instant_tasks.len(),
      self.next_instant_tasks.len()
    );

    self.current_instant_tasks.push(c);
  }

  /// Registers a continuation to execute at the next instant.
  pub fn on_next_instant(&mut self, c: Box<Continuation<()>>) {
    self.next_instant_tasks.push(c);
  }

  /// Registers a continuation to execute at the end of the instant. Runtime calls for `c`
  /// behave as if they where executed during the next instant.
  pub fn on_end_of_instant(&mut self, c: Box<Continuation<()>>) {
    println!("On end of instant (cur: {}, endof: {}, next: {})",
      self.current_instant_tasks.len(),
      self.end_of_instant_tasks.len(),
      self.next_instant_tasks.len()
    );

    self.end_of_instant_tasks.push(c);
  }
}
