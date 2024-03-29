



use ratatui::{Frame, prelude::*, widgets::{Paragraph, Block, Borders}};

use crate::{app::App, button::Button};
use crate::button::footer_button;

pub fn draw_header(frame: &mut Frame, app: &mut App, area: Rect)
{

    let header = Layout::default()
    .direction(Direction::Horizontal)
    .margin(0)
    .constraints(
        [
            Constraint::Min(20),
            Constraint::Max(30)
        ].as_ref()
    )
    .split(area);

    let p = Paragraph::new(Span::styled(
        "  Planck 6502 emulator", Style::new().white().on_green().add_modifier(Modifier::BOLD)
    ))
    .alignment(Alignment::Left)
        .block(Block::default()
            .borders(Borders::NONE)
            .on_green()
        );

    frame.render_widget(p, header[0]);


    let sl = Paragraph::new(Span::styled(format!("Log level: {} {:03.2} MHz  ", app.log_level, app.speed/1000.0), 
    Style::new().white().on_green().add_modifier(Modifier::BOLD)))
    
    .alignment(Alignment::Right)
        .block(Block::default()
            //.title("")
            .borders(Borders::NONE)
            .on_green()
            
        );

    frame.render_widget(sl, header[1]);


}

pub fn draw_footer(f: &mut Frame, area: Rect, buttons: Vec<Button>)
{

    let block = Block::new()
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::LightBlue))
        ;

    f.render_widget(block, area);

    let constraints = Constraint::from_ratios(buttons.iter().map(|_| (1 as u32, buttons.len() as u32)).into_iter());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            constraints,
        )
        .split(area);

    for (i, button) in buttons.iter().enumerate() {
        let footer = Paragraph::new(footer_button(button.clone()))
        .block(Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::LightBlue))
        );
        f.render_widget(footer, chunks[i]);
    }
    
}