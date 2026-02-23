#![cfg_attr(coverage_nightly, coverage(off))]
// KeyboardHandler - Keyboard shortcut management

use std::collections::HashMap;

use super::types::{KeyCode, TerminalEvent, TuiAction};

// ============================================================================
// KeyboardHandler - Keyboard shortcut management
// ============================================================================

/// Keyboard shortcut handler
pub struct KeyboardHandler {
    /// Key bindings map
    bindings: HashMap<KeyCode, TuiAction>,
}

impl KeyboardHandler {
    /// Create new keyboard handler with default bindings
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // Default key bindings
        bindings.insert(KeyCode::Right, TuiAction::NextFrame);
        bindings.insert(KeyCode::Left, TuiAction::PreviousFrame);
        bindings.insert(KeyCode::Char(' '), TuiAction::TogglePlayback);
        bindings.insert(KeyCode::Home, TuiAction::JumpToStart);
        bindings.insert(KeyCode::End, TuiAction::JumpToEnd);
        bindings.insert(KeyCode::Char('q'), TuiAction::Quit);

        Self { bindings }
    }

    /// Check if handler has default bindings
    pub fn has_default_bindings(&self) -> bool {
        self.bindings.contains_key(&KeyCode::Right)
            && self.bindings.contains_key(&KeyCode::Left)
            && self.bindings.contains_key(&KeyCode::Char(' '))
    }

    /// Bind a key to an action
    pub fn bind_key(&mut self, key: KeyCode, action: TuiAction) {
        self.bindings.insert(key, action);
    }

    /// Unbind a key
    pub fn unbind_key(&mut self, key: KeyCode) {
        self.bindings.remove(&key);
    }

    /// Get action for key
    pub fn get_action(&self, key: KeyCode) -> Option<TuiAction> {
        self.bindings.get(&key).copied()
    }

    /// Check if key is bound
    pub fn is_key_bound(&self, key: KeyCode) -> bool {
        self.bindings.contains_key(&key)
    }

    /// Handle terminal event and return action
    pub fn handle_event(&self, event: &TerminalEvent) -> Option<TuiAction> {
        match event {
            TerminalEvent::Key(key_code) => self.get_action(*key_code),
            _ => None,
        }
    }

    /// List all current bindings
    pub fn list_bindings(&self) -> Vec<(KeyCode, TuiAction)> {
        self.bindings.iter().map(|(k, a)| (*k, *a)).collect()
    }

    /// Generate help text for all bindings
    pub fn generate_help_text(&self) -> String {
        let mut help = String::new();

        // Sort bindings for consistent output
        let mut bindings: Vec<_> = self.bindings.iter().collect();
        bindings.sort_by_key(|(k, _)| format!("{:?}", k));

        for (key, action) in bindings {
            let key_str = match key {
                KeyCode::Char(' ') => "Space".to_string(),
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Right => "→".to_string(),
                KeyCode::Left => "←".to_string(),
                KeyCode::Home => "Home".to_string(),
                KeyCode::End => "End".to_string(),
            };
            help.push_str(&format!("{}: {} | ", key_str, action.description()));
        }

        // Remove trailing " | "
        if help.len() >= 3 {
            help.truncate(help.len() - 3);
        }

        help
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}
