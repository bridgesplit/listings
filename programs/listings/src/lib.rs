use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

// mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie - prod program id
// tsthbYzhRwHcVgoGJVv87QFFa13V7fLnKMrpgFMEgRa - staging program id

declare_id!("tsthbYzhRwHcVgoGJVv87QFFa13V7fLnKMrpgFMEgRa");

#[program]
pub mod listings {
    use super::*;

    /// initializer a new market
    pub fn init_market(ctx: Context<InitMarket>) -> ProgramResult {
        instructions::market::init::handler(ctx)
    }

    /// close a market
    pub fn close_market(ctx: Context<CloseMarket>) -> ProgramResult {
        instructions::market::close::handler(ctx)
    }

    /// initializer a new bid
    pub fn init_buy_order(ctx: Context<InitBuyOrder>, data: InitOrderData) -> ProgramResult {
        instructions::order::init::buy::handler(ctx, data)
    }

    /// initializer a new listing
    pub fn init_sell_order(ctx: Context<InitSellOrder>, data: InitOrderData) -> ProgramResult {
        instructions::order::init::sell::handler(ctx, data)
    }

    /// edit a bid
    pub fn edit_buy_order(ctx: Context<EditBuyOrder>, data: EditOrderData) -> ProgramResult {
        instructions::order::edit::buy::handler(ctx, data)
    }

    /// edit a listing
    pub fn edit_sell_order(ctx: Context<EditSellOrder>, data: EditOrderData) -> ProgramResult {
        instructions::order::edit::sell::handler(ctx, data)
    }

    /// fill a bid/listing
    pub fn fill_order<'info>(ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>) -> Result<()> {
        instructions::order::fill::handler(ctx)
    }

    /// cancel a bid/listing
    pub fn close_order(ctx: Context<CloseOrder>) -> ProgramResult {
        instructions::order::close::handler(ctx)
    }

    /// initializer a new bidding wallet
    pub fn init_wallet(ctx: Context<InitBiddingWallet>, amount: u64) -> ProgramResult {
        instructions::wallet::init::handler(ctx, amount)
    }

    /// edit a bidding wallet
    pub fn edit_wallet(
        ctx: Context<EditBiddingWallet>,
        amount: u64,
        edit_side: u8,
    ) -> ProgramResult {
        instructions::wallet::edit::handler(ctx, amount, edit_side)
    }
}
