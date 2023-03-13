use std::fmt::{self, Debug, Formatter};

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
    /// mint of the nft held in the order
    pub nft_mint: Pubkey,
    /// reserved space for future changes
    reserve: [u8; 512],
}

impl Tracker {
    /// initialize a new order account
    pub fn init(&mut self, market: Pubkey, order: Pubkey, nft_mint: Pubkey) {
        self.version = TRACKER_VERSION;
        self.market = market;
        self.order = order;
        self.nft_mint = nft_mint;
    }
}

impl Debug for Tracker {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{{ version: {},
                market: {},
                order: {},
                nft_mint: {},
                reserve: {:?} }}",
            self.version, self.market, self.order, self.nft_mint, self.reserve
        )
    }
}
