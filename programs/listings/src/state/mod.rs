pub const MARKET_SEED: &str = "market";
pub const ORDER_SEED: &str = "order";
pub const TRACKER_SEED: &str = "tracker";
pub const WALLET_SEED: &str = "wallet";

pub mod market;
pub mod order;
pub mod tracker;
pub mod wallet;

pub use market::*;
pub use order::*;
pub use tracker::*;
pub use wallet::*;
