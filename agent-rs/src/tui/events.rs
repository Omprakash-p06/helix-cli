use crossterm::event::{KeyCode, KeyEvent};

use super::commands::filter_commands;
use super::state::TuiState;
use super::TuiAction;

/// Handle keyboard input when the command palette is open.
pub fn handle_command_palette_input(key: KeyEvent, state: &mut TuiState) -> Option<TuiAction> {
    match key.code {
        KeyCode::Esc => {
            state.command_palette.visible = false;
            state.command_palette.filter.clear();
            state.command_palette.selected_index = 0;
            Some(TuiAction::CloseCommandPalette)
        }
        KeyCode::Char(c) => {
            state.command_palette.filter.push(c);
            // Re-filter and clamp selection
            let filtered = filter_commands(
                &state.command_palette.commands,
                &state.command_palette.filter,
            );
            if state.command_palette.selected_index >= filtered.len() {
                state.command_palette.selected_index = 0;
            }
            None
        }
        KeyCode::Backspace => {
            state.command_palette.filter.pop();
            let filtered = filter_commands(
                &state.command_palette.commands,
                &state.command_palette.filter,
            );
            if state.command_palette.selected_index >= filtered.len() {
                state.command_palette.selected_index = 0;
            }
            None
        }
        KeyCode::Up => {
            state.command_palette.selected_index =
                state.command_palette.selected_index.saturating_sub(1);
            None
        }
        KeyCode::Down => {
            let filtered = filter_commands(
                &state.command_palette.commands,
                &state.command_palette.filter,
            );
            let max = filtered.len().saturating_sub(1);
            if state.command_palette.selected_index < max {
                state.command_palette.selected_index += 1;
            }
            None
        }
        KeyCode::Enter => {
            let filtered = filter_commands(
                &state.command_palette.commands,
                &state.command_palette.filter,
            );
            if let Some(cmd) = filtered.get(state.command_palette.selected_index) {
                state.command_palette.visible = false;
                state.command_palette.filter.clear();
                state.command_palette.selected_index = 0;
                Some(TuiAction::SelectCommand(
                    // Find the index in the original unfiltered list
                    state
                        .command_palette
                        .commands
                        .iter()
                        .position(|c| c.id == cmd.id)
                        .unwrap_or(0),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_closes_palette() {
        let mut state = TuiState::new();
        state.command_palette.visible = true;
        let action = handle_command_palette_input(
            KeyEvent::new(KeyCode::Esc, Default::default()),
            &mut state,
        );
        assert!(matches!(action, Some(TuiAction::CloseCommandPalette)));
        assert!(!state.command_palette.visible);
    }

    #[test]
    fn char_appends_filter() {
        let mut state = TuiState::new();
        let action = handle_command_palette_input(
            KeyEvent::new(KeyCode::Char('h'), Default::default()),
            &mut state,
        );
        assert!(action.is_none());
        assert_eq!(state.command_palette.filter, "h");
    }

    #[test]
    fn enter_selects_filtered_command() {
        let mut state = TuiState::new();
        state.command_palette.visible = true;
        // Select first command (should be /help)
        state.command_palette.selected_index = 0;
        let action = handle_command_palette_input(
            KeyEvent::new(KeyCode::Enter, Default::default()),
            &mut state,
        );
        assert!(matches!(action, Some(TuiAction::SelectCommand(_))));
        assert!(!state.command_palette.visible);
    }
}
