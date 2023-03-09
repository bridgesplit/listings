use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::{state::*, utils::transfer_sol};

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
    msg!("Editing sol balancer of the bidding wallet account");

    // transfer the amount to the wallet account to initializer if it is a deposit
    // transfer the amount from the wallet account to initializer if it is a withdraw
    transfer_sol(
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        amount,
    )?;

    Wallet::edit_balance(&mut ctx.accounts.wallet, amount, edit_side);
    Ok(())
}
