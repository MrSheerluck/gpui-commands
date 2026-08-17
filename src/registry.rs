use crate::Command;
use std::any::TypeId;
use std::collections::HashMap;

/// The single source of truth for every command an app supports.
#[derive(Default)]
pub struct CommandRegistry {
    commands: Vec<Command>,
    by_action_type: HashMap<TypeId, ()>,
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command.
    ///
    /// # Panics
    ///
    /// - If the command has no handler.
    /// - If a command for the same action type is already registered,
    ///   mirroring GPUI's own duplicate-action convention.
    #[track_caller]
    pub fn register(&mut self, command: Command) {
        assert!(
            command.handler.is_some(),
            "command `{}` requires a handler",
            command.name()
        );
        let type_id = command.action().as_any().type_id();
        assert!(
            self.by_action_type.insert(type_id, ()).is_none(),
            "command for action `{}` already registered",
            command.action().name()
        );
        self.commands.push(command);
    }

    /// All registered commands, in registration order.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use gpui::actions;

    actions!([Ping, Pong]);

    #[test]
    fn register_and_introspect() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("Ping", Ping).handler(|_, _| {}));
        registry.register(Command::new("Pong", Pong).handler(|_, _| {}));
        assert_eq!(registry.commands().len(), 2);
        assert_eq!(registry.commands()[0].name(), "Ping");
        assert_eq!(registry.commands()[1].category_name(), "General");
    }

    #[test]
    #[should_panic(expected = "already registered")]
    fn duplicate_action_type_panics() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("Ping", Ping).handler(|_, _| {}));
        registry.register(Command::new("Ping Again", Ping).handler(|_, _| {}));
    }

    #[test]
    #[should_panic(expected = "requires a handler")]
    fn missing_handler_panics() {
        let mut registry = CommandRegistry::new();
        registry.register(Command::new("Ping", Ping));
    }
}
