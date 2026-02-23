#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

// ============================================================================
// TerminalEvent and KeyCode tests
// ============================================================================

#[test]
fn test_terminal_event_key() {
    let event = TerminalEvent::Key(KeyCode::Char('a'));
    assert_eq!(event, TerminalEvent::Key(KeyCode::Char('a')));
}

#[test]
fn test_terminal_event_resize() {
    let event = TerminalEvent::Resize(80, 24);
    assert_eq!(event, TerminalEvent::Resize(80, 24));
}

#[test]
fn test_terminal_event_mouse() {
    let event = TerminalEvent::Mouse(10, 20);
    assert_eq!(event, TerminalEvent::Mouse(10, 20));
}

#[test]
fn test_key_code_char() {
    let key = KeyCode::Char('x');
    assert_eq!(key, KeyCode::Char('x'));
}

#[test]
fn test_key_code_arrows() {
    assert_ne!(KeyCode::Left, KeyCode::Right);
    assert_eq!(KeyCode::Left, KeyCode::Left);
}

#[test]
fn test_key_code_home_end() {
    assert_ne!(KeyCode::Home, KeyCode::End);
}

// ============================================================================
// EventLoop tests
// ============================================================================

#[test]
fn test_event_loop_new() {
    let event_loop = EventLoop::new();
    assert!(!event_loop.is_running());
    assert!(!event_loop.is_raw_mode_enabled());
}

#[test]
fn test_event_loop_default() {
    let event_loop = EventLoop::default();
    assert!(!event_loop.is_running());
}

#[test]
fn test_event_loop_start_stop() {
    let mut event_loop = EventLoop::new();
    assert!(!event_loop.is_running());

    event_loop.start();
    assert!(event_loop.is_running());

    event_loop.stop();
    assert!(!event_loop.is_running());
}

#[test]
fn test_event_loop_queue_event() {
    let mut event_loop = EventLoop::new();
    let event = TerminalEvent::Key(KeyCode::Char('a'));

    event_loop.queue_event(event.clone());

    let next = event_loop.next_queued_event();
    assert_eq!(next, Some(event));
}

#[test]
fn test_event_loop_queue_multiple_events() {
    let mut event_loop = EventLoop::new();

    event_loop.queue_event(TerminalEvent::Key(KeyCode::Char('a')));
    event_loop.queue_event(TerminalEvent::Key(KeyCode::Char('b')));
    event_loop.queue_event(TerminalEvent::Resize(100, 50));

    assert_eq!(
        event_loop.next_queued_event(),
        Some(TerminalEvent::Key(KeyCode::Char('a')))
    );
    assert_eq!(
        event_loop.next_queued_event(),
        Some(TerminalEvent::Key(KeyCode::Char('b')))
    );
    assert_eq!(
        event_loop.next_queued_event(),
        Some(TerminalEvent::Resize(100, 50))
    );
    assert_eq!(event_loop.next_queued_event(), None);
}

#[test]
fn test_event_loop_parse_event_key() {
    let event_loop = EventLoop::new();
    let event = TerminalEvent::Key(KeyCode::Right);

    let parsed = event_loop.parse_event(event.clone());
    assert_eq!(parsed, Some(event));
}

#[test]
fn test_event_loop_parse_event_resize() {
    let event_loop = EventLoop::new();
    let event = TerminalEvent::Resize(120, 40);

    let parsed = event_loop.parse_event(event.clone());
    assert_eq!(parsed, Some(event));
}

#[test]
fn test_event_loop_parse_event_mouse_filtered() {
    let event_loop = EventLoop::new();
    let event = TerminalEvent::Mouse(5, 5);

    // Mouse events should be filtered out
    let parsed = event_loop.parse_event(event);
    assert_eq!(parsed, None);
}

// ============================================================================
// PlaybackState tests
// ============================================================================

#[test]
fn test_playback_state_paused() {
    let state = PlaybackState::Paused;
    assert_eq!(state, PlaybackState::Paused);
}

#[test]
fn test_playback_state_playing() {
    let state = PlaybackState::Playing;
    assert_eq!(state, PlaybackState::Playing);
}

#[test]
fn test_playback_state_inequality() {
    assert_ne!(PlaybackState::Paused, PlaybackState::Playing);
}

// ============================================================================
// TimelineRenderer tests
// ============================================================================

