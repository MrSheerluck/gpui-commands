//! A minimal GPUI app demonstrating `gpui-commands`: commands registered once
//! in a `CommandRegistry` with keybindings, where every command produces a
//! visible change in the app.
//!
//! Run with: `cargo run --example basic`
//!
//! Keybindings:
//!   cmd-enter        Add Item
//!   cmd-backspace    Delete Last Item
//!   cmd-b            Toggle Sidebar
//!   cmd-s            Save
//!   cmd-shift-f      Format Document

use gpui::{
    actions, App, Application, Bounds, Context, FocusHandle, Focusable, Render, SharedString,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_commands::{Command, CommandRegistry};

actions!(demo, [AddItem, DeleteItem, ToggleSidebar, SaveFile, FormatDocument]);

struct DemoApp {
    focus_handle: FocusHandle,
    items: Vec<SharedString>,
    sidebar_visible: bool,
    status: SharedString,
    saves: u32,
}

impl DemoApp {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            items: vec![
                "Write the roadmap".into(),
                "Scaffold the crate".into(),
                "Ship v1".into(),
            ],
            sidebar_visible: true,
            status: "Ready — press cmd-enter to add an item".into(),
            saves: 0,
        }
    }

    fn add_item(&mut self, cx: &mut Context<Self>) {
        self.items
            .push(format!("New item {}", self.items.len() + 1).into());
        self.status = format!("Added item #{}", self.items.len()).into();
        cx.notify();
    }

    fn delete_item(&mut self, cx: &mut Context<Self>) {
        match self.items.pop() {
            Some(removed) => self.status = format!("Removed \"{removed}\"").into(),
            None => self.status = "Nothing left to delete".into(),
        }
        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        self.status = if self.sidebar_visible {
            "Sidebar shown".into()
        } else {
            "Sidebar hidden".into()
        };
        cx.notify();
    }

    fn save_file(&mut self, cx: &mut Context<Self>) {
        self.saves += 1;
        self.status = format!("Saved ✓ ({saves} saves)", saves = self.saves).into();
        cx.notify();
    }

    fn format_document(&mut self, cx: &mut Context<Self>) {
        self.items.sort();
        self.status = format!("Formatted — {} items sorted", self.items.len()).into();
        cx.notify();
    }
}

impl Focusable for DemoApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .track_focus(&self.focus_handle(cx))
            .on_action({
                let entity = entity.clone();
                move |_: &AddItem, _, cx| {
                    entity.update(cx, |view, cx| view.add_item(cx));
                }
            })
            .on_action({
                let entity = entity.clone();
                move |_: &DeleteItem, _, cx| {
                    entity.update(cx, |view, cx| view.delete_item(cx));
                }
            })
            .on_action({
                let entity = entity.clone();
                move |_: &ToggleSidebar, _, cx| {
                    entity.update(cx, |view, cx| view.toggle_sidebar(cx));
                }
            })
            .on_action({
                let entity = entity.clone();
                move |_: &SaveFile, _, cx| {
                    entity.update(cx, |view, cx| view.save_file(cx));
                }
            })
            .on_action({
                let entity = entity.clone();
                move |_: &FormatDocument, _, cx| {
                    entity.update(cx, |view, cx| view.format_document(cx));
                }
            })
            .child(header(&self.status, self.saves))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .overflow_hidden()
                    .when(self.sidebar_visible, |this| {
                        this.child(sidebar(&self.items))
                    })
                    .child(document(&self.items)),
            )
            .child(footer())
    }
}

fn header(status: &SharedString, saves: u32) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .justify_between()
        .items_center()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(rgb(0x313244))
        .child(div().text_lg().text_color(rgb(0xffffff)).child("gpui-commands demo"))
        .child(
            div()
                .flex()
                .gap_3()
                .child(div().text_sm().text_color(rgb(0xa6e3a1)).child(status.clone()))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x89b4fa))
                        .child(format!("saves: {saves}")),
                ),
        )
}

fn sidebar(items: &[SharedString]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .w(px(180.0))
        .h_full()
        .p_3()
        .bg(rgb(0x181825))
        .border_r_1()
        .border_color(rgb(0x313244))
        .child(div().text_sm().text_color(rgb(0x89b4fa)).child("FILES"))
        .children(["notes.txt", "todo.md", "ideas.md"].map(|name| {
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .hover(|style| style.bg(rgb(0x313244)))
                .child(name)
        }))
        .child(div().mt_3().text_sm().text_color(rgb(0x6c7086)).child(format!(
            "{} items in document",
            items.len()
        )))
}

fn document(items: &[SharedString]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .p_4()
        .child(div().text_sm().text_color(rgb(0x89b4fa)).child("DOCUMENT"))
        .children(items.iter().map(|item| {
            div()
                .flex()
                .flex_row()
                .gap_2()
                .px_2()
                .py_1()
                .child(div().text_color(rgb(0x6c7086)).child("•"))
                .child(item.clone())
        }))
}

fn footer() -> impl IntoElement {
    div()
        .px_4()
        .py_2()
        .border_t_1()
        .border_color(rgb(0x313244))
        .text_sm()
        .text_color(rgb(0x6c7086))
        .child(
            "cmd-enter Add  ·  cmd-backspace Delete  ·  cmd-b Sidebar  ·  cmd-s Save  ·  cmd-shift-f Format",
        )
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(720.0), px(460.0)),
                        cx,
                    ))),
                    ..Default::default()
                },
                |_, cx| cx.new(DemoApp::new),
            )
            .unwrap();

        let mut registry = CommandRegistry::new();

        registry.register(
            Command::new("Add Item", AddItem)
                .category("Edit")
                .keybinding("cmd-enter")
                .handler({
                    let window = window.clone();
                    move |_, cx| {
                        window.update(cx, |view, _, cx| view.add_item(cx)).unwrap();
                    }
                }),
        );

        registry.register(
            Command::new("Delete Last Item", DeleteItem)
                .category("Edit")
                .keybinding("cmd-backspace")
                .handler({
                    let window = window.clone();
                    move |_, cx| {
                        window.update(cx, |view, _, cx| view.delete_item(cx)).unwrap();
                    }
                }),
        );

        registry.register(
            Command::new("Toggle Sidebar", ToggleSidebar)
                .category("View")
                .keybinding("cmd-b")
                .handler({
                    let window = window.clone();
                    move |_, cx| {
                        window.update(cx, |view, _, cx| view.toggle_sidebar(cx)).unwrap();
                    }
                }),
        );

        registry.register(
            Command::new("Save", SaveFile)
                .category("File")
                .keybinding("cmd-s")
                .handler({
                    let window = window.clone();
                    move |_, cx| {
                        window.update(cx, |view, _, cx| view.save_file(cx)).unwrap();
                    }
                }),
        );

        registry.register(
            Command::new("Format Document", FormatDocument)
                .category("Edit")
                .keybinding("cmd-shift-f")
                .handler({
                    let window = window.clone();
                    move |_, cx| {
                        window.update(cx, |view, _, cx| view.format_document(cx)).unwrap();
                    }
                }),
        );

        // One call binds every registered keybinding with GPUI.
        registry.install_keybindings(cx);

        // Give the root view focus so keybindings dispatch to its on_action
        // handlers, and activate the app.
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx));
                cx.activate(true);
            })
            .unwrap();
    });
}
