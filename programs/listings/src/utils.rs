use anchor_lang::{
    prelude::{AccountInfo, CpiContext},
    solana_program::{
        entrypoint::ProgramResult, program::invoke_signed, system_instruction::transfer,
    },
    ToAccountInfo,
};
use bridgesplit_program_utils::{BridgesplitTransfer, bridgesplit_transfer, ExtraTransferParams};

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
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    bridgesplit_transfer(cpi_ctx, transfer_params, 1)
}


#[allow(clippy::too_many_arguments)]
pub fn freeze_nft<'info>(
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
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    bridgesplit_transfer(cpi_ctx, transfer_params, 1)
}

#[allow(clippy::too_many_arguments)]
pub fn unfreeze_nft<'info>(
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
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    bridgesplit_transfer(cpi_ctx, transfer_params, 1)
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
