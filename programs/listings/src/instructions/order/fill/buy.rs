use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use bridgesplit_program_utils::{
    get_bump_in_seed_form, state::Metadata, ExtraRevokeParams, ExtraTransferParams,
    MplTokenMetadata,
};
use mpl_token_metadata::instruction::RevokeArgs;
use vault::utils::lamport_transfer;

use crate::{
    state::*,
    utils::{get_pnft_params, transfer_nft, unfreeze_nft},
};

#[derive(Accounts)]
#[instruction()]
pub struct FillBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == buyer.key(),
    )]
    /// CHECK: constraint check
    pub buyer: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_ref(),
        order.owner.as_ref()],
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
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        constraint = order.nft_mint == Pubkey::default() || order.nft_mint == nft_mint.key()
    )]
    pub nft_mint: Box<Account<'info, Mint>>,
    pub nft_metadata: Box<Account<'info, Metadata>>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = initializer,
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
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
    pub clock: Sysvar<'info, Clock>,
}

/// seller is initializer and is transferring the nft to buyer who is the owner of the order account
/// buyer is the owner of the order account and is transferring sol to seller via bidding wallet
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillBuyOrder<'info>>) -> Result<()> {
    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());
    let pnft_params = get_pnft_params(ctx.remaining_accounts.to_vec());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    // edit wallet account to decrease balance
    msg!("Edit wallet balance: {}", ctx.accounts.wallet.key());
    Wallet::edit_balance(&mut ctx.accounts.wallet, false, ctx.accounts.order.price);

    // unfreeze nft for transfer
    unfreeze_nft(
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.seller_nft_ta.to_account_info(),
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.associated_token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        signer_seeds,
        ExtraRevokeParams {
            master_edition: Some(ctx.accounts.nft_edition.to_account_info()),
            delegate_record: pnft_params.token_record.clone(),
            token_record: pnft_params.token_record.clone(),
            authorization_rules_program: pnft_params.authorization_rules_program.clone(),
            authorization_rules: pnft_params.authorization_rules.clone(),
            token: Some(ctx.accounts.nft_mint.to_account_info()),
            spl_token_program: Some(ctx.accounts.token_program.to_account_info()),
            delegate_args: RevokeArgs::UtilityV1,
        },
        pnft_params.clone(),
    )?;

    // transfer nft
    transfer_nft(
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.buyer.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.seller_nft_ta.to_account_info(),
        ctx.accounts.buyer_nft_ta.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.associated_token_program.to_account_info(),
        ctx.accounts.mpl_token_metadata_program.to_account_info(),
        ExtraTransferParams {
            owner_token_record: ctx.remaining_accounts.get(3).cloned(),
            dest_token_record: pnft_params.token_record,
            authorization_rules: pnft_params.authorization_rules,
            authorization_rules_program: pnft_params.authorization_rules_program.clone(),
            authorization_data: None,
        },
    )?;

    // transfer sol from buyer to seller
    lamport_transfer(
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.order.price,
    )?;

    // edit order
    let price = ctx.accounts.order.price;
    let size = ctx.accounts.order.size;

    Order::edit_buy(
        &mut ctx.accounts.order,
        price,
        size - 1,
        ctx.accounts.clock.unix_timestamp,
    );

    if size == 1 {
        // close order account
        msg!("Close buy order account: {}", ctx.accounts.order.key());
        ctx.accounts.order.state = OrderState::Closed.into();
        ctx.accounts
            .order
            .close(ctx.accounts.buyer.to_account_info())?;
    } else {
        msg!("Filled buy order: {}", ctx.accounts.order.key());
    }

    Ok(())
}
