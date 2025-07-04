use crate::computer::card::Card;


#[derive(Clone, Debug)]
pub struct Empty {

}

impl Card for Empty {

    fn get_interrupt(&mut self) -> bool {
        false
    }
    fn tick(&mut self) {
    }

    fn read(&mut self, _reg: u16) -> u8 {
        0
    }

    fn write(&mut self, _reg: u16, _val: u8) {
    }
}