#[test]
fn test_timeline_renderer_new() {
    let renderer = TimelineRenderer::new(100);
    assert_eq!(renderer.total_frames(), 100);
    assert_eq!(renderer.current_frame(), 0);
    assert_eq!(renderer.playback_state(), PlaybackState::Paused);
}

#[test]
fn test_timeline_renderer_set_current_frame() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.set_current_frame(50);
    assert_eq!(renderer.current_frame(), 50);
}

#[test]
fn test_timeline_renderer_set_current_frame_bounds() {
    let mut renderer = TimelineRenderer::new(10);
    renderer.set_current_frame(100); // Exceeds total
    assert_eq!(renderer.current_frame(), 9); // Should be clamped to last frame
}

#[test]
fn test_timeline_renderer_advance_frame_forward() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.set_current_frame(50);
    renderer.advance_frame(5);
    assert_eq!(renderer.current_frame(), 55);
}

#[test]
fn test_timeline_renderer_advance_frame_backward() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.set_current_frame(50);
    renderer.advance_frame(-10);
    assert_eq!(renderer.current_frame(), 40);
}

#[test]
fn test_timeline_renderer_advance_frame_clamp_start() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.set_current_frame(5);
    renderer.advance_frame(-20);
    assert_eq!(renderer.current_frame(), 0);
}

#[test]
fn test_timeline_renderer_toggle_playback() {
    let mut renderer = TimelineRenderer::new(100);
    assert_eq!(renderer.playback_state(), PlaybackState::Paused);

    renderer.toggle_playback();
    assert_eq!(renderer.playback_state(), PlaybackState::Playing);

    renderer.toggle_playback();
    assert_eq!(renderer.playback_state(), PlaybackState::Paused);
}

#[test]
fn test_timeline_renderer_jump_to_start() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.set_current_frame(75);
    renderer.jump_to_start();
    assert_eq!(renderer.current_frame(), 0);
}

#[test]
fn test_timeline_renderer_jump_to_end() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.jump_to_end();
    assert_eq!(renderer.current_frame(), 99);
}

#[test]
fn test_timeline_renderer_progress_percentage() {
    let mut renderer = TimelineRenderer::new(100);
    assert_eq!(renderer.progress_percentage(), 0.0);

    renderer.set_current_frame(50);
    assert!((renderer.progress_percentage() - 50.0).abs() < 0.01);

    renderer.set_current_frame(99);
    assert!((renderer.progress_percentage() - 99.0).abs() < 0.01);
}

#[test]
fn test_timeline_renderer_progress_percentage_empty() {
    let renderer = TimelineRenderer::new(0);
    assert_eq!(renderer.progress_percentage(), 0.0);
}

#[test]
fn test_timeline_renderer_frame_info() {
    let mut renderer = TimelineRenderer::new(100);
    renderer.set_current_frame(42);
    assert_eq!(renderer.frame_info(), "42/100");
}

#[test]
fn test_timeline_renderer_playback_controls_text() {
    let mut renderer = TimelineRenderer::new(100);
    assert!(renderer.playback_controls_text().contains("Play"));

    renderer.toggle_playback();
    assert!(renderer.playback_controls_text().contains("Pause"));
}

#[test]
fn test_timeline_renderer_keyboard_shortcuts() {
    let renderer = TimelineRenderer::new(100);
    let shortcuts = renderer.keyboard_shortcuts();
    assert!(shortcuts.contains("Prev"));
    assert!(shortcuts.contains("Next"));
    assert!(shortcuts.contains("Quit"));
}

// ============================================================================
// VariableInspectorView tests
// ============================================================================

#[test]
fn test_variable_inspector_new() {
    let view = VariableInspectorView::new();
    assert_eq!(view.variable_count(), 0);
    assert_eq!(view.scroll_offset(), 0);
}

#[test]
fn test_variable_inspector_default() {
    let view = VariableInspectorView::default();
    assert_eq!(view.variable_count(), 0);
}

#[test]
fn test_variable_inspector_from_variables() {
    let vars = vec![
        ("x".to_string(), "10".to_string()),
        ("y".to_string(), "20".to_string()),
    ];
    let view = VariableInspectorView::from_variables(vars);
    assert_eq!(view.variable_count(), 2);
}

#[test]
fn test_variable_inspector_add_variable() {
    let mut view = VariableInspectorView::new();
    view.add_variable("counter".to_string(), "42".to_string());
    assert_eq!(view.variable_count(), 1);
}

