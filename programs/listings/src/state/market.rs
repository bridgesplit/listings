use anchor_lang::prelude::*;
use num_enum::IntoPrimitive;

pub const MARKET_VERSION: u8 = 1;

#[account()]
pub struct Market {
    /// market account version
    pub version: u8,
    /// mint of the index to which the NFTs belong to
    pub pool_mint: Pubkey,
    /// initializer of the market - can edit and close the market
    pub initializer: Pubkey,
    /// state representing the market - open/closed
    pub state: u8,
    /// reserved space for future changes
    pub reserve: [u8; 512],
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum MarketState {
    /// market is open and can be used to create orders
    Open,
    /// market is closed and cannot be used to create orders
    Closed,
}

impl Market {
    /// initialize a new market
    pub fn init(&mut self, pool_mint: Pubkey, owner: Pubkey) {
        self.version = MARKET_VERSION;
        self.pool_mint = pool_mint;
        self.initializer = owner;
        self.state = MarketState::Open.into();
    }

    /// return true if the market is active
    pub fn is_active(state: u8) -> bool {
        state != <MarketState as Into<u8>>::into(MarketState::Closed)
    }
}
