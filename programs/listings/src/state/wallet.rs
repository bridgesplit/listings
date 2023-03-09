use anchor_lang::prelude::*;

use crate::instructions::order::EditSide;

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
    /// number of active bids the wallet is currently holding
    pub active_bids: u64,
    /// reserved space for future changes
    reserve: [u8; 512],
}

impl Wallet {
    /// initialize a new order account
    pub fn init(&mut self, owner: Pubkey, amount: u64) {
        self.version = WALLET_VERSION;
        self.owner = owner;
        self.balance = amount;
        self.active_bids = 0;
    }

    /// edit an order account
    /// if size is 0, order is closed
    /// any size change is considered partial
    pub fn edit_balance(&mut self, amount: u64, edit_side: u8) {
        if EditSide::is_increase(edit_side) {
            self.balance += amount;
        } else {
            self.balance -= amount;
        }
    }

    pub fn edit_active_bids(&mut self, size: u64, edit_side: u8) {
        if EditSide::is_increase(edit_side) {
            self.active_bids += size;
        } else {
            self.active_bids -= size;
        }
    }

    pub fn validate(balance: u64, amount: u64, edit_side: u8) -> bool {
        if EditSide::is_increase(edit_side) {
            if balance >= amount {
                return true;
            }
            return false;
        }
        true
    }
}
