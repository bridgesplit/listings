use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use bridgesplit_program_utils::state::Metadata;
use vault::{
    errors::SpecificErrorCode,
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{
    state::*,
    utils::{transfer_nft, transfer_sol, unfreeze_nft},
};

#[derive(Accounts)]
#[instruction()]
pub struct FillSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == seller.key(),
    )]
    /// CHECK: constraint check
    pub seller: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = Order::is_active(order.state),
        constraint = order.market == market.key(),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.as_ref(),
        order.owner.as_ref()],
        bump,
        close = seller
    )]
    pub order: Box<Account<'info, Order>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    pub nft_metadata: Box<Account<'info, Metadata>>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = nft_mint,
        associated_token::authority = initializer,
    )]
    pub buyer_nft_ta: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub instructions_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
    pub clock: Sysvar<'info, Clock>,
}

/// Initializer is the buyer and is buying an nft from the seller
/// The seller is the owner of the order account
/// Buyer transfers sol to seller account
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillSellOrder<'info>>) -> Result<()> {
    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    let nft_authority = ctx.accounts.wallet.to_account_info();
    let sol_holder = ctx.accounts.initializer.to_account_info();

    // validate seller
    if ctx.accounts.order.owner != ctx.accounts.seller.key() {
        return Err(SpecificErrorCode::WrongAccount.into());
    }

    // // unfreeze nft first so that a transfer can be made
    // unfreeze_nft(
    //     ctx.accounts.nft_mint.to_account_info(),
    //     ctx.accounts.nft_edition.to_account_info(),
    //     ctx.accounts.seller_nft_ta.to_account_info(),
    //     ctx.accounts.wallet.to_account_info(),
    //     ctx.accounts.token_program.to_account_info(),
    //     ctx.accounts.mpl_token_metadata_program.to_account_info(),
    //     signer_seeds,
    // )?;

    // edit wallet account to decrease balance and active bids
    msg!("Edit wallet balance: {}", ctx.accounts.wallet.key());
    Wallet::edit_balance(&mut ctx.accounts.wallet, false, ctx.accounts.order.price);

    let remaining_accounts = ctx.remaining_accounts.to_vec();

    // // transfer nft
    // transfer_nft(
    //     nft_authority,
    //     ctx.accounts.initializer.to_account_info(),
    //     ctx.accounts.initializer.to_account_info(),
    //     ctx.accounts.nft_mint.to_account_info(),
    //     ctx.accounts.nft_metadata.to_account_info(),
    //     ctx.accounts.nft_edition.to_account_info(),
    //     ctx.accounts.seller_nft_ta.to_account_info(),
    //     ctx.accounts.buyer_nft_ta.to_account_info(),
    //     ctx.accounts.system_program.to_account_info(),
    //     ctx.accounts.instructions_program.to_account_info(),
    //     ctx.accounts.token_program.to_account_info(),
    //     ctx.accounts.associated_token_program.to_account_info(),
    //     ctx.accounts.mpl_token_metadata_program.to_account_info(),
    //     remaining_accounts,
    //     signer_seeds,
    // )?;

    // transfer sol from buyer to seller
    transfer_sol(
        sol_holder,
        ctx.accounts.seller.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        signer_seeds,
        ctx.accounts.order.price,
    )?;

    // close order account
    msg!("Close sell order account: {}", ctx.accounts.order.key());
    ctx.accounts.order.state = OrderState::Closed.into();

    Ok(())
}
