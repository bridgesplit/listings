use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use bridgesplit_program_utils::anchor_lang;
use vault::utils::{get_bump_in_seed_form, lamport_transfer};

use crate::{state::*, utils::transfer_sol};

#[derive(Accounts)]
#[instruction(amount_change: u64, is_increase: bool)]
pub struct EditBiddingWallet<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = is_increase || amount_change <= wallet.balance,
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<EditBiddingWallet>,
    amount_change: u64,
    is_increase: bool,
) -> ProgramResult {
    msg!("Edit wallet balance: {}", ctx.accounts.wallet.key());

    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.initializer.key.as_ref(),
        bump,
    ][..]];

    Wallet::edit_balance(&mut ctx.accounts.wallet, is_increase, amount_change);

    // transfer the amount to the wallet account to initializer if it is a deposit
    // transfer the amount from the wallet account to initializer if it is a withdraw
    if is_increase {
        transfer_sol(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            Some(signer_seeds),
            amount_change,
        )?;
    } else {
        lamport_transfer(
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            amount_change,
        )?;
    }

    Wallet::emit_event(
        &mut ctx.accounts.wallet.clone(),
        ctx.accounts.wallet.key(),
        WalletEditType::Edit,
    );
    Ok(())
}
