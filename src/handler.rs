use crate::{app::{App, AppResult, Tab}, computer};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            match app.current_tab {
                Tab::Main => {
                    let _ = app.tx.send(computer::ControllerMessage::SendChar(0x1B as char));
                },
                _ => {},
            };
        }
        
        // Counter handlers
        KeyCode::F(1) => {
            app.current_tab = match app.current_tab {
                Tab::Main => Tab::Help,
                Tab::Memory => Tab::Help,
                Tab::Help => Tab::Main,
            }
        }
        KeyCode::F(2) => {
            app.quit();
        }

        KeyCode::F(3) => {
            app.current_tab = match app.current_tab {
                Tab::Main => Tab::Memory,
                Tab::Memory => Tab::Main,
                Tab::Help => Tab::Main,
            }
        }

        KeyCode::F(4) => {
            match app.current_tab {
                Tab::Memory | Tab::Main => {
                    let _ = app.tx.send(crate::computer::ControllerMessage::Reset);
                },
                _ => {}
            }
        }

        KeyCode::F(5) => {
            match app.current_tab {
                Tab::Main => {
                    app.log_level = app.log_level.saturating_sub(1);
                    let _ = app.tx.send(crate::computer::ControllerMessage::SetDebug(app.log_level));
                },
                _ => {}
            }
        }

        KeyCode::F(6) => {
            match app.current_tab {
                Tab::Main => {
                    app.log_level = app.log_level.saturating_add(1);
                    let _ = app.tx.send(crate::computer::ControllerMessage::SetDebug(app.log_level));
                },
                _ => {}
            }
        }


        KeyCode::F(7) => {
            match app.current_tab {
                Tab::Main => {
                    let _ = app.tx.send(crate::computer::ControllerMessage::TogglePause);
                },
                _ => {}
            }
        }
        
        KeyCode::Enter => {
            match app.current_tab {
                Tab::Main => {
                    // Send data to computer
                    let _ = app.tx.send(computer::ControllerMessage::SendChar(0x0D as char));
                },
                _ => {},
            }
        }
        
        KeyCode::Up => {
            match app.current_tab {
                Tab::Memory => {
                    app.memory_scroll = app.memory_scroll.saturating_sub(1);
                    app.memory_scroll_state = app.memory_scroll_state.position(app.memory_scroll);
                },
                Tab::Main => {
                    app.output_scroll = app.output_scroll.saturating_sub(1);
                    app.output_scroll_state = app.output_scroll_state.position(app.output_scroll);
                },
                _ => {},
            }
        }
        
        KeyCode::Down => {
            match app.current_tab {
                Tab::Memory => {
                    app.memory_scroll = app.memory_scroll.saturating_add(1);
                    app.memory_scroll_state = app.memory_scroll_state.position(app.memory_scroll);
                },
                Tab::Main => {
                    app.output_scroll = app.output_scroll.saturating_add(1);
                    app.output_scroll_state = app.output_scroll_state.position(app.output_scroll);
                },
                _ => {},
            }
        }
        
        KeyCode::PageUp => {
            match app.current_tab {
                Tab::Memory => {
                    app.memory_scroll = app.memory_scroll.saturating_sub(16);
                    app.memory_scroll_state = app.memory_scroll_state.position(app.memory_scroll);
                },
                Tab::Main => {
                    app.output_scroll = app.output_scroll.saturating_sub(16);
                    app.output_scroll_state = app.output_scroll_state.position(app.output_scroll);
                },
                _ => {},
            }
        }
        
        KeyCode::PageDown => {
            match app.current_tab {
                Tab::Memory => {
                    app.memory_scroll = app.memory_scroll.saturating_add(16);
                    app.memory_scroll_state = app.memory_scroll_state.position(app.memory_scroll);
                },
                Tab::Main => {
                    app.output_scroll = app.output_scroll.saturating_add(16);
                    app.output_scroll_state = app.output_scroll_state.position(app.output_scroll);
                },
                _ => {},
            }
        }

        KeyCode::Char(c) => {
            if c == 'c' && key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
            match app.current_tab {
                Tab::Main => {
                    let _ = app.tx.send(computer::ControllerMessage::SendChar(c));
                    return Ok(()) ;
                    //app.cursor_position = app.cursor_position.saturating_add(1);
                },
                _ => {
                    
                }
            }
        },
        KeyCode::Backspace => {
            match app.current_tab {
                Tab::Main => {
                    let _ = app.tx.send(computer::ControllerMessage::SendChar(0x08 as char));
                    // app.cursor_position = app.cursor_position.saturating_sub(1);
                },
                _ => {},
            }
        },
        
        _ => {}
    }
    Ok(())
}