#[test]
fn test_variable_inspector_scroll_down() {
    let vars = vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
    ];
    let mut view = VariableInspectorView::from_variables(vars);

    assert_eq!(view.scroll_offset(), 0);
    view.scroll_down();
    assert_eq!(view.scroll_offset(), 1);
}

#[test]
fn test_variable_inspector_scroll_up() {
    let vars = vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
    ];
    let mut view = VariableInspectorView::from_variables(vars);
    view.set_scroll_offset(1);

    view.scroll_up();
    assert_eq!(view.scroll_offset(), 0);
}

#[test]
fn test_variable_inspector_scroll_up_at_top() {
    let mut view = VariableInspectorView::new();
    view.scroll_up();
    assert_eq!(view.scroll_offset(), 0);
}

#[test]
fn test_variable_inspector_viewport_height() {
    let mut view = VariableInspectorView::new();
    assert_eq!(view.viewport_height(), 10);

    view.set_viewport_height(20);
    assert_eq!(view.viewport_height(), 20);
}

#[test]
fn test_variable_inspector_page_down() {
    let vars: Vec<_> = (0..30)
        .map(|i| (format!("var{}", i), format!("{}", i)))
        .collect();
    let mut view = VariableInspectorView::from_variables(vars);
    view.set_viewport_height(10);

    view.page_down();
    assert_eq!(view.scroll_offset(), 10);
}

#[test]
fn test_variable_inspector_page_up() {
    let vars: Vec<_> = (0..30)
        .map(|i| (format!("var{}", i), format!("{}", i)))
        .collect();
    let mut view = VariableInspectorView::from_variables(vars);
    view.set_viewport_height(10);
    view.set_scroll_offset(20);

    view.page_up();
    assert_eq!(view.scroll_offset(), 10);
}

#[test]
fn test_variable_inspector_visible_range() {
    let vars: Vec<_> = (0..30)
        .map(|i| (format!("var{}", i), format!("{}", i)))
        .collect();
    let mut view = VariableInspectorView::from_variables(vars);
    view.set_viewport_height(10);

    let (start, end) = view.visible_range();
    assert_eq!(start, 0);
    assert_eq!(end, 10);
}

#[test]
fn test_variable_inspector_get_variable() {
    let vars = vec![("name".to_string(), "value".to_string())];
    let view = VariableInspectorView::from_variables(vars);

    let var = view.get_variable(0);
    assert!(var.is_some());
    let (name, value) = var.unwrap();
    assert_eq!(name, "name");
    assert_eq!(value, "value");
}

#[test]
fn test_variable_inspector_get_variable_out_of_bounds() {
    let view = VariableInspectorView::new();
    assert!(view.get_variable(0).is_none());
}

#[test]
fn test_variable_inspector_format_line() {
    let vars = vec![("x".to_string(), "42".to_string())];
    let view = VariableInspectorView::from_variables(vars);

    let line = view.format_line(0);
    assert_eq!(line, Some("x: 42".to_string()));
}

#[test]
fn test_variable_inspector_visible_lines() {
    let vars = vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
    ];
    let mut view = VariableInspectorView::from_variables(vars);
    view.set_viewport_height(5);

    let lines = view.visible_lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "a: 1");
    assert_eq!(lines[1], "b: 2");
}

// ============================================================================
// StackFrameNavigator tests
// ============================================================================

#[test]
fn test_stack_frame_navigator_new() {
    let nav = StackFrameNavigator::new();
    assert_eq!(nav.frame_count(), 0);
    assert_eq!(nav.selected_index(), 0);
}

#[test]
fn test_stack_frame_navigator_default() {
    let nav = StackFrameNavigator::default();
    assert_eq!(nav.frame_count(), 0);
}

#[test]
fn test_stack_frame_navigator_from_frames() {
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("helper".to_string(), "lib.rs".to_string(), 20),
    ];
    let nav = StackFrameNavigator::from_frames(frames);
    assert_eq!(nav.frame_count(), 2);
}

#[test]
fn test_stack_frame_navigator_add_frame() {
    let mut nav = StackFrameNavigator::new();
    nav.add_frame("test".to_string(), "test.rs".to_string(), 42);
    assert_eq!(nav.frame_count(), 1);
}

#[test]
fn test_stack_frame_navigator_select_next() {
    let frames = vec![
        ("a".to_string(), "a.rs".to_string(), 1),
        ("b".to_string(), "b.rs".to_string(), 2),
        ("c".to_string(), "c.rs".to_string(), 3),
    ];
    let mut nav = StackFrameNavigator::from_frames(frames);

    assert_eq!(nav.selected_index(), 0);
    nav.select_next();
    assert_eq!(nav.selected_index(), 1);
    nav.select_next();
    assert_eq!(nav.selected_index(), 2);
}

