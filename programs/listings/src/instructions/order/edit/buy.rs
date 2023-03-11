use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::state::*;

use super::EditOrderData;

#[derive(Accounts)]
#[instruction(data: EditOrderData)]
pub struct EditBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = Wallet::validate(wallet.balance, data.price, data.side),
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(
        constraint = Order::validate_edit_side(data.side, market.state),
        constraint = data.price > 0 && data.size > 0,
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        // cannot increase size of order if it is already filled/cancelled
        constraint = Order::validate_edit_side(data.side, order.state),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<EditBuyOrder>, data: EditOrderData) -> ProgramResult {
    msg!("Edit buy order");

    // edit the order with size
    Order::edit(
        &mut ctx.accounts.order,
        data.price,
        data.size,
        data.side,
        ctx.accounts.clock.unix_timestamp,
    );

    // edit wallet active bids
    Wallet::edit(&mut ctx.accounts.wallet, 0, data.size, data.side);
    Ok(())
}
