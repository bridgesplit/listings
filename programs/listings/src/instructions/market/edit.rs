use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token::Mint;

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
pub struct EditMarket<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account()]
    pub pool_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = market.owner == initializer.key(),
        seeds = [MARKET_SEED.as_ref(),
        market.owner.as_ref(),
        market.pool_mint.as_ref()],
        bump
    )]
    pub market: Box<Account<'info, Market>>,
}

pub fn handler(ctx: Context<EditMarket>) -> ProgramResult {
    msg!("Editing market");
    ctx.accounts.market.pool_mint = ctx.accounts.pool_mint.key();
    Ok(())
}
