//! Code-Iot core: everything that is pure logic lives here so it can be
//! unit-tested without a database or a browser, and shared by both the
//! server and the WebAssembly frontend.
//!
//! * [`model`]    – courses, sections, lessons and lesson steps
//! * [`content`]  – the built-in course catalog
//! * [`api`]      – JSON types exchanged between server and web app
//! * [`grading`]  – checking a learner's answer against a step
//! * [`progress`] – XP, levels, streaks, weeks and badges
//! * [`shuffle`]  – deterministic shuffling for answer tiles
//!
//! There is intentionally no locking anywhere: every course and every lesson
//! is open from the start.

pub mod access;
pub mod api;
pub mod content;
pub mod grading;
pub mod i18n;
pub mod model;
pub mod progress;
pub mod shuffle;

pub use content::{catalog, find_course, find_lesson};
