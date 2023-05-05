pub const MARKET_SEED: &str = "market";
pub const ORDER_SEED: &str = "order";
pub const WALLET_SEED: &str = "wallet";

pub const PROTOCOL_FEES_BPS: u64 = 50;
pub const PROTOCOL_TREASURY: &str = "ovo1kT7RqrAZwFtgSGEgNfa7nHjeZoK6ykg1GknJEXG";

pub mod market;
pub mod order;
pub mod wallet;

pub use market::*;
pub use order::*;
pub use wallet::*;
