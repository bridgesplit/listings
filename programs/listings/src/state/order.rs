use anchor_lang::prelude::*;
use num_enum::IntoPrimitive;

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
    /// always for 1 for sell
    pub size: u64,
    /// bid amount in lamports
    pub price: u64,
    /// order state - ready/partial/closed
    pub state: u8,
    /// order account creation time
    pub init_time: i64,
    /// last time the order was edited
    pub last_edit_time: i64,
    /// nft mint in case order is a sell order
    pub nft_mint: Pubkey,
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
        nft_mint: Pubkey,
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
        self.nft_mint = nft_mint;
        self.side = side;
        self.size = size;
        self.price = price;
        self.state = state;
        self.init_time = time;
        self.last_edit_time = time;
    }

    /// edit a buy order account
    /// if size is 0, order is closed
    /// any size change is considered partial
    pub fn edit_buy(&mut self, new_price: u64, new_size: u64, time: i64) {
        self.size = new_size;
        self.price = new_price;
        self.last_edit_time = time;
    }

    /// edit a sell order account
    pub fn edit_sell(&mut self, new_price: u64, time: i64) {
        self.price = new_price;
        self.last_edit_time = time;
    }

    /// return true if the order is active
    pub fn is_active(state: u8) -> bool {
        state != <OrderState as Into<u8>>::into(OrderState::Closed)
    }
}
