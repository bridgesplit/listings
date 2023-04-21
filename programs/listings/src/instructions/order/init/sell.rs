use anchor_lang::{prelude::*, solana_program::{entrypoint::ProgramResult, sysvar}};
use anchor_spl::{token::{Mint, Token, TokenAccount}, associated_token::AssociatedToken};
use bridgesplit_program_utils::{BridgesplitFreeze, bridgesplit_freeze, ExtraFreezeParams, ExtraDelegateParams, get_extra_delegate_params,  pnft::utils::AuthorizationDataLocal, BridgesplitDelegate, bridgesplit_delegate};
use vault::{
    state::{Appraisal, APPRAISAL_SEED},
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};
use mpl_token_metadata::{processor::{AuthorizationData}, instruction::DelegateArgs};


use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{
        print_webhook_logs_for_order, print_webhook_logs_for_tracker,
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
    /// CHECK: constraint checks in cpis
    pub nft_edition: UncheckedAccount<'info>,
    /// CHECK: constraint checks in cpis
    pub nft_metadata: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = nft_ta.owner == initializer.key(),
        constraint = nft_ta.mint == nft_mint.key(),
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub ata_program: Program<'info, AssociatedToken>,
    #[account(address = sysvar::instructions::id())]
    /// CHECK: address check
    pub instructions_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub mpl_token_metadata_program: Program<'info, MplTokenMetadata>,
    pub clock: Sysvar<'info, Clock>,
}


impl<'info>InitSellOrder<'info> {

    pub fn execute_bridgesplit_delegate(
        &self,  
        delegate_params: ExtraDelegateParams<'info>,
        amount: u64) -> Result<()> {
    
        let accounts = BridgesplitDelegate {
            authority: self.initializer.to_account_info(),
            payer: self.initializer.to_account_info(),
            token_ta: self.nft_ta.to_account_info(),
            delegate: self.wallet.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            metadata: self.nft_metadata.to_account_info(),
            system_program: self.system_program.to_account_info(),
            sysvar_instructions: self.instructions_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
    
        let cpi_ctx = CpiContext::new(self.mpl_token_metadata_program.to_account_info(), accounts);
        bridgesplit_delegate(cpi_ctx, delegate_params, amount)
    
    }
    
    pub fn execute_bridgesplit_freeze(
        &self,  
        freeze_params: ExtraFreezeParams<'info>, 
        signer_seeds: &[&[&[u8]]]) -> Result<()> {
    
        let accounts = BridgesplitFreeze {
            authority: self.initializer.to_account_info(),
            payer: self.initializer.to_account_info(),
            token_owner: self.initializer.to_account_info(),
            token: self.nft_ta.to_account_info(),
            delegate: self.wallet.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            metadata: self.nft_metadata.to_account_info(),
            edition: self.nft_edition.to_account_info(),
            mpl_token_metadata: self.mpl_token_metadata_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            instructions: self.instructions_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            ata_program: self.ata_program.to_account_info()
        };
    
        let cpi_ctx = CpiContext::new_with_signer(self.mpl_token_metadata_program.to_account_info(), accounts, signer_seeds);
        bridgesplit_freeze(cpi_ctx, freeze_params)
    
    }





}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, InitSellOrder<'info>>, data: InitOrderData, authorization_data: Option<AuthorizationDataLocal>) -> ProgramResult {
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
        ctx.accounts.initializer.key(),
        ctx.accounts.nft_mint.key(),
    );

    let authorization_data_mplx: Option<AuthorizationData> = if authorization_data.is_some() {
        Some(authorization_data.as_ref().unwrap().clone().into())
    } else {
        None
    };
    

    let delegate_params:ExtraDelegateParams<'info> = get_extra_delegate_params(ctx.remaining_accounts.to_vec(), DelegateArgs::UtilityV1 { amount: 1, authorization_data: authorization_data_mplx });
    let freeze_params = ExtraFreezeParams { token_record: delegate_params.token_record.clone(), rules_acc: delegate_params.authorization_rules.clone(), authorization_data: authorization_data, authorization_rules_program: delegate_params.authorization_rules_program.clone() };

    ctx.accounts.execute_bridgesplit_delegate(delegate_params, 1)?;

    ctx.accounts.execute_bridgesplit_freeze(freeze_params, signer_seeds)?;

    // freeze the nft of the seller with the bidding wallet account as the authority

    Wallet::edit(&mut ctx.accounts.wallet, 0, 1, EditSide::Increase.into());

    // log for webhook calcs
    print_webhook_logs_for_order(ctx.accounts.order.clone(), ctx.accounts.wallet.clone())?;
    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;
    print_webhook_logs_for_tracker(ctx.accounts.tracker.clone())?;
    Ok(())
}
