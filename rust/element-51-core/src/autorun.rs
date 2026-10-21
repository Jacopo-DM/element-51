//! When `Run automatically` starts a run: wait for the edits to stop, never
//! run twice at once, and run once more if an edit came in during a run.
//!
//! The rule has no clock and no page: the caller passes the time in
//! milliseconds and starts the timer and the run itself.

/// How long after the last change a run starts.
pub const DEBOUNCE_MS: f64 = 300.0;

/// What the caller does after a change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reaction {
    /// Nothing: the box is not ticked.
    Ignore,
    /// Call `due` after this many milliseconds.
    Wait(f64),
    /// A run is going. One more run is queued; tell the user.
    Queued,
}

#[derive(Clone, Debug, Default)]
pub struct AutoRun {
    ticked: bool,
    last_change: Option<f64>,
    running: bool,
    queued: bool,
}

impl AutoRun {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ticked(&self) -> bool {
        self.ticked
    }

    /// Tick or untick the box. Unticking forgets a change that was waiting
    /// and a queued run; a run that is going is left alone.
    pub fn set_ticked(&mut self, on: bool) {
        self.ticked = on;
        if !on {
            self.last_change = None;
            self.queued = false;
        }
    }

    /// The box was just ticked: the graph is run once at once. True means:
    /// start a run now. If a run is going, one more is queued instead.
    pub fn tick_run(&mut self) -> bool {
        if !self.ticked {
            return false;
        }
        if self.running {
            self.queued = true;
            return false;
        }
        self.running = true;
        true
    }

    /// A change that alters a result happened at `now`.
    pub fn change(&mut self, now: f64) -> Reaction {
        if !self.ticked {
            return Reaction::Ignore;
        }
        if self.running {
            self.queued = true;
            return Reaction::Queued;
        }
        self.last_change = Some(now);
        Reaction::Wait(DEBOUNCE_MS)
    }

    /// A timer from `change` fired at `now`. True means: start a run now.
    /// An earlier timer fires before the newest change has waited long
    /// enough and gets false; the newest change has its own timer.
    pub fn due(&mut self, now: f64) -> bool {
        let Some(last) = self.last_change else {
            return false;
        };
        if !self.ticked || self.running || now - last < DEBOUNCE_MS {
            return false;
        }
        self.last_change = None;
        self.running = true;
        true
    }

    /// A run was started some other way (the Run button).
    pub fn started(&mut self) {
        self.running = true;
    }

    /// The whole run ended. True means: a change came in during it, so start
    /// one more run now.
    pub fn finished(&mut self) -> bool {
        self.running = false;
        if self.queued && self.ticked {
            self.queued = false;
            self.running = true;
            return true;
        }
        self.queued = false;
        false
    }
}
