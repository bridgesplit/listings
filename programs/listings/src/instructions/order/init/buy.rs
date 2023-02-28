use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::{state::*, utils::transfer_sol};

use super::InitOrderData;

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
pub struct InitBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.owner.as_ref(),
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
        data.side,
        1,
        data.price,
        OrderState::Ready,
    );

    // transfer the buy amount sol to the order account
    transfer_sol(
        ctx.accounts.market.to_account_info(),
        ctx.accounts.order.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        data.price,
    )?;
    Ok(())
}
