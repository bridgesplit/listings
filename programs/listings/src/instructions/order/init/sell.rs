use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token::{Mint, Token, TokenAccount};
use vault::{
    state::{Appraisal, APPRAISAL_SEED},
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{state::*, utils::freeze_nft};

use super::InitOrderData;

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
pub struct InitSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.owner.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        init,
        seeds = [ORDER_SEED.as_ref(),
        data.nonce.as_ref(),
        market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Order>()
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        seeds = [APPRAISAL_SEED, market.pool_mint.as_ref(), nft_mint.key().as_ref()],
        bump,
    )]
    pub appraisal: Box<Account<'info, Appraisal>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    /// CHECK: checked in cpi
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        constraint = nft_ta.owner == initializer.key(),
        constraint = nft_ta.mint == nft_mint.key(),
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
}

pub fn handler(ctx: Context<InitSellOrder>, data: InitOrderData) -> ProgramResult {
    msg!("Initialize a new sell order");

    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        data.side,
        1,
        data.price,
        OrderState::Ready.into(),
    );

    let bump = &get_bump_in_seed_form(ctx.bumps.get("order").unwrap());

    let signer_seeds = &[&[
        ORDER_SEED.as_ref(),
        ctx.accounts.order.nonce.as_ref(),
        ctx.accounts.order.market.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    // freeze the nft of the seller with the order account as the authority
    freeze_nft(
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.nft_ta.to_account_info(),
        ctx.accounts.order.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        signer_seeds,
    )?;
    Ok(())
}
