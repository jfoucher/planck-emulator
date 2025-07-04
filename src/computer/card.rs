use core::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardType {
    CF,
    Serial,
    IO,
    Ram,
    None,
}


pub struct CardData<'a> {
    pub card_type: CardType,
    pub value: Box<dyn Card + 'a>,
}

impl Debug for CardData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardData").field("card_type", &self.card_type).finish()
    }
}

pub trait Card {
    fn get_interrupt(&mut self) -> bool;
    fn tick(&mut self);
    fn read(&mut self, reg: u16) -> u8;
    fn write(&mut self, reg: u16, val: u8);
}