# gpui-commands

A coherence layer for the [GPUI](https://github.com/zed-industries/zed) framework: one
registry, one API, and a searchable command palette for every command an application
supports.

GPUI already provides the building blocks - the `actions!` macro declares action structs,
`cx.bind_keys` maps keys to actions, and `.on_action()` wires handlers. But there is no
single registry tracking *every command an app supports*, no metadata (name, category,
description) attached to actions, and no reusable command palette. `gpui-commands` sits on
top of GPUI's existing system and provides that missing coherence layer.

## Installation

```toml
[dependencies]
gpui = "0.2"
gpui-commands = "0.1"
```

## Quick start

```rust
use gpui::*;
use gpui_commands::{Command, CommandPalette, CommandRegistry};

actions!(editor, [SaveFile, ToggleSidebar, FormatDocument]);

fn build_app(cx: &mut App) {
    let mut registry = CommandRegistry::new();

    registry.register(
        Command::new("Save File", SaveFile)
            .category("File")
            .keybinding("cmd-s")
            .handler(|window, cx| {
                // save logic
            }),
    );

    registry.register(
        Command::new("Toggle Sidebar", ToggleSidebar)
            .category("View")
            .keybinding("cmd-b")
            .handler(|window, cx| {
                // toggle logic
            }),
    );

    // Palette-only commands are valid too (no keybinding, no category).
    registry.register(
        Command::new("Format Document", FormatDocument)
            .category("Edit")
            .handler(|window, cx| {
                // format logic
            }),
    );

    // One call binds every registered keybinding with GPUI.
    registry.install_keybindings(cx);

    // The palette takes over the window's root view (call after creating
    // your windows). Press cmd-shift-p to open it.
    CommandPalette::install(registry, cx);
}
```

See `examples/basic.rs` for a runnable app:

```bash
cargo run --example basic
```

## The command palette

Press `cmd-shift-p` (configurable via `install_with_trigger`) and a dimmed overlay
appears with a search card: type to filter commands with fuzzy subsequence matching
(`tgsb` finds "Toggle Sidebar"), `↑`/`↓` to move the highlight, `Enter` or a click to
execute, `Escape` or a click outside the card to dismiss. Every command row shows its
name, category, and bound shortcut.

## API

- `Command::new(name, action)` - start building a command (name is what the palette
  shows; `action` is the underlying GPUI action). Builder steps:
  `.category(...)`, `.keybinding(...)`, `.handler(f)` (required before registering).
- `CommandRegistry::new()` / `register(command)` / `commands()` /
  `install_keybindings(cx)` - the single source of truth. Registering the same action
  type twice, or a command without a handler, panics - matching GPUI's own convention.
- `CommandPalette::install(registry, cx)` / `install_with_trigger(registry, trigger, cx)` - store the registry as a global, bind the trigger, and mount the palette.

## How it works

Keybindings are ordinary GPUI keybindings: `cmd-s` dispatches the `SaveFile` action,
which your app handles with its own `.on_action` listeners exactly as before - the crate
never hijacks dispatch. The `.handler()` closures are the palette's activation path: when
you press `Enter` in the palette, the selected command's handler runs with
`(&mut Window, &mut App)`. The palette is installed as the window's permanent root view,
keeping the app's root alive inside it - so app state and focus survive open/close cycles.

## License

MIT
