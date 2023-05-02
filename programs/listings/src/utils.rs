use anchor_lang::{
    prelude::{AccountInfo, CpiContext},
    solana_program::{
        entrypoint::ProgramResult, program::invoke_signed, system_instruction::transfer,
    },
    ToAccountInfo,
};
use bridgesplit_program_utils::{
    bridgesplit_transfer, delegate_and_freeze, pnft::utils::PnftParams, thaw_and_revoke,
    BridgesplitTransfer, DelegateAndFreeze, ExtraDelegateParams, ExtraRevokeParams,
    ExtraTransferParams, ThawAndRevoke,
};

#[allow(clippy::too_many_arguments)]
pub fn transfer_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    to: AccountInfo<'info>,
    nft_mint: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    nft_edition: AccountInfo<'info>,
    from_nft_ta: AccountInfo<'info>,
    to_nft_ta: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    instructions_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    mpl_token_metadata_program: AccountInfo<'info>,
    transfer_params: ExtraTransferParams<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<(), anchor_lang::prelude::Error> {
    let cpi_program = mpl_token_metadata_program.to_account_info();
    let cpi_accounts = BridgesplitTransfer {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        token_owner: authority.to_account_info(),
        token: from_nft_ta.to_account_info(),
        destination_owner: to.to_account_info(),
        destination: to_nft_ta.to_account_info(),
        mint: nft_mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        edition: nft_edition.to_account_info(),
        system_program: system_program.to_account_info(),
        instructions: instructions_program.to_account_info(),
        token_program: token_program.to_account_info(),
        ata_program: associated_token_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridgesplit_transfer(cpi_ctx, transfer_params, 1)
}

#[allow(clippy::too_many_arguments)]
pub fn freeze_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    nft_edition: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    sysvar_instructions: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    mpl_token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    delegate_params: ExtraDelegateParams<'info>,
    freeze_params: PnftParams<'info>,
) -> Result<(), anchor_lang::prelude::Error> {
    let cpi_program = mpl_token_metadata_program.to_account_info();
    let cpi_accounts = DelegateAndFreeze {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        token_owner: authority.to_account_info(),
        mint: mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        edition: nft_edition.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
        ata_program: associated_token_program.to_account_info(),
        token: token.to_account_info(),
        delegate: delegate.to_account_info(),
        mpl_token_metadata_program,
        sysvar_instructions: sysvar_instructions.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    delegate_and_freeze(cpi_ctx, signer_seeds, delegate_params, freeze_params)
}

#[allow(clippy::too_many_arguments)]
pub fn unfreeze_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    nft_edition: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    sysvar_instructions: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    mpl_token_metadata_program: AccountInfo<'info>,
    revoke: bool,
    signer_seeds: &[&[&[u8]]],
    freeze_params: ExtraRevokeParams<'info>,
    delegate_params: PnftParams<'info>,
) -> Result<(), anchor_lang::prelude::Error> {
    let cpi_program = mpl_token_metadata_program.to_account_info();
    let cpi_accounts = ThawAndRevoke {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        token_owner: authority.to_account_info(),
        token: token.to_account_info(),
        mint: mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        edition: nft_edition.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
        ata_program: associated_token_program.to_account_info(),
        delegate: delegate.to_account_info(),
        mpl_token_metadata_program,
        sysvar_instructions: sysvar_instructions.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    thaw_and_revoke(
        cpi_ctx,
        signer_seeds,
        revoke,
        delegate_params,
        freeze_params,
    )
}

/// transfer sol
/// amount in lamports
pub fn transfer_sol<'info>(
    from_account: AccountInfo<'info>,
    to_account: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]; 1],
    amount: u64,
) -> ProgramResult {
    invoke_signed(
        &transfer(from_account.key, to_account.key, amount),
        &[
            from_account.to_account_info(),
            to_account.to_account_info(),
            system_program.to_account_info(),
        ],
        signer_seeds,
    )
    .map_err(Into::into)
}

pub fn get_pnft_params(ra: Vec<AccountInfo>) -> PnftParams {
    PnftParams {
        token_record: ra.get(0).cloned(),
        authorization_rules: ra.get(1).cloned(),
        authorization_rules_program: ra.get(2).cloned(),
        authorization_data: None,
    }
}
