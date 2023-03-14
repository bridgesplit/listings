use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token::{Mint, Token, TokenAccount};
use vault::{
    state::{Appraisal, APPRAISAL_SEED},
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{
        freeze_nft, print_webhook_logs_for_order, print_webhook_logs_for_tracker,
        print_webhook_logs_for_wallet,
    },
};

use super::InitOrderData;

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
pub struct InitSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
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
        constraint = data.price > 0 && data.size > 0,
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
        seeds::program = vault::ID,
    )]
    pub appraisal: Box<Account<'info, Appraisal>>,
    #[account(
        init,
        seeds = [TRACKER_SEED.as_ref(),
        nft_mint.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Tracker>()
    )]
    pub tracker: Box<Account<'info, Tracker>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    /// CHECK: checked in cpi
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = nft_ta.owner == initializer.key(),
        constraint = nft_ta.mint == nft_mint.key(),
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<InitSellOrder>, data: InitOrderData) -> ProgramResult {
    msg!("Initialize a new sell order");

    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.wallet.key(),
        data.nonce,
        ctx.accounts.clock.unix_timestamp,
        OrderSide::Sell.into(),
        1,
        data.price,
        OrderState::Ready.into(),
    );

    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.initializer.key.as_ref(),
        bump,
    ][..]];

    // initialize the nft tracker
    Tracker::init(
        &mut ctx.accounts.tracker,
        ctx.accounts.market.key(),
        ctx.accounts.order.key(),
        ctx.accounts.nft_mint.key(),
    );

    // freeze the nft of the seller with the bidding wallet account as the authority
    freeze_nft(
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.nft_ta.to_account_info(),
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        signer_seeds,
    )?;

    Wallet::edit(&mut ctx.accounts.wallet, 0, 1, EditSide::Increase.into());

    // log for webhook calcs
    print_webhook_logs_for_order(ctx.accounts.order.clone(), ctx.accounts.wallet.clone())?;
    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;
    print_webhook_logs_for_tracker(ctx.accounts.tracker.clone())?;
    Ok(())
}
