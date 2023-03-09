use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token::Mint;

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account()]
    pub pool_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds = [MARKET_SEED.as_ref(),
        pool_mint.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Market>()
    )]
    pub market: Box<Account<'info, Market>>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitMarket>) -> ProgramResult {
    msg!("Initializing new market");
    Market::init(
        &mut ctx.accounts.market,
        ctx.accounts.pool_mint.key(),
        ctx.accounts.initializer.key(),
    );
    Ok(())
}
