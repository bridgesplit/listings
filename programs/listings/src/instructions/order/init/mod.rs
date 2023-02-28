use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitOrderData {
    pub nonce: Pubkey,
    pub side: u8,
    pub price: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
