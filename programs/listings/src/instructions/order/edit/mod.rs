use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq)]
pub enum EditSide {
    Increase,
    Decrease,
}

impl EditSide {
    pub fn is_increase(self) -> bool {
        self == EditSide::Increase
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct EditOrderData {
    pub side: EditSide,
    pub price: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