#[test]
fn test_stack_frame_navigator_select_next_at_end() {
    let frames = vec![("a".to_string(), "a.rs".to_string(), 1)];
    let mut nav = StackFrameNavigator::from_frames(frames);
    nav.select_next();
    nav.select_next(); // Should stay at last
    assert_eq!(nav.selected_index(), 0);
}

#[test]
fn test_stack_frame_navigator_select_previous() {
    let frames = vec![
        ("a".to_string(), "a.rs".to_string(), 1),
        ("b".to_string(), "b.rs".to_string(), 2),
    ];
    let mut nav = StackFrameNavigator::from_frames(frames);
    nav.set_selected_index(1);

    nav.select_previous();
    assert_eq!(nav.selected_index(), 0);
}

#[test]
fn test_stack_frame_navigator_select_previous_at_start() {
    let mut nav = StackFrameNavigator::new();
    nav.select_previous();
    assert_eq!(nav.selected_index(), 0);
}

#[test]
fn test_stack_frame_navigator_get_frame() {
    let frames = vec![("main".to_string(), "main.rs".to_string(), 42)];
    let nav = StackFrameNavigator::from_frames(frames);

    let frame = nav.get_frame(0);
    assert!(frame.is_some());
    let (func, file, line) = frame.unwrap();
    assert_eq!(func, "main");
    assert_eq!(file, "main.rs");
    assert_eq!(line, 42);
}

#[test]
fn test_stack_frame_navigator_get_selected_frame() {
    let frames = vec![
        ("first".to_string(), "f.rs".to_string(), 1),
        ("second".to_string(), "s.rs".to_string(), 2),
    ];
    let mut nav = StackFrameNavigator::from_frames(frames);
    nav.set_selected_index(1);

    let selected = nav.get_selected_frame();
    assert!(selected.is_some());
    let (func, _, _) = selected.unwrap();
    assert_eq!(func, "second");
}

#[test]
fn test_stack_frame_navigator_format_frame_line() {
    let frames = vec![("process".to_string(), "mod.rs".to_string(), 100)];
    let nav = StackFrameNavigator::from_frames(frames);

    let line = nav.format_frame_line(0);
    assert_eq!(line, Some("process @ mod.rs:100".to_string()));
}

#[test]
fn test_stack_frame_navigator_format_frame_with_marker() {
    let frames = vec![
        ("a".to_string(), "a.rs".to_string(), 1),
        ("b".to_string(), "b.rs".to_string(), 2),
    ];
    let nav = StackFrameNavigator::from_frames(frames);

    let selected_line = nav.format_frame_with_marker(0);
    assert!(selected_line.unwrap().contains("▶"));

    let unselected_line = nav.format_frame_with_marker(1);
    assert!(!unselected_line.unwrap().starts_with("▶"));
}

#[test]
fn test_stack_frame_navigator_is_frame_selected() {
    let frames = vec![
        ("a".to_string(), "a.rs".to_string(), 1),
        ("b".to_string(), "b.rs".to_string(), 2),
    ];
    let nav = StackFrameNavigator::from_frames(frames);

    assert!(nav.is_frame_selected(0));
    assert!(!nav.is_frame_selected(1));
}

#[test]
fn test_stack_frame_navigator_render_all_frames() {
    let frames = vec![
        ("a".to_string(), "a.rs".to_string(), 1),
        ("b".to_string(), "b.rs".to_string(), 2),
    ];
    let nav = StackFrameNavigator::from_frames(frames);

    let rendered = nav.render_all_frames();
    assert_eq!(rendered.len(), 2);
}

// ============================================================================
// TuiAction tests
// ============================================================================

#[test]
fn test_tui_action_description() {
    assert_eq!(TuiAction::NextFrame.description(), "Next frame");
    assert_eq!(TuiAction::PreviousFrame.description(), "Previous frame");
    assert_eq!(TuiAction::TogglePlayback.description(), "Play/Pause");
    assert_eq!(TuiAction::JumpToStart.description(), "Jump to start");
    assert_eq!(TuiAction::JumpToEnd.description(), "Jump to end");
    assert_eq!(TuiAction::Quit.description(), "Quit");
    assert_eq!(TuiAction::ScrollDown.description(), "Scroll down");
    assert_eq!(TuiAction::ScrollUp.description(), "Scroll up");
    assert_eq!(TuiAction::SelectNextFrame.description(), "Next stack frame");
    assert_eq!(
        TuiAction::SelectPreviousFrame.description(),
        "Previous stack frame"
    );
}

