use anchor_lang::prelude::*;

pub const TRACKER_VERSION: u8 = 1;

#[account()]
/// tracker account - used to track nfts held
pub struct Tracker {
    /// order account version
    pub version: u8,
    /// market account
    pub market: Pubkey,
    /// order account
    pub order: Pubkey,
    /// owner account
    pub owner: Pubkey,
    /// mint of the nft held in the order
    pub nft_mint: Pubkey,
    /// reserved space for future changes
    reserve: [u8; 512],
}

impl Tracker {
    /// initialize a new order account
    pub fn init(&mut self, market: Pubkey, order: Pubkey, owner: Pubkey, nft_mint: Pubkey) {
        self.version = TRACKER_VERSION;
        self.market = market;
        self.order = order;
        self.owner = owner;
        self.nft_mint = nft_mint;
    }
}
