use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use bridgesplit_program_utils::{ExtraFreezeParams, BridgesplitFreeze, ExtraTransferParams, BridgesplitTransfer, bridgesplit_thaw, bridgesplit_transfer, get_extra_freeze_params, pnft::utils::AuthorizationDataLocal};
use vault::{
    errors::SpecificErrorCode,
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{
        print_webhook_logs_for_order, print_webhook_logs_for_wallet, transfer_sol,
    },
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
    pub nft_mint: Box<Account<'info, Mint>>,
    /// CHECK: in cpi
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [TRACKER_SEED.as_ref(),
        nft_mint.key().as_ref()],
        bump,
    )]
    pub tracker: Box<Account<'info, Tracker>>,
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


impl<'info>FillSellOrder<'info> {


    pub fn execute_bridgesplit_thaw(
        &self,  
        signer_seeds: &[&[&[u8]]],
        freeze_params: ExtraFreezeParams<'info>
    ) -> Result<()> {
    
        let accounts = BridgesplitFreeze {
            authority: self.initializer.to_account_info(),
            payer: self.initializer.to_account_info(),
            token_owner: self.initializer.to_account_info(),
            token: self.seller_nft_ta.to_account_info(),
            delegate: self.wallet.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            metadata: self.nft_metadata.to_account_info(),
            edition: self.nft_edition.to_account_info(),
            mpl_token_metadata: self.mpl_token_metadata_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            instructions: self.instructions_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            ata_program: self.associated_token_program.to_account_info()
        };
    
        let cpi_ctx = CpiContext::new_with_signer(self.mpl_token_metadata_program.to_account_info(), accounts, signer_seeds);
        bridgesplit_thaw(cpi_ctx, freeze_params)
    
    }


    pub fn execute_bridgesplit_transfer(&self, params: ExtraTransferParams<'info>, amount: u64) -> Result<()> {
        let accounts = BridgesplitTransfer {
            authority: self.initializer.to_account_info(),
            payer: self.initializer.to_account_info(),
            token_owner: self.initializer.to_account_info(),
            token: self.seller_nft_ta.to_account_info(),
            destination_owner: self.initializer.to_account_info(),
            destination: self.buyer_nft_ta.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            metadata: self.nft_metadata.to_account_info(),
            edition: self.nft_edition.to_account_info(),
            system_program: self.system_program.to_account_info(),
            instructions: self.instructions_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            ata_program: self.associated_token_program.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.mpl_token_metadata_program.to_account_info(), accounts);
        bridgesplit_transfer(cpi_ctx, params, amount)


        


    }


}


/// Initializer is the buyer and is buying an nft from the seller
/// The seller is the owner of the order account
/// Buyer transfers sol to seller account
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillSellOrder<'info>>,  authorization_data: Option<AuthorizationDataLocal>) -> Result<()> {
    msg!("Filling order");

    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let signer_seeds = &[&[
        WALLET_SEED.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    let sol_holder = ctx.accounts.initializer.to_account_info();

    // validate seller
    if ctx.accounts.order.owner != ctx.accounts.seller.key() {
        return Err(SpecificErrorCode::WrongAccount.into());
    }

    let freeze_params = get_extra_freeze_params(ctx.remaining_accounts.to_vec(), authorization_data);

    let transfer_params = ExtraTransferParams {
        owner_token_record: freeze_params.token_record.clone(),
        dest_token_record: ctx.remaining_accounts.get(3).cloned(),
        rules_acc: freeze_params.rules_acc.clone(),
        authorization_data:freeze_params.authorization_data.clone(),
        authorization_rules_program: freeze_params.authorization_rules_program.clone()

    };

    // unfreeze nft first so that a transfer can be made
    ctx.accounts.execute_bridgesplit_thaw(signer_seeds, freeze_params)?;

    // edit wallet account to decrease balance and active bids
    Wallet::edit(&mut ctx.accounts.wallet, 0, 1, EditSide::Decrease.into());

    // transfer nft
    ctx.accounts.execute_bridgesplit_transfer(transfer_params, 1)?;

    // transfer sol from buyer to seller
    transfer_sol(
        sol_holder,
        ctx.accounts.seller.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        signer_seeds,
        ctx.accounts.order.price,
    )?;

    // edit order
    let price = ctx.accounts.order.price;
    Order::edit(
        &mut ctx.accounts.order,
        price,
        1,
        EditSide::Decrease.into(),
        ctx.accounts.clock.unix_timestamp,
    );

    // close the tracker account
    ctx.accounts
        .tracker
        .close(ctx.accounts.seller.to_account_info())?;

    msg!("Closed tracker account: &{:?}&", ctx.accounts.tracker.key());

    print_webhook_logs_for_order(ctx.accounts.order.clone(), ctx.accounts.wallet.clone())?;
    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;

    Ok(())
}
