use super::TuiAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandCategory {
    Navigation,
    Input,
    View,
    Session,
    Mode,
    GSD,
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
    let mut cmds = vec![
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
            id: "theme".into(),
            name: "/theme".into(),
            description: "Switch color theme".into(),
            example: "/theme nord".into(),
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
    ];

    // ── GSD SDK Commands ─────────────────────────────────────────────────────
    let gsd_cmds = vec![
        ("/gsd-new-project", "Initialize a fresh GSD planning structure", "/gsd-new-project --auto"),
        ("/gsd-map-codebase", "Scan and index your current codebase state", "/gsd-map-codebase tech"),
        ("/gsd-discuss-phase", "Capture implementation decisions before planning", "/gsd-discuss-phase 1"),
        ("/gsd-plan-phase", "Research + plan + verify for a phase", "/gsd-plan-phase 1"),
        ("/gsd-execute-phase", "Execute all plans in parallel waves", "/gsd-execute-phase 1"),
        ("/gsd-verify-work", "Manual user acceptance testing", "/gsd-verify-work 1"),
        ("/gsd-ship", "Create PR from verified phase work", "/gsd-ship 1"),
        ("/gsd-next", "Automatically advance to the next workflow step", "/gsd-next"),
        ("/gsd-fast", "Inline trivial tasks (skip planning)", "/gsd-fast \"add readme\""),
        ("/gsd-audit-milestone", "Verify milestone achieved definition of done", "/gsd-audit-milestone"),
        ("/gsd-complete-milestone", "Archive milestone and tag release", "/gsd-complete-milestone"),
        ("/gsd-new-milestone", "Start next version from existing codebase", "/gsd-new-milestone v2"),
        ("/gsd-ui-phase", "Generate UI design contract (UI-SPEC.md)", "/gsd-ui-phase 1"),
        ("/gsd-ui-review", "Retroactive visual audit of frontend code", "/gsd-ui-review 1"),
        ("/gsd-progress", "Check project progress and next steps", "/gsd-progress"),
        ("/gsd-settings", "Configure model profile and workflow agents", "/gsd-settings"),
        ("/gsd-debug", "Systematic debugging with persistent state", "/gsd-debug \"fix test failure\""),
        ("/gsd-health", "Validate .planning/ directory integrity", "/gsd-health --repair"),
        ("/gsd-stats", "Display project statistics", "/gsd-stats"),
        ("/gsd-quick", "Execute ad-hoc task with GSD guarantees", "/gsd-quick \"cleanup logs\""),
    ];

    for (name, desc, ex) in gsd_cmds {
        cmds.push(Command {
            id: name.to_lowercase().replace("-", "_").replace("/", ""),
            name: name.into(),
            description: desc.into(),
            example: ex.into(),
            shortcut: None,
            category: CommandCategory::GSD,
            immediate: name == "/gsd-next" || name == "/gsd-progress" || name == "/gsd-stats",
        });
    }

    cmds.extend(vec![
        Command {
            id: "gsd_plan".into(),
            name: "/gsd plan".into(),
            description: "Plan the next GSD phase".into(),
            example: "/gsd plan 5".into(),
            shortcut: Some("/p".into()),
            category: CommandCategory::GSD,
            immediate: false,
        },
        Command {
            id: "gsd_execute".into(),
            name: "/gsd execute".into(),
            description: "Execute the active GSD phase".into(),
            example: "/gsd execute 5".into(),
            shortcut: Some("/e".into()),
            category: CommandCategory::GSD,
            immediate: false,
        },
        Command {
            id: "gsd_verify".into(),
            name: "/gsd verify".into(),
            description: "Verify work completed by the current phase".into(),
            example: "/gsd verify 5".into(),
            shortcut: Some("/v".into()),
            category: CommandCategory::GSD,
            immediate: false,
        },
        Command {
            id: "gsd_discuss".into(),
            name: "/gsd discuss".into(),
            description: "Gather phase context before planning".into(),
            example: "/gsd discuss 5".into(),
            shortcut: Some("/d".into()),
            category: CommandCategory::GSD,
            immediate: false,
        },
        Command {
            id: "gsd_next".into(),
            name: "/gsd next".into(),
            description: "Advance to the next GSD workflow step".into(),
            example: "/gsd next".into(),
            shortcut: Some("/n".into()),
            category: CommandCategory::GSD,
            immediate: true,
        },
        Command {
            id: "gsd_status".into(),
            name: "/gsd status".into(),
            description: "Show the current GSD status".into(),
            example: "/gsd status".into(),
            shortcut: Some("/s".into()),
            category: CommandCategory::GSD,
            immediate: true,
        },
    ]);

    cmds
}

pub fn execute_command(cmd: &Command) -> Option<TuiAction> {
    match cmd.id.as_str() {
        "help" => Some(TuiAction::ShowHelp),
        "clear" => Some(TuiAction::SystemCommand("/clear".into())),
        "agent" => Some(TuiAction::SystemCommand("/mode agentic".into())),
        "chat" => Some(TuiAction::SystemCommand("/mode chat".into())),
        "quit" => Some(TuiAction::Quit),
        // All GSD commands route to SystemCommand which main.rs handles
        id if id.starts_with("gsd_") => Some(TuiAction::SystemCommand(cmd.name.clone())),
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

fn shortcut_matches(command: &Command, filter: &str) -> bool {
    if let Some(shortcut) = &command.shortcut {
        let shortcut = shortcut.to_lowercase();
        let filter = filter.to_lowercase();
        if shortcut.starts_with(&filter) || shortcut == filter {
            return true;
        }
    }

    if command.category == CommandCategory::GSD && filter.starts_with('/') {
        let suffix = command.name.trim_start_matches('/');
        let mut parts = suffix.split_whitespace();
        if let Some(prefix) = parts.next() {
            let initials = parts.filter_map(|part| part.chars().next()).collect::<String>();
            let compact = if initials.is_empty() {
                format!("/{}", prefix)
            } else {
                format!("/{} {}", prefix, initials)
            };
            if filter == format!("/{}", prefix.chars().next().unwrap_or_default())
                || compact.starts_with(filter)
            {
                return true;
            }
        }
    }

    false
}

pub fn match_partial(commands: &[Command], filter: &str) -> Option<Command> {
    let trimmed = filter.trim();
    if trimmed.is_empty() {
        return None;
    }

    commands
        .iter()
        .find(|command| {
            command.name.to_lowercase().starts_with(&trimmed.to_lowercase())
                || command.id.to_lowercase().starts_with(&trimmed.to_lowercase())
                || shortcut_matches(command, trimmed)
        })
        .cloned()
}
