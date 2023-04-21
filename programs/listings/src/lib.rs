use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

// mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie - prod program id
// tsthbYzhRwHcVgoGJVv87QFFa13V7fLnKMrpgFMEgRa - staging program id

declare_id!("tsthbYzhRwHcVgoGJVv87QFFa13V7fLnKMrpgFMEgRa");
use bridgesplit_program_utils::pnft::utils::AuthorizationDataLocal;

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
        order_data: InitOrderData,
        authorization_data: Option<AuthorizationDataLocal>,
    ) -> ProgramResult {
        instructions::order::init::sell::handler(ctx, order_data, authorization_data)
    }

    /// edit a bid
    pub fn edit_buy_order(ctx: Context<EditBuyOrder>, order_data: EditOrderData) -> ProgramResult {
        instructions::order::edit::buy::handler(ctx, order_data)
    }

    /// edit a listing
    pub fn edit_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, EditSellOrder<'info>>,
        order_data: EditOrderData,
        authorization_data: Option<AuthorizationDataLocal>,
    ) -> ProgramResult {
        instructions::order::edit::sell::handler(ctx, order_data, authorization_data)
    }

    /// fill a bid
    pub fn fill_buy_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillBuyOrder<'info>>,
        authorization_data: Option<AuthorizationDataLocal>,
    ) -> Result<()> {
        instructions::order::fill::buy::handler(ctx, authorization_data)
    }

    /// fill a listing
    pub fn fill_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillSellOrder<'info>>,
        authorization_data: Option<AuthorizationDataLocal>,
    ) -> Result<()> {
        instructions::order::fill::sell::handler(ctx, authorization_data)
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
