use crate::{app::{App, AppResult, Tab, InputMode}};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            match app.current_tab {
                Tab::Main => {
                    app.quit();
                },
                Tab::Help => {
                    app.quit();
                },
                _ => {},
            };
        }
        
        // Counter handlers
        KeyCode::F(1) => {
            app.current_tab = match app.current_tab {
                Tab::Main => Tab::Help,
                Tab::Help => Tab::Main,
            }
        }
        KeyCode::F(2) => {
            app.quit();
        }
        
        KeyCode::Enter => {
            match app.current_tab {
                Tab::Main => {
                    // Send data to computer
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
                    app.input.push(c)
                },
                _ => {
                    
                }
            }
        },
        KeyCode::Backspace => {
            match app.current_tab {
                Tab::Main => {
                    app.input.pop();
                },
                _ => {},
            }
        },
        
        _ => {}
    }
    Ok(())
}
