use anchor_lang::prelude::*;

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
    /// type of order - buy/sell
    pub side: OrderSide,
    /// number of bids order is making
    pub size: u64,
    /// bid amount in lamports
    pub price: u64,
    /// order state - ready/partial/closed
    pub state: OrderState,
    /// reserved space for future changes
    reserve: [u8; 512],
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
/// bid type for order
pub enum OrderSide {
    /// bid for buying NFT
    Buy,
    /// bid for selling NFT
    Sell,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq)]
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
    pub fn init(
        &mut self,
        market: Pubkey,
        owner: Pubkey,
        side: OrderSide,
        size: u64,
        price: u64,
        state: OrderState,
    ) {
        self.version = ORDER_VERSION;
        self.market = market;
        self.owner = owner;
        self.side = side;
        self.size = size;
        self.price = price;
        self.state = state;
    }

    /// check if a buy order being filled has a higher price than the sell order
    pub fn spill_over(buy_price: u64, sell_price: u64) -> bool {
        buy_price > sell_price
    }

    /// edit an order account
    /// if size is 0, order is closed
    /// any size change is considered partial
    pub fn edit(&mut self, price: u64, edit_side: EditSide) {
        let size = Self::edit_size(self.size, edit_side);
        self.size = size;
        self.price = price;
        // mark order as partial if size is less than original size and closed if size is 0
        if size != 0 {
            self.state = OrderState::Partial;
        } else {
            self.state = OrderState::Closed;
        }
    }

    /// fetch the new size of the order account after an edit
    pub fn edit_size(current_size: u64, edit_side: EditSide) -> u64 {
        let mut size = current_size;
        if edit_side.is_increase() {
            size += 1;
        } else {
            size -= 1;
        }
        size
    }

    /// validate if an increase edit can be made to the order account
    pub fn validate_edit_side(edit_side: EditSide, state: OrderState) -> bool {
        if !Self::is_active(state) && edit_side.is_increase() {
            return false;
        }
        true
    }

    /// return true if the order is active
    pub fn is_active(state: OrderState) -> bool {
        state != OrderState::Closed
    }
}
