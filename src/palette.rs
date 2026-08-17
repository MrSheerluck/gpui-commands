//! The command palette: a searchable overlay listing every registered
//! command, opened by a configurable trigger keybinding.

use crate::{Command, CommandRegistry};
use gpui::{
    actions, point, App, AnyView, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding,
    KeyDownEvent, MouseButton, Render, SharedString, Window, div, prelude::*, px, rgb, rgba,
};

// The action dispatched when the palette's trigger keybinding is pressed.
actions!(gpui_commands, [ToggleCommandPalette]);

/// A searchable command palette overlay.
///
/// `CommandPalette::install` stores the registry as a GPUI global, binds the
/// trigger keybinding, and replaces each open window's root view with the
/// palette, which keeps the previous root inside itself: while closed it
/// renders the previous root untouched (the palette's wrapper div is in the
/// element tree so the trigger still bubbles to it), and while open it draws
/// a dimmed overlay with the palette card on top. App state is preserved
/// across open/close cycles because the previous root view is never destroyed.
///
/// Call `install` after creating the windows you want the palette to live in.
pub struct CommandPalette {
    previous_root: AnyView,
    previous_focus: Option<FocusHandle>,
    input_focus: FocusHandle,
    open: bool,
    query: SharedString,
    selected_index: usize,
}

impl CommandPalette {
    /// Install the palette with the default trigger (`cmd-shift-p` /
    /// `ctrl-shift-p`) in every open window: stores the registry as a global
    /// and binds the trigger keybinding in the app's keymap.
    pub fn install(registry: CommandRegistry, cx: &mut App) {
        Self::install_with_trigger(registry, "cmd-shift-p", cx);
    }

    /// Like [`CommandPalette::install`], but with a custom trigger
    /// keybinding, e.g. `"cmd-k"`.
    pub fn install_with_trigger(registry: CommandRegistry, trigger: &str, cx: &mut App) {
        cx.set_global(registry);
        cx.bind_keys([KeyBinding::new(trigger, ToggleCommandPalette, None)]);

        let windows = cx.windows();
        for window in windows {
            let _ = window.update(cx, |previous_root, window, cx| {
                window.replace_root(cx, |window, cx| {
                    CommandPalette::new(previous_root, window, cx)
                });
            });
        }
    }

    fn new(previous_root: AnyView, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            previous_root,
            previous_focus: None,
            input_focus: cx.focus_handle(),
            open: false,
            query: SharedString::new(""),
            selected_index: 0,
        }
    }

    fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.previous_focus = window.focused(cx);
            self.selected_index = 0;
            window.focus(&self.input_focus);
        } else {
            if let Some(previous) = &self.previous_focus {
                window.focus(previous);
            }
            self.query = SharedString::new("");
            self.selected_index = 0;
        }
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.open = false;
            if let Some(previous) = &self.previous_focus {
                window.focus(previous);
            }
            self.query = SharedString::new("");
            self.selected_index = 0;
            cx.notify();
        }
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = cx.global::<CommandRegistry>().commands().len();
        if count == 0 {
            return;
        }
        let current = self.selected_index as isize;
        self.selected_index = (current + delta).rem_euclid(count as isize) as usize;
        cx.notify();
    }

    fn execute_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.update_global::<CommandRegistry, _>(|registry, cx| {
            if let Some(command) = registry.commands().get(self.selected_index) {
                command.run(window, cx);
            }
        });
        self.close(window, cx);
    }

    fn select_and_execute(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_index = index;
        self.execute_selected(window, cx);
    }

    fn handle_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        match keystroke.key.as_str() {
            "escape" => self.close(window, cx),
            "backspace" => {
                if !self.query.is_empty() {
                    let mut text = self.query.to_string();
                    text.pop();
                    self.query = text.into();
                    cx.notify();
                }
            }
            "up" => self.move_selection(-1, cx),
            "down" => self.move_selection(1, cx),
            "enter" => self.execute_selected(window, cx),
            _ => {
                // Modifier chords are shortcuts, not text input.
                let modifiers = keystroke.modifiers;
                if modifiers.platform || modifiers.control || modifiers.alt {
                    return;
                }
                if let Some(char) = keystroke.key_char.as_ref() {
                    let mut text = self.query.to_string();
                    text.push_str(char);
                    self.query = text.into();
                    cx.notify();
                }
            }
        }
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.input_focus.clone()
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // The trigger is handled by an element-level on_action on this wrapper
        // div, so it fires during normal action bubbling with direct access to
        // the window — no re-entering the dispatch in progress.
        if self.open {
            let focus = self.input_focus.clone();
            let query = self.query.clone();
            let selected = self.selected_index;
            let commands = cx
                .global::<CommandRegistry>()
                .commands()
                .iter()
                .collect::<Vec<_>>();

            let window_bounds = window.bounds();
            let width = f32::from(window_bounds.size.width);
            let card_bounds = Bounds::from_corners(
                point(px((width - 520.0) / 2.0), px(96.0)),
                point(px((width + 520.0) / 2.0), px(96.0 + 320.0)),
            );

            div()
                .id("command-palette")
                .size_full()
                .relative()
                .on_action({
                    let entity = entity.clone();
                    move |_: &ToggleCommandPalette, window, cx| {
                        entity.update(cx, |palette, cx| palette.toggle(window, cx));
                    }
                })
                .child(self.previous_root.clone())
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .bottom_0()
                        .left_0()
                        .bg(rgba(0x000000b0))
                        .on_mouse_down(
                            MouseButton::Left,
                            {
                                let entity = entity.clone();
                                move |event, window, cx| {
                                    if !card_bounds.contains(&event.position) {
                                        entity.update(cx, |palette, cx| {
                                            palette.close(window, cx)
                                        });
                                    }
                                }
                            },
                        )
                        .flex()
                        .justify_center()
                        .items_start()
                        .pt(px(96.0))
                        .child(card(&entity, &focus, &query, &commands, selected)),
                )
                .into_any_element()
        } else {
            div()
                .id("command-palette")
                .size_full()
                .on_action({
                    let entity = entity.clone();
                    move |_: &ToggleCommandPalette, window, cx| {
                        entity.update(cx, |palette, cx| palette.toggle(window, cx));
                    }
                })
                .child(self.previous_root.clone())
                .into_any_element()
        }
    }
}

