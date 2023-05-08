use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use bridgesplit_program_utils::anchor_lang;

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
    pub fn init_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, InitSellOrder<'info>>,
        data: InitOrderData,
    ) -> ProgramResult {
        instructions::order::init::sell::handler(ctx, data)
    }

    /// edit a bid
    pub fn edit_buy_order(ctx: Context<EditBuyOrder>, data: EditBuyOrderData) -> ProgramResult {
        instructions::order::edit::buy::handler(ctx, data)
    }

    /// edit a listing
    pub fn edit_sell_order(ctx: Context<EditSellOrder>, data: EditSellOrderData) -> ProgramResult {
        instructions::order::edit::sell::handler(ctx, data)
    }

    /// fill a bid
    pub fn fill_buy_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillBuyOrder<'info>>,
    ) -> Result<()> {
        instructions::order::fill::buy::handler(ctx)
    }

    /// fill a listing
    pub fn fill_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillSellOrder<'info>>,
    ) -> Result<()> {
        instructions::order::fill::sell::handler(ctx)
    }

    /// cancel a buy order
    pub fn close_buy_order(ctx: Context<CloseBuyOrder>) -> ProgramResult {
        instructions::order::close::buy::handler(ctx)
    }

    /// cancel a sell order
    pub fn close_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseSellOrder<'info>>,
    ) -> ProgramResult {
        instructions::order::close::sell::handler(ctx)
    }

    /// initializer a new bidding wallet
    pub fn init_wallet(ctx: Context<InitBiddingWallet>, amount: u64) -> ProgramResult {
        instructions::wallet::init::handler(ctx, amount)
    }

    /// edit a bidding wallet
    pub fn edit_wallet(
        ctx: Context<EditBiddingWallet>,
        amount_change: u64,
        increase: bool,
    ) -> ProgramResult {
        instructions::wallet::edit::handler(ctx, amount_change, increase)
    }
}
