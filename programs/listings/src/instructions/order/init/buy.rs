use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{print_webhook_logs_for_order, print_webhook_logs_for_wallet},
};

use super::InitOrderData;

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
pub struct InitBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        // make sure bidding wallet has enough balance to place the order
        constraint = Wallet::validate(wallet.balance, data.price * data.size, EditSide::Increase.into()),
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        constraint = data.price > 0 && data.size > 0,
        init,
        seeds = [ORDER_SEED.as_ref(),
        data.nonce.as_ref(),
        market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Order>()
    )]
    pub order: Box<Account<'info, Order>>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<InitBuyOrder>, data: InitOrderData) -> ProgramResult {
    msg!("Initialize a new listings buy order");

    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.wallet.key(),
        data.nonce,
        ctx.accounts.clock.unix_timestamp,
        OrderSide::Buy.into(),
        data.size,
        data.price,
        OrderState::Ready.into(),
    );

    // increase wallet active bids
    Wallet::edit(
        &mut ctx.accounts.wallet,
        0,
        data.size,
        EditSide::Increase.into(),
    );

    print_webhook_logs_for_order(ctx.accounts.order.clone(), ctx.accounts.wallet.clone())?;
    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;
    Ok(())
}
