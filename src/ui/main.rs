
use itertools::Itertools;
use ratatui::{Frame, prelude::*, widgets::{Paragraph, Block, Borders, Wrap, Scrollbar, ScrollbarOrientation}};


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
                Constraint::Max(20),     // debug output
                Constraint::Min(22),
                Constraint::Length(1),     // Tab Footer
            ]
            .as_ref(),
        )
        .split(area);

    let p = Paragraph::new(app.debug.iter().join("\n"))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Debug ")
            .title_alignment(Alignment::Center)
        )
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[0]);    
  
    let mut output: Vec<Line> = app.output.iter().map(|l| Line::from(l.as_str())).collect();
    app.output_scroll_state = app.output_scroll_state.content_length(output.len());

    if output.len() < app.output_scroll {
        app.output_scroll = output.len();
    }

    if output.len() > 1 && output.len() - 1 < app.output_scroll {
        app.output_scroll = output.len() - 1;
    }

    let ch = chunks[1].height as usize - 2;
    if output.len() > ch && output.len() - ch < app.output_scroll {
        app.output_scroll = output.len() - ch;
    }



    if output.len() >= app.output_scroll {
        output.drain(0..app.output_scroll);
    }


    let p = Paragraph::new(output)

    .style(Style::default().fg(Color::Yellow))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Output ")
            .title_alignment(Alignment::Center)
        )
        
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, chunks[1]);   

    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
            chunks[1],
        &mut app.output_scroll_state,
    );

    let mut cy = chunks[1].y + app.output.len() as u16 - app.output_scroll as u16;
    // if cy > chunks[1].y+chunks[1].height - 2 {
    //     cy = chunks[1].y+chunks[1].height - 2;
    // }
    if cy < chunks[1].y+chunks[1].height - 1 {
        f.set_cursor(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            chunks[1].x + app.cursor_position as u16 + 1,
            // Move one line down, from the border to the input line
            cy,
        );
    }



    let buttons = vec![
        Button::new("Help".to_string(), Some("1".to_string())),
        Button::new("Quit".to_string(), Some("2".to_string())),
        Button::new("Memory".to_string(), Some("3".to_string())),

        Button::new("Reset".to_string(), Some("4".to_string())),
    ];

    header::draw_footer(f, chunks[2], buttons); 
}
