//! # gpui-commands
//!
//! A coherence layer for the GPUI framework: one registry, one API, and a
//! searchable command palette for every command an application supports.
//!
//! - [`Command`] — a display name, category, underlying GPUI action, optional
//!   keybinding, and handler, built with a chained builder.
//! - [`CommandRegistry`] — the single source of truth for every command an
//!   app registers, with automatic `bind_keys` registration.
//! - [`CommandPalette`] — a searchable, keyboard-driven overlay over the app,
//!   opened with a configurable trigger keybinding (`cmd-shift-p` by default).
//!
//! See the crate README for a full quick-start example.

mod command;
mod fuzzy;
mod palette;
mod registry;

pub use command::Command;
pub use palette::CommandPalette;
pub use registry::CommandRegistry;
