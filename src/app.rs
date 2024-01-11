use std::collections::VecDeque;
use std::{error, fs};
use ratatui::widgets::ScrollbarState;
use std::thread::{self};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use crate::computer::{self, Computer, ComputerMessage, Processor};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Main,
    Memory,
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
    pub input: String,
    pub output: VecDeque<String>,
    pub debug: VecDeque<String>,
    pub rx: Receiver<computer::ComputerMessage>,
    pub tx: Sender<computer::ControllerMessage>,
    pub memory_scroll_state: ScrollbarState,
    pub memory_scroll: usize,
    pub mem: Vec<u8>,
    pub processor: Processor,
}


impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(rom_file: String, cf_file: Option<String>) -> Self {
        let data = fs::read(rom_file).expect("could not read file");

        let disk_data = match cf_file {
            Some(d) => fs::read(d).expect("could not read file"),
            None => vec![],
        };
        let (tx, rx) = mpsc::channel::<computer::ControllerMessage>();
        let (computer_tx, computer_rx) = mpsc::channel::<computer::ComputerMessage>();
        let computer_data = data.clone();
        let _ = thread::spawn(move || {
            let mut computer = Computer::new(computer_tx, rx, computer_data, disk_data);
            computer.reset();

            loop {
                computer.step();
            }
        });

        let mut output = VecDeque::new();
        output.push_back(String::from(""));

        Self {
            running: true,
            current_tab: Tab::Main,
            input: String::from(""),
            output,
            debug: VecDeque::new(),
            tx,
            rx: computer_rx,
            memory_scroll_state: ScrollbarState::default(),
            memory_scroll: 0,
            mem: vec![],
            processor: Processor {
                flags: 0b00110000,
                acc: 0,
                rx: 0,
                ry: 0,
                pc: 0x400,
                sp: 0,
                clock: 0,
                inst: 0xea,
            }
        }
    }


    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if self.current_tab == Tab::Memory {
            let _ = self.tx.send(computer::ControllerMessage::GetMemory);
            let _ = self.tx.send(computer::ControllerMessage::GetProc);
        }
        
        while let Some(message) = self.rx.try_iter().next() {
            // Handle messages arriving from the UI.
            match message {
                ComputerMessage::Info(info) => {
                    self.debug.push_back(info);
                    if self.debug.len() > 20 {
                        self.debug.pop_front();
                    }
                }

                ComputerMessage::Memory(mem) => {
                    self.mem = mem;
                }

                ComputerMessage::Processor(proc) => {
                    self.processor = proc;
                }
                ComputerMessage::Output(val) => {
                    if val == 0x0D {
                        self.output.push_back(String::from(""));
                        if self.output.len() > 20 {
                            self.output.pop_front();
                        }
                    } else {
                        if let Some(mut l) = self.output.pop_back() {
                            l.push(val as char);
                            self.output.push_back(l);
                        }
                    }
                }
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
