use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use vault::utils::{get_bump_in_seed_form, lamport_transfer};

use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{print_webhook_logs_for_wallet, transfer_sol},
};

#[derive(Accounts)]
#[instruction()]
pub struct EditBiddingWallet<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EditBiddingWallet>, amount: u64, edit_side: u8) -> ProgramResult {
    msg!("Editing sol balance of the listings bidding wallet account");

    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.initializer.key.as_ref(),
        bump,
    ][..]];

    Wallet::edit(&mut ctx.accounts.wallet, amount, 0, edit_side);

    // transfer the amount to the wallet account to initializer if it is a deposit
    // transfer the amount from the wallet account to initializer if it is a withdraw
    if EditSide::is_increase(edit_side) {
        transfer_sol(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            signer_seeds,
            amount,
        )?;
    } else {
        lamport_transfer(
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            amount,
        )?;
    }

    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;

    Ok(())
}
