use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_mpl_token_metadata::state::Metadata;
use anchor_spl::token::{Mint, Token, TokenAccount};
use vault::{
    state::{Appraisal, APPRAISAL_SEED},
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{freeze_nft, unfreeze_nft},
};

use super::EditOrderData;

#[derive(Accounts)]
#[instruction(data: EditOrderData)]
pub struct EditSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        seeds = [MARKET_SEED.as_ref(),
        market.owner.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = order.market == market.key(),
        constraint = order.owner == initializer.key(),
        // cannot increase size of order if it is already filled/cancelled
        constraint = Order::validate_edit_side(data.side, order.state),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        market.key().as_ref(),
        order.owner.as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        seeds = [APPRAISAL_SEED, market.pool_mint.as_ref(), nft_mint.key().as_ref()],
        bump,
    )]
    pub appraisal: Box<Account<'info, Appraisal>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    pub nft_metadata: Box<Account<'info, Metadata>>,
    #[account(
        constraint = nft_ta.owner == initializer.key(),
        constraint = nft_ta.mint == nft_mint.key(),
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
}

pub fn handler(ctx: Context<EditSellOrder>, data: EditOrderData) -> ProgramResult {
    msg!("Edit sell order");

    let bump = &get_bump_in_seed_form(ctx.bumps.get("order").unwrap());

    let order = ctx.accounts.order.clone();

    let signer_seeds = &[&[
        ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.as_ref(),
        order.owner.as_ref(),
        bump,
    ][..]];

    // update the sell order account
    Order::edit(&mut ctx.accounts.order, data.price, data.side);

    // freeze the nft of the seller with the order account as the authority if edit side is increase and vice versa
    if EditSide::is_increase(data.side) {
        freeze_nft(
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.nft_ta.to_account_info(),
            ctx.accounts.order.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mpl_token_metadata_program.to_account_info(),
            signer_seeds,
        )?;
    } else {
        unfreeze_nft(
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.nft_ta.to_account_info(),
            ctx.accounts.order.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mpl_token_metadata_program.to_account_info(),
            signer_seeds,
        )?;
    }
    Ok(())
}
