use anchor_lang::Key;
use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
pub struct CloseOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
}

pub fn handler(ctx: Context<CloseOrder>) -> ProgramResult {
    msg!("Close order account");
    ctx.accounts.order.state = OrderState::Closed.into();
    Ok(())
}
