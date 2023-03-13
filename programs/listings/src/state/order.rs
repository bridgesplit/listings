use std::fmt::{self, Debug, Formatter};

use anchor_lang::prelude::*;
use num_enum::IntoPrimitive;

use crate::instructions::order::EditSide;

pub const ORDER_VERSION: u8 = 1;

#[account()]
/// order account - each listing has one order account
pub struct Order {
    /// order account version
    pub version: u8,
    /// nonce for uniqueness
    pub nonce: Pubkey,
    /// market to which the order belongs to, must be init'd
    pub market: Pubkey,
    /// owner of the order account
    pub owner: Pubkey,
    /// bidding wallet of the owner
    pub wallet: Pubkey,
    /// type of order - buy/sell
    pub side: u8,
    /// number of bids order is making
    pub size: u64,
    /// bid amount in lamports
    pub price: u64,
    /// order state - ready/partial/closed
    pub state: u8,
    /// order account creation time
    pub init_time: i64,
    /// last time the order was edited
    pub last_edit_time: i64,
    /// reserved space for future changes
    reserve: [u8; 512],
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, IntoPrimitive)]
#[repr(u8)]
/// bid type for order
pub enum OrderSide {
    /// bid for buying NFT
    Buy,
    /// bid for selling NFT
    Sell,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq, IntoPrimitive)]
#[repr(u8)]
/// state of the order
pub enum OrderState {
    /// order account has been created and ready to be filled
    Ready,
    /// some of the bids have been filled, used only in UI to show some orders have been filled
    Partial,
    /// all bids have been filled and the order account is now closed
    Closed,
}

impl Order {
    /// initialize a new order account
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        market: Pubkey,
        owner: Pubkey,
        wallet: Pubkey,
        nonce: Pubkey,
        time: i64,
        side: u8,
        size: u64,
        price: u64,
        state: u8,
    ) {
        self.version = ORDER_VERSION;
        self.market = market;
        self.nonce = nonce;
        self.owner = owner;
        self.wallet = wallet;
        self.side = side;
        self.size = size;
        self.price = price;
        self.state = state;
        self.init_time = time;
        self.last_edit_time = time;
    }

    /// check if a buy order being filled has a higher price than the sell order
    pub fn spill_over(buy_price: u64, sell_price: u64) -> bool {
        buy_price > sell_price
    }

    /// edit an order account
    /// if size is 0, order is closed
    /// any size change is considered partial
    pub fn edit(&mut self, price: u64, edit_size: u64, edit_side: u8, time: i64) {
        let size = Self::edit_size(self.size, edit_size, edit_side);
        self.size = size;
        self.price = price;
        self.last_edit_time = time;
        // mark order as partial if size is less than original size and closed if size is 0
        if size != 0 {
            self.state = OrderState::Partial.into();
        } else {
            self.state = OrderState::Closed.into();
        }
    }

    /// fetch the new size of the order account after an edit
    /// edit_size is the number of bids to be added or removed
    pub fn edit_size(current_size: u64, edit_size: u64, edit_side: u8) -> u64 {
        let mut size = current_size;
        if EditSide::is_increase(edit_side) {
            size += edit_size;
        } else {
            size -= edit_size;
        }
        size
    }

    /// validate if an increase edit can be made to the order account
    pub fn validate_edit_side(edit_side: u8, state: u8) -> bool {
        if !Self::is_active(state) && EditSide::is_increase(edit_side) {
            return false;
        }
        true
    }

    /// return true if the order is active
    pub fn is_active(state: u8) -> bool {
        state != <OrderState as Into<u8>>::into(OrderState::Closed)
    }
}

impl Debug for Order {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{{ version: {},
            nonce: {},
            market: {},
            owner: {},
            wallet: {},
            side: {},
            size: {},
            price: {},
            state: {},
            init_time: {},
            last_edit_time: {} }}",
            self.version,
            self.nonce,
            self.market,
            self.owner,
            self.wallet,
            self.side,
            self.size,
            self.price,
            self.state,
            self.init_time,
            self.last_edit_time,
        )
    }
}
