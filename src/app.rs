use std::collections::VecDeque;
use std::time::SystemTime;
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
    pub output: VecDeque<String>,
    pub debug: VecDeque<String>,
    pub rx: Receiver<computer::ComputerMessage>,
    pub tx: Sender<computer::ControllerMessage>,
    pub memory_scroll_state: ScrollbarState,
    pub memory_scroll: usize,
    pub output_scroll_state: ScrollbarState,
    pub output_scroll: usize,
    pub mem: Vec<u8>,
    pub processor: Processor,
    pub cursor_position: usize,
    pub tick_time: SystemTime,
    pub old_clock: u128,
    pub speed: f64,
    pub log_level: u8,
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
            output,
            debug: VecDeque::new(),
            tx,
            rx: computer_rx,
            memory_scroll_state: ScrollbarState::default(),
            memory_scroll: 0,
            output_scroll_state: ScrollbarState::default(),
            output_scroll: 0,
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
                irq: false,
            },
            cursor_position: 0,
            tick_time: SystemTime::now(),
            old_clock: 0,
            speed: 0.0,
            log_level: 0,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        match self.current_tab {
            Tab::Main => {
                let _ = self.tx.send(computer::ControllerMessage::GetProc);
            },
            Tab::Memory => {
                let _ = self.tx.send(computer::ControllerMessage::GetMemory);
                let _ = self.tx.send(computer::ControllerMessage::GetProc);
            },
            Tab::Help => { },
        }



        
        while let Some(message) = self.rx.try_iter().next() {
            // Handle messages arriving from the UI.
            match message {
                ComputerMessage::Info(info) => {
                    self.debug.push_back(info.clone());
                    log::info!("{}", info);
                    if self.debug.len() > 10 {
                        self.debug.pop_front();
                    }
                }

                ComputerMessage::Memory(mem) => {
                    self.mem = mem;
                }

                ComputerMessage::Processor(proc) => {
                    self.processor = proc;


                    let t = SystemTime::now();

                    let a = match t.duration_since(self.tick_time) {
                        Ok(t) => t.as_millis(),
                        Err(_) => 20,
                    };

                    if a > 1000 {
                        self.speed = (self.processor.clock as f64 - self.old_clock as f64) / a as f64;

                        self.tick_time = SystemTime::now();
                        self.old_clock = self.processor.clock;
                    }
                
                    
                }
                ComputerMessage::Output(val) => {
                    if val == 0x0D || val == 0x0A {
                        self.cursor_position = 0;
                        self.output.push_back(String::from(""));
                        if self.output.len() > 22 {
                            self.output_scroll = self.output.len() - 20;
                        }
                    }else if val == 0x08 {
                        if let Some(mut l) = self.output.pop_back() {
                            l.pop();
                            self.output.push_back(l);
                            self.cursor_position = self.cursor_position.saturating_sub(1);
                        }
                    } else {
                        if let Some(mut l) = self.output.pop_back() {
                            l.push(val as char);
                            self.output.push_back(l);
                            self.cursor_position = self.cursor_position.saturating_add(1);
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
