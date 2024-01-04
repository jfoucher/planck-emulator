

use tui::{Frame, prelude::*, widgets::{Paragraph, Block, Borders}};

use crate::{app::App, button::Button};
use crate::button::footer_button;

pub fn draw_header<'a, B>(frame: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
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

    let p = Paragraph::new("Planck emulator")
        .block(Block::default()
            .borders(Borders::NONE)
        );

    frame.render_widget(p, header[0]);


    let sl = Paragraph::new("Another")
        .block(Block::default()
            //.title("")
            .borders(Borders::NONE)
        );

    frame.render_widget(sl, header[1]);


}

pub fn draw_footer<'a, B>(f: &mut Frame<B>, area: Rect, buttons: Vec<Button>)
where
    B: Backend,
{

    let block = Block::new()
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::LightBlue))
        ;

    f.render_widget(block, area);

    let constraints: Vec<Constraint> = buttons.iter().map(|_| Constraint::Ratio(1, buttons.len() as u32)).collect();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            constraints.as_ref(),
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