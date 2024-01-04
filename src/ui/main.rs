use std::time::SystemTime;

use chrono::{DateTime, Local};

use tui::{Frame, prelude::*, widgets::{Paragraph, Block, Borders, Wrap, ListItem, List, Table, Row, Padding}};


use crate::{app::{App, InputMode}, button::{Button, action_button}};
use crate::ui::header;
use super::modal;

const MAIN_HELP_TEXT: &str = "
This is the Planck 6502 emulator. Enjoy
";



pub fn draw_main_help<'a, B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(0)
    .constraints(
        [
            Constraint::Min(6),     // Help text
            Constraint::Max(1),     // Tab Footer
        ]
        .as_ref(),
    )
    .split(area);
    let t_title = Span::styled(format!("{: ^width$}", "Main help", width = f.size().width as usize), Style::default().add_modifier(Modifier::BOLD).fg(Color::White).bg(Color::Magenta));
    let p = Paragraph::new(MAIN_HELP_TEXT)
        .block(Block::default()
            .title(t_title)
            .title_alignment(Alignment::Center)
            .borders(Borders::NONE)
        )
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[0]);    

    let buttons = vec![
        Button::new("Close".to_string(), Some("1".to_string())),
        Button::new("Quit".to_string(), Some("2".to_string())),
    ];
    header::draw_footer(f, chunks[1], buttons);

}


pub fn draw_main_tab<'a, B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Min(6),     // Job history
                Constraint::Length(1),     // title
                Constraint::Length(10),
                Constraint::Max(1),     // Tab Footer
            ]
            .as_ref(),
        )
        .split(area);

    let p = Paragraph::new(app.debug.clone())
        .block(Block::default()
            .borders(Borders::NONE)
        )
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[0]);    
}