#[test]
fn test_tui_action_equality() {
    assert_eq!(TuiAction::Quit, TuiAction::Quit);
    assert_ne!(TuiAction::Quit, TuiAction::NextFrame);
}

// ============================================================================
// KeyboardHandler tests
// ============================================================================

#[test]
fn test_keyboard_handler_new() {
    let handler = KeyboardHandler::new();
    assert!(handler.has_default_bindings());
}

#[test]
fn test_keyboard_handler_default() {
    let handler = KeyboardHandler::default();
    assert!(handler.has_default_bindings());
}

#[test]
fn test_keyboard_handler_default_bindings() {
    let handler = KeyboardHandler::new();

    assert_eq!(
        handler.get_action(KeyCode::Right),
        Some(TuiAction::NextFrame)
    );
    assert_eq!(
        handler.get_action(KeyCode::Left),
        Some(TuiAction::PreviousFrame)
    );
    assert_eq!(
        handler.get_action(KeyCode::Char(' ')),
        Some(TuiAction::TogglePlayback)
    );
    assert_eq!(
        handler.get_action(KeyCode::Home),
        Some(TuiAction::JumpToStart)
    );
    assert_eq!(handler.get_action(KeyCode::End), Some(TuiAction::JumpToEnd));
    assert_eq!(
        handler.get_action(KeyCode::Char('q')),
        Some(TuiAction::Quit)
    );
}

#[test]
fn test_keyboard_handler_bind_key() {
    let mut handler = KeyboardHandler::new();
    handler.bind_key(KeyCode::Char('n'), TuiAction::NextFrame);

    assert_eq!(
        handler.get_action(KeyCode::Char('n')),
        Some(TuiAction::NextFrame)
    );
}

#[test]
fn test_keyboard_handler_unbind_key() {
    let mut handler = KeyboardHandler::new();
    handler.unbind_key(KeyCode::Right);

    assert_eq!(handler.get_action(KeyCode::Right), None);
}

#[test]
fn test_keyboard_handler_is_key_bound() {
    let handler = KeyboardHandler::new();

    assert!(handler.is_key_bound(KeyCode::Right));
    assert!(!handler.is_key_bound(KeyCode::Char('z')));
}

#[test]
fn test_keyboard_handler_handle_event_key() {
    let handler = KeyboardHandler::new();
    let event = TerminalEvent::Key(KeyCode::Right);

    let action = handler.handle_event(&event);
    assert_eq!(action, Some(TuiAction::NextFrame));
}

#[test]
fn test_keyboard_handler_handle_event_resize() {
    let handler = KeyboardHandler::new();
    let event = TerminalEvent::Resize(80, 24);

    let action = handler.handle_event(&event);
    assert_eq!(action, None);
}

#[test]
fn test_keyboard_handler_handle_event_unbound_key() {
    let handler = KeyboardHandler::new();
    let event = TerminalEvent::Key(KeyCode::Char('z'));

    let action = handler.handle_event(&event);
    assert_eq!(action, None);
}

#[test]
fn test_keyboard_handler_list_bindings() {
    let handler = KeyboardHandler::new();
    let bindings = handler.list_bindings();

    assert!(!bindings.is_empty());
    assert!(bindings.len() >= 6); // At least 6 default bindings
}

#[test]
fn test_keyboard_handler_generate_help_text() {
    let handler = KeyboardHandler::new();
    let help = handler.generate_help_text();

    assert!(!help.is_empty());
    assert!(help.contains("Next frame") || help.contains("Previous frame"));
}

#[test]
fn test_keyboard_handler_custom_binding() {
    let mut handler = KeyboardHandler::new();
    handler.bind_key(KeyCode::Char('j'), TuiAction::ScrollDown);
    handler.bind_key(KeyCode::Char('k'), TuiAction::ScrollUp);

    assert_eq!(
        handler.get_action(KeyCode::Char('j')),
        Some(TuiAction::ScrollDown)
    );
    assert_eq!(
        handler.get_action(KeyCode::Char('k')),
        Some(TuiAction::ScrollUp)
    );
}
