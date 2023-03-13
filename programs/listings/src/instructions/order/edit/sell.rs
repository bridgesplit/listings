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
        freeze_nft, print_webhook_logs_for_order, print_webhook_logs_for_wallet, unfreeze_nft,
    },
};

use super::EditOrderData;

#[derive(Accounts)]
#[instruction(data: EditOrderData)]
pub struct EditSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = Wallet::validate(wallet.balance, data.price, data.side),
        seeds = [WALLET_SEED.as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(
        constraint = Order::validate_edit_side(data.side, market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = order.market == market.key(),
        constraint = order.owner == initializer.key(),
        constraint = data.price > 0 && data.size > 0,
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
        init_if_needed,
        seeds = [TRACKER_SEED.as_ref(),
        nft_mint.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Tracker>()
    )]
    pub tracker: Box<Account<'info, Tracker>>,
    #[account(
        seeds = [APPRAISAL_SEED, market.pool_mint.as_ref(), nft_mint.key().as_ref()],
        bump,
        seeds::program = vault::ID,
    )]
    pub appraisal: Box<Account<'info, Appraisal>>,
    pub nft_mint: Box<Account<'info, Mint>>,
    /// CHECK: constraint checks in cpis
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = nft_ta.owner == initializer.key(),
        constraint = nft_ta.mint == nft_mint.key(),
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<EditSellOrder>, data: EditOrderData) -> ProgramResult {
    let bump = &get_bump_in_seed_form(ctx.bumps.get("order").unwrap());

    let order = ctx.accounts.order.clone();

    let signer_seeds = &[&[WALLET_SEED.as_ref(), order.owner.as_ref(), bump][..]];

    // update the sell order account
    Order::edit(
        &mut ctx.accounts.order,
        data.price,
        1,
        data.side,
        ctx.accounts.clock.unix_timestamp,
    );

    // freeze the nft of the seller with the bidding wallet account as the authority if edit side is increase and vice versa
    if EditSide::is_increase(data.side) {
        // initialize the tracker account
        Tracker::init(
            &mut ctx.accounts.tracker,
            ctx.accounts.market.key(),
            ctx.accounts.order.key(),
            ctx.accounts.nft_mint.key(),
        );

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
    } else {
        // close the tracker account
        ctx.accounts
            .tracker
            .close(ctx.accounts.initializer.to_account_info())?;

        msg!("Closed tracker account: {:?}", ctx.accounts.tracker.key());
        
        unfreeze_nft(
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.nft_ta.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mpl_token_metadata_program.to_account_info(),
            signer_seeds,
        )?;
    }

    print_webhook_logs_for_order(&mut ctx.accounts.order)?;

    // edit active bids in wallet account
    Wallet::edit(&mut ctx.accounts.wallet, 0, 1, data.side);

    print_webhook_logs_for_wallet(&mut ctx.accounts.wallet)?;
    Ok(())
}
