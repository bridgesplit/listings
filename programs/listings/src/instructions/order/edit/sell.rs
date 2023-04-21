use anchor_lang::{
    prelude::*,
    solana_program::{entrypoint::ProgramResult, sysvar},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use vault::{
    state::{Appraisal, APPRAISAL_SEED},
    utils::{get_bump_in_seed_form, MplTokenMetadata},
};

use crate::{
    instructions::order::edit::EditSide,
    state::*,
    utils::{print_webhook_logs_for_order, print_webhook_logs_for_wallet},
};
use bridgesplit_program_utils::{
    bridgesplit_delegate, bridgesplit_freeze, bridgesplit_thaw, get_extra_delegate_params,
    pnft::utils::AuthorizationDataLocal, BridgesplitDelegate, BridgesplitFreeze,
    ExtraDelegateParams, ExtraFreezeParams,
};
use mpl_token_metadata::{instruction::DelegateArgs, processor::AuthorizationData};

use super::EditOrderData;

#[derive(Accounts)]
#[instruction(data: EditOrderData)]
pub struct EditSellOrder<'info> {
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
        constraint = data.price > 0,
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

impl<'info> EditSellOrder<'info> {
    pub fn execute_bridgesplit_delegate(
        &self,
        delegate_params: ExtraDelegateParams<'info>,
        amount: u64,
    ) -> Result<()> {
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
        signer_seeds: &[&[&[u8]]],
        freeze_params: ExtraFreezeParams<'info>,
    ) -> Result<()> {
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
            ata_program: self.ata_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.mpl_token_metadata_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        bridgesplit_freeze(cpi_ctx, freeze_params)
    }

    pub fn execute_bridgesplit_thaw(
        &self,
        signer_seeds: &[&[&[u8]]],
        freeze_params: ExtraFreezeParams<'info>,
    ) -> Result<()> {
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
            ata_program: self.ata_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.mpl_token_metadata_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        bridgesplit_thaw(cpi_ctx, freeze_params)
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, EditSellOrder<'info>>,
    data: EditOrderData,
    authorization_data: Option<AuthorizationDataLocal>,
) -> ProgramResult {
    let bump = &get_bump_in_seed_form(ctx.bumps.get("wallet").unwrap());

    let order = ctx.accounts.order.clone();

    let wallet_signer_seeds = &[&[WALLET_SEED.as_ref(), order.owner.as_ref(), bump][..]];

    // update the sell order account
    Order::edit(
        &mut ctx.accounts.order,
        data.price,
        data.size,
        data.side,
        ctx.accounts.clock.unix_timestamp,
    );

    let authorization_data_mplx: Option<AuthorizationData> = if authorization_data.is_some() {
        Some(authorization_data.as_ref().unwrap().clone().into())
    } else {
        None
    };
    let delegate_params = get_extra_delegate_params(
        ctx.remaining_accounts.to_vec(),
        DelegateArgs::UtilityV1 {
            amount: 1,
            authorization_data: authorization_data_mplx,
        },
    );
    let freeze_params = ExtraFreezeParams {
        token_record: delegate_params.token_record.clone(),
        rules_acc: delegate_params.authorization_rules.clone(),
        authorization_data,
        authorization_rules_program: delegate_params.authorization_rules_program.clone(),
    };

    if data.size != 0 {
        // freeze the nft of the seller with the bidding wallet account as the authority if edit side is increase and vice versa
        if EditSide::is_increase(data.side) {
            // initialize the tracker account
            Tracker::init(
                &mut ctx.accounts.tracker,
                ctx.accounts.market.key(),
                ctx.accounts.order.key(),
                ctx.accounts.initializer.key(),
                ctx.accounts.nft_mint.key(),
            );

            ctx.accounts
                .execute_bridgesplit_delegate(delegate_params, 1)?;
            ctx.accounts
                .execute_bridgesplit_freeze(wallet_signer_seeds, freeze_params)?;
        } else {
            // close the tracker account
            ctx.accounts
                .tracker
                .close(ctx.accounts.initializer.to_account_info())?;

            msg!("Closed tracker account: &{:?}&", ctx.accounts.tracker.key());
            ctx.accounts
                .execute_bridgesplit_thaw(wallet_signer_seeds, freeze_params)?;
        }
    }
    print_webhook_logs_for_order(ctx.accounts.order.clone(), ctx.accounts.wallet.clone())?;

    // edit active bids in wallet account
    Wallet::edit(&mut ctx.accounts.wallet, 0, 1, data.side);

    print_webhook_logs_for_wallet(ctx.accounts.wallet.clone())?;
    Ok(())
}