fn card(
    entity: &Entity<CommandPalette>,
    focus: &FocusHandle,
    query: &SharedString,
    commands: &[&Command],
    selected: usize,
) -> impl IntoElement {
    let entity = entity.clone();
    let key_entity = entity.clone();
    let focus = focus.clone();
    let query = query.clone();

    div()
        .flex()
        .flex_col()
        .w(px(520.0))
        .h(px(320.0))
        .bg(rgb(0x181825))
        .border_1()
        .border_color(rgb(0x45475a))
        .rounded_lg()
        .shadow_lg()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(rgb(0x313244))
                .child(div().text_color(rgb(0x89b4fa)).child(">"))
                .child(
                    div()
                        .flex_1()
                        .track_focus(&focus)
                        .on_key_down(move |event, window, cx| {
                            key_entity.update(cx, |palette, cx| {
                                palette.handle_key(event, window, cx)
                            });
                        })
                        .child(if query.is_empty() {
                            div()
                                .text_color(rgb(0x6c7086))
                                .child("Search commands…")
                        } else {
                            div().text_color(rgb(0xcdd6f4)).child(query)
                        }),
                ),
        )
        .child(
            div()
                .id("command-list")
                .flex()
                .flex_col()
                .flex_1()
                .overflow_y_scroll()
                .children(
                    commands
                        .iter()
                        .enumerate()
                        .map(|(index, command)| command_row(&entity, command, index, selected)),
                ),
        )
}

