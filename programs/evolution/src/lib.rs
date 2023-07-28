use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use bridgesplit_program_utils::{
    anchor_lang,
    pnft::update::{metaplex_update, MetaplexUpdate, UpdateParams},
};
use mpl_token_metadata::{
    instruction::{
        CollectionDetailsToggle, CollectionToggle, RuleSetToggle, UpdateArgs, UsesToggle,
    },
    solana_program::sysvar,
    state::{Data, Metadata, TokenMetadataAccount},
};

// mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie - prod program id
// tsthbYzhRwHcVgoGJVv87QFFa13V7fLnKMrpgFMEgRa - staging program id

declare_id!("mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie");

#[program]
pub mod listings {
    use super::*;

    pub fn upgrade_nft_ix<'info>(
        ctx: Context<'_, '_, '_, 'info, UpgradeNft<'info>>,
        update_params: UpdateData,
    ) -> Result<()> {
        upgrade_nft(ctx, update_params)
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct UpgradeNft<'info> {
    #[account(
        mut,
        constraint = "collection.owner" == authority.key().to_string(),
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = owner
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,
    /// CHECK: Checks done in Metaplex
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: Checks done in Metaplex
    #[account(mut)]
    pub nft_master_edition: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: Checks done in Metaplex
    pub authorization_rules: UncheckedAccount<'info>,
    /// CHECK: Checks done in Metaplex
    pub authorization_rules_program: UncheckedAccount<'info>,
    /// CHECK: Checks done in Metaplex
    pub mpl_token_metadata: UncheckedAccount<'info>,
    /// CHECK: checked by address and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

#[account]
pub struct UpdateData {
    pub name: String,
    pub uri: String,
}

pub fn upgrade_nft<'info>(
    ctx: Context<'_, '_, '_, 'info, UpgradeNft<'info>>,
    data: UpdateData,
) -> Result<()> {
    let cpi_program = ctx.accounts.mpl_token_metadata.to_account_info();
    let cpi_accounts = MetaplexUpdate {
        authority: ctx.accounts.authority.to_account_info(),
        mint: ctx.accounts.nft_mint.to_account_info(),
        metadata: ctx.accounts.nft_metadata.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
    };

    let metadata = Metadata::from_account_info(&ctx.accounts.nft_metadata.to_account_info())?;

    let cpi_ctx: CpiContext<'_, '_, '_, '_, MetaplexUpdate<'_>> =
        CpiContext::new(cpi_program, cpi_accounts);
    let update_params = UpdateParams {
        update_args: UpdateArgs::V1 {
            new_update_authority: Some(metadata.update_authority),
            data: Some(Data {
                name: data.name,
                symbol: metadata.data.symbol,
                uri: data.uri,
                seller_fee_basis_points: metadata.data.seller_fee_basis_points,
                creators: metadata.data.creators,
            }),
            primary_sale_happened: Some(true),
            is_mutable: Some(true),
            collection: CollectionToggle::None,
            collection_details: CollectionDetailsToggle::None,
            uses: UsesToggle::None,
            rule_set: RuleSetToggle::None,
            authorization_data: None,
        },
        delegate_record: None,
        token: Some(ctx.accounts.nft_ta.to_account_info()),
        edition: Some(ctx.accounts.nft_master_edition.to_account_info()),
        authorization_rules_program: Some(
            ctx.accounts.authorization_rules_program.to_account_info(),
        ),
        authorization_rules: Some(ctx.accounts.authorization_rules.to_account_info()),
    };

    metaplex_update(cpi_ctx, update_params)
}
