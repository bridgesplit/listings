use anchor_lang::{
    prelude::*,
    solana_program::{entrypoint::ProgramResult, sysvar},
};
use anchor_mpl_token_metadata::state::Metadata;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use vault::utils::{get_bump_in_seed_form, MplTokenMetadata};

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
    #[account(
        constraint = buy_order.owner == buyer.key()
    )]
    /// CHECK: constraint check
    pub buyer: UncheckedAccount<'info>,
    #[account(
        constraint = buy_order.owner == buyer.key()
    )]
    /// CHECK: constraint check
    pub seller: UncheckedAccount<'info>,
    #[account(
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.owner.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = Order::is_active(sell_order.state),
        constraint = sell_order.market == market.key(),
        seeds = [ORDER_SEED.as_ref(),
        sell_order.nonce.as_ref(),
        sell_order.owner.as_ref(),
        market.key().as_ref()],
        bump,
    )]
    pub sell_order: Box<Account<'info, Order>>,
    #[account(
        mut,
        constraint = Order::is_active(buy_order.state),
        constraint = buy_order.market == market.key(),
        seeds = [ORDER_SEED.as_ref(),
        buy_order.nonce.as_ref(),
        buy_order.owner.as_ref(),
        market.key().as_ref()],
        bump,
    )]
    pub buy_order: Box<Account<'info, Order>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    pub nft_metadata: Box<Account<'info, Metadata>>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>) -> ProgramResult {
    msg!("Filling order");
    let bump = &get_bump_in_seed_form(ctx.bumps.get("sell_order").unwrap());

    let signer_seeds = &[&[
        ORDER_SEED.as_ref(),
        ctx.accounts.sell_order.nonce.as_ref(),
        ctx.accounts.sell_order.market.as_ref(),
        ctx.accounts.sell_order.owner.as_ref(),
        bump,
    ][..]];

    // unfreeze nft first so that a transfer can be made
    unfreeze_nft(
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.seller_nft_ta.to_account_info(),
        ctx.accounts.sell_order.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        signer_seeds,
    )?;

    let remaining_accounts = ctx.remaining_accounts.to_vec();

    // transfer nft from sell order account to buyer
    transfer_nft(
        ctx.accounts.sell_order.to_account_info(),
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

    // transfer sol from buy order account to seller
    transfer_sol(
        ctx.accounts.buy_order.to_account_info(),
        ctx.accounts.seller.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sell_order.price,
    )?;

    // if buy order price is greater than sell order price transfer remaining sol to buyer back
    if Order::spill_over(ctx.accounts.buy_order.price, ctx.accounts.sell_order.price) {
        transfer_sol(
            ctx.accounts.buy_order.to_account_info(),
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.buy_order.price - ctx.accounts.sell_order.price,
        )?;
    };

    // edit buy order
    let buy_price = ctx.accounts.buy_order.price;
    Order::edit(
        &mut ctx.accounts.buy_order,
        buy_price,
        EditSide::Decrease.into(),
    );

    // edit sell order
    let sell_price = ctx.accounts.sell_order.price;
    Order::edit(
        &mut ctx.accounts.sell_order,
        sell_price,
        EditSide::Decrease.into(),
    );

    Ok(())
}
