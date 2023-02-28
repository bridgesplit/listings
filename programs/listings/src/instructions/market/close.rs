use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
pub struct CloseMarket<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = market.owner == initializer.key(),
        seeds = [MARKET_SEED.as_ref(),
        market.owner.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
}

pub fn handler(ctx: Context<CloseMarket>) -> ProgramResult {
    msg!("Closing market");
    ctx.accounts.market.state = MarketState::Closed;
    Ok(())
}
