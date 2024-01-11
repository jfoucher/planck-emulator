
use itertools::Itertools;
use ratatui::{Frame, prelude::*, widgets::{Paragraph, Block, Borders, Wrap}};


use crate::{app::App, button::Button};
use crate::ui::header;

const MAIN_HELP_TEXT: &str = "
This is the Planck 6502 emulator. Enjoy
";



pub fn draw_main_help(f: &mut Frame, _: &mut App, area: Rect)
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
        Button::new("Memory".to_string(), Some("3".to_string())),
    ];
    header::draw_footer(f, chunks[1], buttons);

}


pub fn draw_main_tab(f: &mut Frame, app: &mut App, area: Rect)
{
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(20),     // debug output
                Constraint::Max(1),     // Tab Footer
                Constraint::Length(20),
                Constraint::Max(1),     // Tab Footer
            ]
            .as_ref(),
        )
        .split(area);

    let p = Paragraph::new(app.debug.iter().join("\n"))
        .block(Block::default()
            .borders(Borders::NONE)
        )
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[0]);    

    let p = Paragraph::new("OUTPUT")
        .block(Block::default()
            .borders(Borders::NONE)
        )
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[1]);       

    let p = Paragraph::new(app.output.iter().join("\n"))
        .block(Block::default()
            .borders(Borders::NONE)
        )
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[2]);   

    let buttons = vec![
        Button::new("Help".to_string(), Some("1".to_string())),
        Button::new("Quit".to_string(), Some("2".to_string())),
        Button::new("Memory".to_string(), Some("3".to_string())),

        Button::new("Reset".to_string(), Some("4".to_string())),
    ];

    header::draw_footer(f, chunks[3], buttons); 
}
