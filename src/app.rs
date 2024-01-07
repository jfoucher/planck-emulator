use std::collections::{HashMap, VecDeque};
use std::ops::Add;
use std::{error, fs};
use std::fs::File;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::time::Duration;
use tui::widgets::ScrollbarState;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use crate::computer::{self, Computer, ComputerMessage};
use crate::ui::stateful_list::StatefulList;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Main,
    Help,
}


#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub enum Message {
    ButtonPressed(String),
}

pub struct InputState {
    pub mode: InputMode,
    pub value: String,
    pub cursor_position: u16,
}

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.

pub struct App {
    /// Is the application running?
    pub running: bool,
    pub current_tab: Tab,
    pub line: String,
    pub debug: VecDeque<String>,
    pub rx: Receiver<computer::ComputerMessage>,
    pub tx: Sender<computer::ControllerMessage>,

}


impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(rom_file: String) -> Self {
        let data = fs::read(rom_file).expect("could not read file");
        
        let (tx, rx) = mpsc::channel::<computer::ControllerMessage>();
        let (computer_tx, computer_rx) = mpsc::channel::<computer::ComputerMessage>();
        let computer_data = data.clone();
        let child = thread::spawn(move || {
            let mut computer = Computer::new(computer_tx, rx, computer_data);
            //computer.reset();

            loop {
                computer.step();
            }
        });

        Self {
            running: true,
            current_tab: Tab::Main,
            line: String::from(""),
            debug: VecDeque::new(),
            tx: tx,
            rx: computer_rx,
        }
    }


    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        while let Some(message) = self.rx.try_iter().next() {
            // Handle messages arriving from the UI.
            match message {
                ComputerMessage::Info(info) => {
                    self.debug.push_back(info);
                    if self.debug.len() > 30 {
                        self.debug.pop_front();
                    }
                }
                _ => {}
            };
        }
    }

    pub fn init(&mut self) {
        
    
    }


    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
