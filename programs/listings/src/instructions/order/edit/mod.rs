use anchor_lang::prelude::*;
use num_enum::IntoPrimitive;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq, IntoPrimitive)]
#[repr(u8)]
/// What type of edit is being made
/// Increase: Increase the bid size
/// Decrease: Decrease the bid size
pub enum EditSide {
    Increase,
    Decrease,
}

impl EditSide {
    pub fn is_increase(side: u8) -> bool {
        side == <EditSide as Into<u8>>::into(EditSide::Increase)
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct EditOrderData {
    pub side: u8,
    pub size: u64,
    pub price: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
