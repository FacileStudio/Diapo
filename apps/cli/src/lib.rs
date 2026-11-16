//! The `diapo` command line: write a deck as one text file, serve it with live reload, build
//! one offline HTML file, and sync it with a Diapo server.
//!
//! Every HTTP call lives in [`api`]. The decisions of `pull` and `push` are the pure functions
//! of [`sync::plan`].

pub mod api;
mod browser;
pub mod cli;
mod commands;
pub mod config;
pub mod diagnose;
pub mod failure;
pub mod login;
mod mime;
pub mod project;
mod run;
mod serve;
pub mod sync;
pub mod ui;
pub mod url;

pub use run::run;
