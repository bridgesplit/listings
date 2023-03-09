use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_mpl_token_metadata::state::Metadata;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use vault::{
    errors::SpecificErrorCode,
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{
    state::*,
    utils::{transfer_nft, transfer_sol, unfreeze_nft},
};

use super::EditSide;

#[derive(Accounts)]
#[instruction()]
pub struct FillOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account()]
    /// CHECK: constraint check
    pub buyer: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_ref(),
        buyer.key().as_ref()],
        bump,
    )]
    pub buyer_wallet: Box<Account<'info, Wallet>>,
    #[account()]
    /// CHECK: constraint check
    pub seller: UncheckedAccount<'info>,
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
        order.owner.as_ref(),
        market.key().as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    pub nft_metadata: Box<Account<'info, Metadata>>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_nft_ta: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub instructions_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
}

/// seller is always the one who is transferring the nft
/// buyer is always the one who is receiving the nft
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>) -> Result<()> {
    msg!("Filling order");

    let bump = &get_bump_in_seed_form(ctx.bumps.get("sell_order").unwrap());

    let signer_seeds = &[&[
        ORDER_SEED.as_ref(),
        ctx.accounts.order.nonce.as_ref(),
        ctx.accounts.order.market.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    let nft_authority: AccountInfo;
    let sol_holder: AccountInfo;

    if ctx.accounts.order.side == <OrderSide as Into<u8>>::into(OrderSide::Buy) {
        // Initializer is seller and selling an nft to fill a buy order
        // transfer nft from seller to buyer
        // transfer sol from order account to seller

        // validate buyer
        if ctx.accounts.order.owner != ctx.accounts.buyer.key() {
            return Err(SpecificErrorCode::WrongAccount.into());
        }
        nft_authority = ctx.accounts.seller.to_account_info();
        sol_holder = ctx.accounts.buyer_wallet.to_account_info();

        // edit wallet account to decrease balance and active bids
        Wallet::edit_active_bids(&mut ctx.accounts.buyer_wallet, 1, EditSide::Decrease.into());
        Wallet::edit_balance(
            &mut ctx.accounts.buyer_wallet,
            ctx.accounts.order.price,
            EditSide::Decrease.into(),
        );
    } else {
        // Initializer is buyer and buying an nft to fill a sell order
        // transfer nft from order account to buyer
        // transfer sol from buyer to seller

        // validate seller
        if ctx.accounts.order.owner != ctx.accounts.seller.key() {
            return Err(SpecificErrorCode::WrongAccount.into());
        }

        // unfreeze nft first so that a transfer can be made
        unfreeze_nft(
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.seller_nft_ta.to_account_info(),
            ctx.accounts.order.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mpl_token_metadata_program.to_account_info(),
            signer_seeds,
        )?;
        nft_authority = ctx.accounts.order.to_account_info();
        sol_holder = ctx.accounts.buyer.to_account_info();
    }

    let remaining_accounts = ctx.remaining_accounts.to_vec();

    // transfer nft
    transfer_nft(
        nft_authority,
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.buyer.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.seller_nft_ta.to_account_info(),
        ctx.accounts.buyer_nft_ta.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.instructions_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.associated_token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        remaining_accounts,
    )?;

    // transfer sol from buyer to seller
    transfer_sol(
        sol_holder,
        ctx.accounts.seller.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.order.price,
    )?;

    // edit order
    let price = ctx.accounts.order.price;
    Order::edit(&mut ctx.accounts.order, price, 1, EditSide::Decrease.into());

    Ok(())
}
