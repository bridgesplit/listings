use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_lang::{solana_program::sysvar, Key};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use bridgesplit_program_utils::anchor_lang;
use bridgesplit_program_utils::{pnft::utils::get_is_pnft, state::Metadata, ExtraRevokeParams};
use token_metadata::instruction::RevokeArgs;
use vault::utils::{get_bump_in_seed_form, MplTokenMetadata};

use crate::{
    state::*,
    utils::{parse_remaining_accounts, revoke_nft, unfreeze_nft},
};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct CloseSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = order.market == market.key(),
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
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_ref(),
        order.owner.as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
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
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, MplTokenMetadata>,
}

//remaining accounts
// 0 token_record or default,
// 1 authorization_rules or default,
// 2 authorization_rules_program or default,
// 4 delegate record or default

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CloseSellOrder<'info>>) -> ProgramResult {
    msg!("Close sell order account: {}", ctx.accounts.order.key());

    let parsed_remaining_accounts = parse_remaining_accounts(
        ctx.remaining_accounts.to_vec(),
        ctx.accounts.initializer.key(),
        ctx.accounts.order.fees_on,
        false,
        None,
    );

    let pnft_params = parsed_remaining_accounts.pnft_params;

    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    let is_pnft = get_is_pnft(&ctx.accounts.nft_metadata);

    // unfreeze nft if not pnft
    if !is_pnft {
        unfreeze_nft(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_ta.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            signer_seeds,
            pnft_params.clone(),
        )?;
    } else {
        //revoke nft if pnft
        revoke_nft(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_ta.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            signer_seeds,
            ExtraRevokeParams {
                delegate_record: parsed_remaining_accounts.delegate_record,
                master_edition: Some(ctx.accounts.nft_edition.to_account_info()),
                token_record: pnft_params.token_record,
                authorization_rules: pnft_params.authorization_rules,
                authorization_rules_program: pnft_params.authorization_rules_program,
                revoke_args: RevokeArgs::SaleV1,
            },
        )?;
    }

    ctx.accounts.order.state = OrderState::Closed.into();

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.pool_mint,
        OrderEditType::Close,
    ));
    Ok(())
}
