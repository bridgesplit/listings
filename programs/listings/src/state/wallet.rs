use anchor_lang::prelude::*;

pub const WALLET_VERSION: u8 = 1;

#[account()]
/// wallet account - bidding authority and funds holder
pub struct Wallet {
    /// order account version
    pub version: u8,
    /// Owner of the wallet
    pub owner: Pubkey,
    /// wallet balance
    pub balance: u64,
    /// reserved space for future changes
    reserve: [u8; 512],
}

impl Wallet {
    /// initialize a new order account
    pub fn init(&mut self, owner: Pubkey, amount: u64) {
        self.version = WALLET_VERSION;
        self.owner = owner;
        self.balance = amount;
    }

    pub fn edit_balance(&mut self, is_increase: bool, amount: u64) {
        if is_increase {
            self.balance += amount;
        } else {
            self.balance -= amount;
        }
    }
}
