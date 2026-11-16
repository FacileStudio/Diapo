//! Assets: the files of a deck, named by the relative path its source writes, stored once by
//! content on the data volume.

pub mod files;
pub mod intake;
pub mod name;
mod router;
mod serve;
pub mod service;
pub mod sniff;
pub mod sweep;

pub use router::routes;
