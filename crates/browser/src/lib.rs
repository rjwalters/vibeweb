//! Vibeweb browser library crate.
//!
//! This crate provides the core browser functionality for Vibeweb,
//! including navigation history and session management.

mod history;

pub use history::{History, HistoryEntry};
