//! What the capture thread and the status thread of one job share.

use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use capture::Error;
use capture::job::{HEARTBEAT, Layout, State, Status, now};

use crate::config::Limits;
use crate::{files, spec};

/// How often the status file is rewritten while a job runs.
const BEAT: Duration = Duration::from_millis(500);

/// The width of a thumbnail when the job names a wider stage.
pub(crate) const THUMBNAIL_WIDTH: u32 = 1280;

const CANCELLED: u8 = 1;
const TIMED_OUT: u8 = 2;
const TOO_MANY: u8 = 3;

/// What the capture thread and the status thread share.
#[derive(Default)]
pub(crate) struct Shared {
    done: AtomicU32,
    total: AtomicU32,
    /// Why the capture must stop: 0 while it may go on.
    stop: AtomicU8,
    pub(crate) over: AtomicBool,
}

impl Shared {
    fn status(&self, state: State) -> Status {
        spec::status(state, self.done.load(Ordering::Relaxed), self.total.load(Ordering::Relaxed))
    }

    /// Rewrites the status and the heartbeat twice a second until the capture is over, and
    /// watches for the `cancel` file and the wall clock meanwhile.
    pub(crate) fn beat(&self, layout: &Layout, wall: Duration) {
        let deadline = Instant::now() + wall;
        let heartbeat = layout.dir.parent().unwrap_or(&layout.dir).join(HEARTBEAT);
        while !self.over.load(Ordering::Relaxed) {
            spec::save(&layout.dir, &self.status(State::Running));
            let _ = files::write(&heartbeat, now().as_bytes());
            let cancelled = files::exists(&layout.cancel());
            let stop = if cancelled { CANCELLED } else { u8::from(Instant::now() > deadline) * TIMED_OUT };
            let _ = self.stop.compare_exchange(0, stop, Ordering::Relaxed, Ordering::Relaxed);
            std::thread::sleep(BEAT);
        }
    }

    pub(crate) fn progress(&self, done: u32, total: u32, most: u32) -> ControlFlow<()> {
        self.done.store(done, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
        if total > most {
            self.stop.store(TOO_MANY, Ordering::Relaxed);
        }
        if self.stop.load(Ordering::Relaxed) == 0 { ControlFlow::Continue(()) } else { ControlFlow::Break(()) }
    }

    pub(crate) fn verdict(&self, captured: Result<u32, Error>, limits: &Limits) -> Status {
        let failed = |reason: String| Status { error: Some(reason), ..self.status(State::Failed) };
        match (captured, self.stop.load(Ordering::Relaxed)) {
            (Ok(made), _) => spec::status(State::Done, made, made),
            (Err(Error::Cancelled), CANCELLED) => self.status(State::Cancelled),
            (Err(Error::Cancelled), TOO_MANY) => failed(format!("the deck makes more than {} pages", limits.pages)),
            (Err(_), TIMED_OUT) | (Err(Error::Cancelled), _) => {
                failed(format!("the capture passed its {} s time limit", limits.wall.as_secs()))
            }
            (Err(error), _) => failed(error.to_string()),
        }
    }
}
