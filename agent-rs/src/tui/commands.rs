use super::TuiAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandCategory {
    Navigation,
    Input,
    View,
    Session,
    Mode,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub example: String,
    pub shortcut: Option<String>,
    pub category: CommandCategory,
    /// If true, the command dispatches immediately (no arguments needed).
    pub immediate: bool,
}

pub fn default_commands() -> Vec<Command> {
    vec![
        Command {
            id: "help".into(),
            name: "/help".into(),
            description: "Display keyboard shortcuts and available commands".into(),
            example: "/help".into(),
            shortcut: Some("F1".into()),
            category: CommandCategory::Navigation,
            immediate: true,
        },
        Command {
            id: "clear".into(),
            name: "/clear".into(),
            description: "Wipe chat & context history entirely".into(),
            example: "/clear".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: true,
        },
        Command {
            id: "model".into(),
            name: "/model".into(),
            description: "Show or change the active model".into(),
            example: "/model llama3".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: false,
        },
        Command {
            id: "agent".into(),
            name: "/agent".into(),
            description: "Switch to agentic mode (tools enabled)".into(),
            example: "/agent".into(),
            shortcut: None,
            category: CommandCategory::Mode,
            immediate: true,
        },
        Command {
            id: "chat".into(),
            name: "/chat".into(),
            description: "Switch to chat mode (concise, no tools)".into(),
            example: "/chat".into(),
            shortcut: None,
            category: CommandCategory::Mode,
            immediate: true,
        },
        Command {
            id: "gsd_plan".into(),
            name: "/gsd plan".into(),
            description: "Plan the next GSD orchestration phase".into(),
            example: "/gsd plan \"fix network issues\"".into(),
            shortcut: None,
            category: CommandCategory::Mode,
            immediate: false,
        },
        Command {
            id: "gsd_execute".into(),
            name: "/gsd execute".into(),
            description: "Execute the current GSD phase plan".into(),
            example: "/gsd execute".into(),
            shortcut: None,
            category: CommandCategory::Mode,
            immediate: true,
        },
        Command {
            id: "undo".into(),
            name: "/undo".into(),
            description: "Undo the last action or message".into(),
            example: "/undo".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: false,
        },
        Command {
            id: "save".into(),
            name: "/save".into(),
            description: "Save the current session to a file".into(),
            example: "/save my_session".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: false,
        },
        Command {
            id: "load".into(),
            name: "/load".into(),
            description: "Load a previously saved session".into(),
            example: "/load my_session".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: false,
        },
        Command {
            id: "resume".into(),
            name: "/resume".into(),
            description: "Resume the latest autosaved session".into(),
            example: "/resume".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: true,
        },
        Command {
            id: "export".into(),
            name: "/export".into(),
            description: "Export conversation to markdown file".into(),
            example: "/export chat.md".into(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: false,
        },
        Command {
            id: "theme".into(),
            name: "/theme".into(),
            description: "Switch color theme (dark, light, nord, gruvbox)".into(),
            example: "/theme nord".into(),
            shortcut: None,
            category: CommandCategory::View,
            immediate: false,
        },
        Command {
            id: "layout".into(),
            name: "/layout".into(),
            description: "Change layout mode (wide or compact)".into(),
            example: "/layout compact".into(),
            shortcut: None,
            category: CommandCategory::View,
            immediate: false,
        },
        Command {
            id: "quit".into(),
            name: "/quit".into(),
            description: "Exit the application".into(),
            example: "/quit".into(),
            shortcut: Some("Ctrl+C×2".into()),
            category: CommandCategory::Session,
            immediate: true,
        },
    ]
}

pub fn execute_command(cmd: &Command) -> Option<TuiAction> {
    match cmd.id.as_str() {
        "help" => Some(TuiAction::ShowHelp),
        "clear" => Some(TuiAction::SystemCommand("/clear".into())),
        "agent" => Some(TuiAction::SystemCommand("/mode agentic".into())),
        "chat" => Some(TuiAction::SystemCommand("/mode chat".into())),
        "save" => Some(TuiAction::SystemCommand("/save".into())),
        "load" => Some(TuiAction::SystemCommand("/load".into())),
        "resume" => Some(TuiAction::SystemCommand("/resume".into())),
        "gsd_plan" => Some(TuiAction::SystemCommand("/gsd plan".into())),
        "gsd_execute" => Some(TuiAction::SystemCommand("/gsd execute".into())),
        "quit" => Some(TuiAction::Quit),
        _ => None,
    }
}

/// Filter commands by matching against id, name, description, or example text.
pub fn filter_commands(commands: &[Command], filter: &str) -> Vec<Command> {
    if filter.trim().is_empty() {
        return commands.to_vec();
    }

    let filter_lower = filter.to_lowercase();
    commands
        .iter()
        .filter(|c| {
            c.id.to_lowercase().contains(&filter_lower)
                || c.name.to_lowercase().contains(&filter_lower)
                || c.description.to_lowercase().contains(&filter_lower)
                || c.example.to_lowercase().contains(&filter_lower)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_commands_have_expected_baseline() {
        let cmds = default_commands();
        assert!(cmds.len() >= 11);
        assert!(cmds.iter().any(|c| c.id == "help"));
        assert!(cmds.iter().any(|c| c.id == "theme"));
        assert!(cmds.iter().any(|c| c.id == "quit"));
        assert!(cmds.iter().any(|c| c.id == "clear"));
        assert!(cmds.iter().any(|c| c.id == "agent"));
        assert!(cmds.iter().any(|c| c.id == "chat"));
        assert!(cmds.iter().any(|c| c.id == "gsd_plan"));
        assert!(cmds.iter().any(|c| c.id == "gsd_execute"));
        assert!(cmds.iter().any(|c| c.id == "resume"));
    }

    #[test]
    fn execute_command_maps_known_ids() {
        let cmd = Command {
            id: "help".to_string(),
            name: "/help".to_string(),
            description: "Display help".to_string(),
            example: "/help".to_string(),
            shortcut: None,
            category: CommandCategory::Navigation,
            immediate: true,
        };
        assert!(matches!(execute_command(&cmd), Some(TuiAction::ShowHelp)));

        let resume = Command {
            id: "resume".to_string(),
            name: "/resume".to_string(),
            description: "Resume latest".to_string(),
            example: "/resume".to_string(),
            shortcut: None,
            category: CommandCategory::Session,
            immediate: true,
        };
        assert!(matches!(
            execute_command(&resume),
            Some(TuiAction::SystemCommand(cmd)) if cmd == "/resume"
        ));
    }

    #[test]
    fn filter_commands_matches_name_and_description() {
        let cmds = default_commands();
        let by_name = filter_commands(&cmds, "clear");
        assert!(by_name.iter().any(|c| c.id == "clear"));

        let by_desc = filter_commands(&cmds, "keyboard");
        assert!(by_desc.iter().any(|c| c.id == "help"));
    }

    #[test]
    fn filter_commands_matches_example_text() {
        let cmds = default_commands();
        let by_example = filter_commands(&cmds, "llama3");
        assert!(by_example.iter().any(|c| c.id == "model"));
    }

    #[test]
    fn immediate_commands_are_correct() {
        let cmds = default_commands();
        let immediate_ids: Vec<&str> = cmds
            .iter()
            .filter(|c| c.immediate)
            .map(|c| c.id.as_str())
            .collect();
        assert!(immediate_ids.contains(&"help"));
        assert!(immediate_ids.contains(&"clear"));
        assert!(immediate_ids.contains(&"agent"));
        assert!(immediate_ids.contains(&"chat"));
        assert!(immediate_ids.contains(&"resume"));
        assert!(immediate_ids.contains(&"quit"));
        // Non-immediate
        let non_immediate: Vec<&str> = cmds
            .iter()
            .filter(|c| !c.immediate)
            .map(|c| c.id.as_str())
            .collect();
        assert!(non_immediate.contains(&"model"));
        assert!(non_immediate.contains(&"theme"));
        assert!(non_immediate.contains(&"layout"));
    }
}
