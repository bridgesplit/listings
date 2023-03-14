use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use vault::utils::get_bump_in_seed_form;

use crate::{
    state::*,
    utils::{print_webhook_logs_for_wallet, transfer_sol},
};

#[derive(Accounts)]
#[instruction()]
pub struct InitBiddingWallet<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        payer = initializer,
        space = 8 + std::mem::size_of::<Wallet>(),
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitBiddingWallet>, amount: u64) -> ProgramResult {
    msg!("Initializing a new bidding wallet account");

    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.initializer.key.as_ref(),
        bump,
    ][..]];

    // transfer the amount to the wallet account to initializer if amount > 0
    if amount > 0 {
        transfer_sol(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            signer_seeds,
            amount,
        )?;
    }

    Wallet::init(
        &mut ctx.accounts.wallet,
        ctx.accounts.initializer.key(),
        amount,
    );

    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;

    Ok(())
}
