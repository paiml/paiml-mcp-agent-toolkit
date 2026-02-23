#![cfg_attr(coverage_nightly, coverage(off))]
// Sprint 78: TUI-001 GREEN - Terminal Event Loop Implementation
//
// Minimal implementation to pass TUI-001 RED tests.
// Provides terminal event handling for interactive timeline TUI.
//
// Sprint 79+: Presentar-terminal Brick architecture (ratatui-free)
// Benefits: Jidoka verification gates, zero-allocation rendering, 95% test coverage
#![cfg(feature = "tui")]

// Presentar core types for Brick trait implementation
#[allow(unused_imports)]
use presentar_core::{Brick, BrickAssertion, BrickBudget, Widget};

mod event_loop;
mod keyboard;
mod renderer;
mod stack_navigator;
mod types;
mod variable_inspector;

#[cfg(test)]
mod tests;

// Re-export all public types for backward compatibility
pub use event_loop::EventLoop;
pub use keyboard::KeyboardHandler;
pub use renderer::TimelineRenderer;
pub use stack_navigator::StackFrameNavigator;
pub use types::{KeyCode, PlaybackState, TerminalEvent, TuiAction};
pub use variable_inspector::VariableInspectorView;
