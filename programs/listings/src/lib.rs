use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie");

#[program]
pub mod listings {
    use super::*;

    pub fn init_market(ctx: Context<InitMarket>) -> ProgramResult {
        instructions::market::init::handler(ctx)
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> ProgramResult {
        instructions::market::close::handler(ctx)
    }

    pub fn init_buy_order(ctx: Context<InitBuyOrder>, data: InitOrderData) -> ProgramResult {
        instructions::order::init::buy::handler(ctx, data)
    }

    pub fn init_sell_order(ctx: Context<InitSellOrder>, data: InitOrderData) -> ProgramResult {
        instructions::order::init::sell::handler(ctx, data)
    }

    pub fn edit_buy_order(ctx: Context<EditBuyOrder>, data: EditOrderData) -> ProgramResult {
        instructions::order::edit::buy::handler(ctx, data)
    }

    pub fn edit_sell_order(ctx: Context<EditSellOrder>, data: EditOrderData) -> ProgramResult {
        instructions::order::edit::sell::handler(ctx, data)
    }

    pub fn fill_order<'info>(ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>) -> Result<()> {
        instructions::order::fill::handler(ctx)
    }

    pub fn close_order(ctx: Context<CloseOrder>) -> ProgramResult {
        instructions::order::close::handler(ctx)
    }
}
