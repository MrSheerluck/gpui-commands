//! # gpui-commands
//!
//! A coherence layer for the GPUI framework: one registry, one API, and a
//! searchable command palette for every command an application supports.

mod command;
mod fuzzy;
mod palette;
mod registry;

pub use command::Command;
pub use palette::CommandPalette;
pub use registry::CommandRegistry;
