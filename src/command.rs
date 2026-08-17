use gpui::{Action, App, SharedString, Window};

/// The signature of a command handler: invoked with the window and app
/// contexts whenever the command is executed.
type CommandHandler = dyn Fn(&mut Window, &mut App);

/// A single command: its display name, category, underlying GPUI action,
/// optional keybinding, and the handler invoked when it is executed.
pub struct Command {
    pub(crate) name: SharedString,
    pub(crate) action: Box<dyn Action>,
    pub(crate) category: SharedString,
    pub(crate) keybinding: Option<SharedString>,
    pub(crate) handler: Option<Box<CommandHandler>>,
}

impl Command {
    /// Start building a command. `name` is what appears in the palette;
    /// `action` is the GPUI action this command wraps.
    pub fn new(name: impl Into<SharedString>, action: impl Action) -> Self {
        Self {
            name: name.into(),
            action: Box::new(action),
            category: SharedString::new_static("General"),
            keybinding: None,
            handler: None,
        }
    }

    /// Set the category this command is grouped under in the palette.
    pub fn category(mut self, category: impl Into<SharedString>) -> Self {
        self.category = category.into();
        self
    }

    /// Record a keybinding for this command, e.g. `"cmd-s"`.
    pub fn keybinding(mut self, binding: impl Into<SharedString>) -> Self {
        self.keybinding = Some(binding.into());
        self
    }

    /// Set the handler invoked when this command is executed.
    pub fn handler(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Box::new(f));
        self
    }

    /// The display name shown in the palette.
    pub fn name(&self) -> &SharedString {
        &self.name
    }

    /// The category this command is grouped under.
    pub fn category_name(&self) -> &SharedString {
        &self.category
    }

    /// The bound keybinding, if any.
    pub fn binding(&self) -> Option<&SharedString> {
        self.keybinding.as_ref()
    }

    /// The underlying GPUI action.
    pub fn action(&self) -> &dyn Action {
        self.action.as_ref()
    }

    /// Execute this command's handler.
    pub fn run(&self, window: &mut Window, cx: &mut App) {
        if let Some(handler) = &self.handler {
            handler(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::actions;

    actions!([SaveFile, ToggleSidebar]);

    #[test]
    fn builder_defaults() {
        let command = Command::new("Save File", SaveFile);
        assert_eq!(command.name(), "Save File");
        assert_eq!(command.category_name(), "General");
        assert!(command.binding().is_none());
        assert_eq!(command.action().name(), "SaveFile");
    }

    #[test]
    fn builder_chaining() {
        let command = Command::new("Toggle Sidebar", ToggleSidebar)
            .category("View")
            .keybinding("cmd-b")
            .handler(|_, _| {});
        assert_eq!(command.category_name(), "View");
        assert_eq!(command.binding().unwrap(), "cmd-b");
        assert!(command.handler.is_some());
    }
}
