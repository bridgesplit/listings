use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_lang::{solana_program::sysvar, Key};
use anchor_mpl_token_metadata::state::Metadata;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use vault::utils::{get_bump_in_seed_form, MplTokenMetadata};

use crate::{state::*, utils::unfreeze_nft};

#[derive(Accounts)]
#[instruction()]
pub struct CloseSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = Order::is_active(order.state),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.as_ref(),
        initializer.key().as_ref()],
        bump,
        close = initializer,
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_ref(),
        order.owner.as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    pub nft_metadata: Box<Account<'info, Metadata>>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = initializer,
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub instructions_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
}

pub fn handler(ctx: Context<CloseSellOrder>) -> ProgramResult {
    msg!("Close sell order account: {}", ctx.accounts.order.key());
    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    // unfreeze nft first
    unfreeze_nft(
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.nft_ta.to_account_info(),
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        signer_seeds,
    )?;

    ctx.accounts.order.state = OrderState::Closed.into();
    Ok(())
}
