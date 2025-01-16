#![feature(iterator_try_collect)]

//! Contains all library components for representing and generating crochet patterns.

/// Example patterns used in testing
pub mod examples;
/// The visual scripting component
pub mod parametric;
/// The pattern representation and building as a crochet graph
pub mod pattern;
/// The textual scripting component using Rhai
pub mod script;
