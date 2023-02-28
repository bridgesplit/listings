use crate::state::OrderSide;
use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitOrderData {
    pub nonce: Pubkey,
    pub side: OrderSide,
    pub price: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
