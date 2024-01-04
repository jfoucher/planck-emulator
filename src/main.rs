

use plu::app::{App, AppResult};
use plu::event::{Event, EventHandler};
use plu::handler::handle_key_events;
use plu::tui::Tui;


use std::{io, env};

use tui::backend::CrosstermBackend;
use tui::Terminal;


fn main() -> AppResult<()> {
    let args: Vec<String> = env::args().collect();
    // Create an application.
    if args.len() < 2 {
        println!("Usage: plu <rom.bin>");
        return Ok(());
    }

    let mut app = App::new(args[1].clone());

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(100);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    app.init();
    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    Ok(())
}
