use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::state::*;

use super::EditBuyOrderData;

#[derive(Accounts)]
#[instruction(data: EditBuyOrderData)]
pub struct EditBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = wallet.balance >= data.new_size * data.new_price,
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = data.new_size > 0 && data.new_price > 0,
        constraint = Order::is_active(order.state),
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

pub fn handler(ctx: Context<EditBuyOrder>, data: EditBuyOrderData) -> ProgramResult {
    Wallet::edit_bids(&mut ctx.accounts.wallet, false, ctx.accounts.order.size);

    // edit the order with size
    Order::edit_buy(
        &mut ctx.accounts.order,
        data.new_price,
        data.new_size,
        ctx.accounts.clock.unix_timestamp,
    );

    // edit wallet active bids
    Wallet::edit_bids(&mut ctx.accounts.wallet, true, data.new_size);

    Ok(())
}
