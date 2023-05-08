use anchor_lang::prelude::*;
use bridgesplit_program_utils::anchor_lang;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitOrderData {
    pub nonce: Pubkey,
    pub price: u64,
    pub size: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
