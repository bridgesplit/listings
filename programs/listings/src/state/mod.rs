pub const MARKET_SEED: &str = "market";
pub const ORDER_SEED: &str = "order";
pub const WALLET_SEED: &str = "wallet";

pub mod market;
pub mod order;
pub mod wallet;

pub use market::*;
pub use order::*;
pub use wallet::*;