fn command_row(
    entity: &Entity<CommandPalette>,
    command: &Command,
    index: usize,
    selected: usize,
) -> impl IntoElement {
    let entity = entity.clone();
    div()
        .flex()
        .flex_row()
        .justify_between()
        .items_center()
        .px_3()
        .py_1()
        .when(index == selected, |this| this.bg(rgb(0x45475a)))
        .hover(|style| style.bg(rgb(0x313244)))
        .on_mouse_down(
            MouseButton::Left,
            move |_, window, cx| {
                entity.update(cx, |palette, cx| {
                    palette.select_and_execute(index, window, cx)
                });
            },
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(div().text_color(rgb(0xcdd6f4)).child(command.name().clone()))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x6c7086))
                        .child(command.category_name().clone()),
                ),
        )
        .when_some(command.binding(), |this, binding| {
            this.child(
                div()
                    .px_2()
                    .py_1()
                    .bg(rgb(0x313244))
                    .rounded_md()
                    .text_sm()
                    .text_color(rgb(0x89b4fa))
                    .child(binding.clone()),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use gpui::{actions, App, Context, FocusHandle, Focusable, Render, TestAppContext, Window, div};

    actions!([PingPalette, PongPalette]);

    struct Placeholder {
        focus: FocusHandle,
    }

    impl Placeholder {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                focus: cx.focus_handle(),
            }
        }
    }

    impl Focusable for Placeholder {
        fn focus_handle(&self, _: &App) -> FocusHandle {
            self.focus.clone()
        }
    }

    impl Render for Placeholder {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .track_focus(&self.focus_handle(cx))
                .child("placeholder")
        }
    }

    fn install_palette(cx: &mut TestAppContext) -> (gpui::AnyWindowHandle, &mut TestAppContext) {
        let window = cx.add_window(|_, cx| Placeholder::new(cx));
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx));
            })
            .unwrap();
        let handle: gpui::AnyWindowHandle = window.into();

        cx.update(|cx| {
            let mut registry = CommandRegistry::new();
            registry.register(Command::new("Ping", PingPalette).handler(|_, _| {}));
            registry.register(Command::new("Pong", PongPalette).handler(|_, _| {}));
            CommandPalette::install(registry, cx);
        });

        (handle, cx)
    }

    fn palette_open(cx: &mut TestAppContext, handle: gpui::AnyWindowHandle) -> bool {
        cx.update(|cx| {
            handle
                .update(cx, |root, _, cx| {
                    let palette = root.downcast::<CommandPalette>().unwrap();
                    palette.read(cx).open
                })
                .unwrap()
        })
    }

    fn palette_selection(cx: &mut TestAppContext, handle: gpui::AnyWindowHandle) -> usize {
        cx.update(|cx| {
            handle
                .update(cx, |root, _, cx| {
                    let palette = root.downcast::<CommandPalette>().unwrap();
                    palette.read(cx).selected_index
                })
                .unwrap()
        })
    }

    #[gpui::test]
    fn install_wraps_root_and_trigger_opens_palette(cx: &mut TestAppContext) {
        let (handle, cx) = install_palette(cx);

        cx.update(|cx| {
            let is_palette = handle
                .update(cx, |root, _, _| root.downcast::<CommandPalette>().is_ok())
                .unwrap();
            assert!(is_palette, "install did not wrap the window's root view");
        });

        cx.simulate_keystrokes(handle, "cmd-shift-p");
        assert!(palette_open(cx, handle), "trigger did not open the palette");

        // Toggle again to close, then again to reopen.
        cx.simulate_keystrokes(handle, "cmd-shift-p");
        cx.simulate_keystrokes(handle, "cmd-shift-p");
        assert!(palette_open(cx, handle), "trigger did not reopen the palette");
    }

    #[gpui::test]
    fn arrow_keys_move_selection_with_wraparound(cx: &mut TestAppContext) {
        let (handle, cx) = install_palette(cx);

        cx.simulate_keystrokes(handle, "cmd-shift-p");
        assert_eq!(palette_selection(cx, handle), 0);

        cx.simulate_keystrokes(handle, "down");
        assert_eq!(palette_selection(cx, handle), 1);

        // Wraps back around to the first command.
        cx.simulate_keystrokes(handle, "down");
        assert_eq!(palette_selection(cx, handle), 0);

        cx.simulate_keystrokes(handle, "up");
        assert_eq!(palette_selection(cx, handle), 1);
    }

    #[gpui::test]
    fn enter_executes_selected_command(cx: &mut TestAppContext) {
        let window = cx.add_window(|_, cx| Placeholder::new(cx));
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx));
            })
            .unwrap();
        let handle: gpui::AnyWindowHandle = window.into();

        let executed = std::rc::Rc::new(std::cell::RefCell::new(0));
        let counter = executed.clone();

        cx.update(|cx| {
            let mut registry = CommandRegistry::new();
            registry.register(
                Command::new("Increment", PingPalette).handler(move |_, _| {
                    *counter.borrow_mut() += 1;
                }),
            );
            CommandPalette::install(registry, cx);
        });

        cx.simulate_keystrokes(handle, "cmd-shift-p");
        cx.simulate_keystrokes(handle, "enter");

        assert_eq!(
            *executed.borrow(),
            1,
            "enter did not run the selected command's handler"
        );
        assert!(!palette_open(cx, handle), "enter did not close the palette");
    }
}