use std::time::SystemTime;

use chrono::{DateTime, Local};

use itertools::Itertools;
use ratatui::{layout::Constraint::*, prelude::*, widgets::*};

use crate::{app::{App, InputMode}, button::{Button, action_button}};
use crate::ui::header;
use super::modal;


pub fn draw_main_tab(f: &mut Frame, app: &mut App, area: Rect)
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Min(20),
                Constraint::Max(1),     // Tab Footer
            ]
            .as_ref(),
        )
        .split(area);

    let sides = Layout::default()
    .direction(Direction::Horizontal)
    .margin(0)
    .constraints(
        [
            Constraint::Max(55),
            Constraint::Min(18),    
            Constraint::Min(0)
        ]
        .as_ref(),
    ).split(chunks[0]);

    let ch = app.mem.chunks(16);



    let mut hex: Vec<Line> = ch.map(|c| c.as_ref().iter()).enumerate().map(|(i, x)| {
        return Line::from(format!("{:04X} {} ", i*16, x.map(|n| format!("{:02X}", n)).join(" ") ))
    }).collect();

    app.memory_scroll_state = app.memory_scroll_state.content_length(hex.len());

    if app.memory_scroll > hex.len() {
        app.memory_scroll = hex.len();
    }
    if hex.len() >= app.memory_scroll {
        hex.drain(0..app.memory_scroll);
    }
    
    // let hex = app.mem.iter().map(|&x| format!("{:X}", x)).join(" ");

    let p = Paragraph::new(hex)
        .block(Block::default()
        .title("Memory Hex").title_alignment(Alignment::Center)
            .borders(Borders::NONE)
        )
        
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, sides[0]);   

    let txt = vec![
        Line::from(format!("PC: {:04X}", app.processor.pc)),
        Line::from(format!("A: {:02X}", app.processor.acc)),
        Line::from(format!("X: {:02X}", app.processor.rx)),
        Line::from(format!("Y: {:02X}", app.processor.ry)),
    ];

    let p = Paragraph::new(txt)
        .block(Block::default()
        .title("Processor")
            .borders(Borders::NONE)
        )
        
        .wrap(Wrap { trim: false })
        ;
    f.render_widget(p, sides[2]);   

    let mut ascii: Vec<Line> = app.mem.chunks(16).map(|c| c.as_ref().iter()).enumerate().map(|(i, x)| {
        // println!("{:?}", x);
         return Line::from(x.map(|&n| {
            if n > 0x20 && n < 0x7F {
                return format!("{}", n as char);
            }
            return String::from(".");
         }).join("") )
     }).collect();

    if ascii.len() >= app.memory_scroll {
        ascii.drain(0..app.memory_scroll);
    }

    let p = Paragraph::new(ascii)
    .block(Block::default()
    .title("ASCII").title_alignment(Alignment::Center)
        .borders(Borders::NONE)
    )
    
    .wrap(Wrap { trim: false })
    ;
    f.render_widget(p, sides[1]);    

    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        sides[1],
        &mut app.memory_scroll_state,
    );


    let buttons = vec![
        
        Button::new("Quit".to_string(), Some("2".to_string())),
        Button::new("Main".to_string(), Some("3".to_string())),
        Button::new("Reset".to_string(), Some("4".to_string())),
    ];

    header::draw_footer(f, chunks[1], buttons); 
}
