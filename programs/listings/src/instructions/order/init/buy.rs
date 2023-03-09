use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::{instructions::order::edit::EditSide, state::*};

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
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitBuyOrder>, data: InitOrderData) -> ProgramResult {
    msg!("Initialize a new buy order");

    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.wallet.key(),
        OrderSide::Buy.into(),
        data.size,
        data.price,
        OrderState::Ready.into(),
    );

    // increase wallet active bids
    Wallet::edit_active_bids(
        &mut ctx.accounts.wallet,
        data.size,
        EditSide::Increase.into(),
    );
    Ok(())
}
