//! Diapo's capture: what the API asks of the isolated render worker, and how the worker does it.
//!
//! [`job`] is the file protocol the two sides share: a job directory, `job.json`, `status.json`
//! and the heartbeat. It needs no browser, so the API depends on this crate without the `chrome`
//! feature. With `chrome`, [`shoot`] loads a compiled deck in `chrome-headless-shell` over the
//! DevTools Protocol pipe, with every request outside the bundle refused, and takes its
//! thumbnail, the stills of its scenes or its PDF. See `docs/capture.md`.
#![deny(unused, dead_code, unsafe_code, missing_docs)]

pub mod job;
mod layout;

#[cfg(feature = "chrome")]
mod browser;
#[cfg(feature = "chrome")]
mod error;
#[cfg(feature = "chrome")]
mod flags;
#[cfg(feature = "chrome")]
mod guard;
#[cfg(feature = "chrome")]
mod merge;
#[cfg(feature = "chrome")]
mod page;
#[cfg(feature = "chrome")]
mod pipe;
#[cfg(feature = "chrome")]
pub mod shoot;
#[cfg(feature = "chrome")]
pub mod tools;

#[cfg(feature = "chrome")]
pub use error::Error;
#[cfg(feature = "chrome")]
pub use tools::Tools;
