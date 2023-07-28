use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, mint_to, Burn, Mint, MintTo, Token, TokenAccount},
};
use bridgesplit_program_utils::{
    anchor_lang,
    pnft::update::{metaplex_update, MetaplexUpdate, UpdateParams},
    MplTokenMetadata,
};
use mpl_token_metadata::{
    instruction::{
        CollectionDetailsToggle, CollectionToggle, RuleSetToggle, UpdateArgs, UsesToggle,
    },
    solana_program::{native_token::LAMPORTS_PER_SOL, sysvar},
    state::{Data, Metadata, TokenMetadataAccount},
};

// mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie - prod program id
// tsthbYzhRwHcVgoGJVv87QFFa13V7fLnKMrpgFMEgRa - staging program id

declare_id!("mrkTzoWMVEBJ3AUrgd2eXNLXrnBuhhQRQyxahtaeTie");

#[program]
pub mod listings {
    use super::*;

    pub fn upgrade_nft<'info>(
        ctx: Context<'_, '_, '_, 'info, UpgradeNft<'info>>,
        update_params: UpdateData,
    ) -> Result<()> {
        upgrade_nft_ix(ctx, update_params)
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct UpgradeNft<'info> {
    #[account(
        mut,
        constraint = authority.key().to_string() == "ovo1kT7RqrAZwFtgSGEgNfa7nHjeZoK6ykg1GknJEXG",
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = nft_ta.amount == 1,
        associated_token::mint = nft_mint,
        associated_token::authority = owner
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        seeds = [authority.key().as_ref()],
        bump,
        payer = owner,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub launchpad_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = launchpad_mint,
        associated_token::authority = owner,
    )]
    pub launchpad_mint_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = ovo_mint.key().to_string() == "ovo2N3VqRfkZgbb56Gse7oLDXJLJEeqq5z9ePHRxhzL",
    )]
    pub ovo_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = ovo_ta.amount >= 100 * LAMPORTS_PER_SOL, // 100 ovo
        associated_token::mint = launchpad_mint,
        associated_token::authority = owner,
    )]
    pub ovo_ta: Box<Account<'info, TokenAccount>>,
    /// CHECK: Checks done in Metaplex
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: Checks done in Metaplex
    #[account(mut)]
    pub nft_master_edition: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mpl_token_metadata: Program<'info, MplTokenMetadata>,
    /// CHECK: checked by address and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    /// CHECK: Checks done in Metaplex
    pub authorization_rules: UncheckedAccount<'info>,
    /// CHECK: Checks done in Metaplex
    pub authorization_rules_program: UncheckedAccount<'info>,
}

#[account]
pub struct UpdateData {
    pub name: String,
    pub uri: String,
    pub mint_token: bool,
}

#[error_code]
pub enum SpecificErrorCode {
    #[msg("Nft has already been upgraded")]
    AlreadyUpgraded,
}

pub fn upgrade_nft_ix<'info>(
    ctx: Context<'_, '_, '_, 'info, UpgradeNft<'info>>,
    data: UpdateData,
) -> Result<()> {
    let metadata = Metadata::from_account_info(&ctx.accounts.nft_metadata.to_account_info())?;

    if metadata.data.uri == data.uri {
        return Err(SpecificErrorCode::AlreadyUpgraded.into());
    }

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            from: ctx.accounts.ovo_ta.to_account_info(),
            mint: ctx.accounts.ovo_mint.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );
    burn(cpi_ctx, 100 * LAMPORTS_PER_SOL)?; // 100 ovo // 9 decimals

    let cpi_program = ctx.accounts.mpl_token_metadata.to_account_info();

    let cpi_accounts = MetaplexUpdate {
        authority: ctx.accounts.authority.to_account_info(),
        mint: ctx.accounts.nft_mint.to_account_info(),
        metadata: ctx.accounts.nft_metadata.to_account_info(),
        payer: ctx.accounts.owner.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
    };

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

    metaplex_update(cpi_ctx, update_params)?;

    if data.mint_token {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.launchpad_mint.to_account_info(),
                to: ctx.accounts.launchpad_mint_ta.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );
        mint_to(cpi_ctx, 1)?
    }

    Ok(())
}
