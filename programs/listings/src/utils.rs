use anchor_lang::{
    prelude::{AccountInfo, CpiContext},
    solana_program::{
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
    AnchorDeserialize, ToAccountInfo,
};
use anchor_mpl_token_metadata::transfer::{metaplex_transfer, MetaplexTransfer, TransferParams};
use anchor_spl::token::{self, Approve, Transfer};
use mpl_token_metadata::{
    instruction::{freeze_delegated_account, thaw_delegated_account},
    state::{Metadata, TokenStandard},
};
use vault::utils::get_metaplex_transfer_params;

fn create_freeze_nft_account_infos<'info>(
    nft_mint: AccountInfo<'info>,
    nft_master_edition: AccountInfo<'info>,
    nft_ta: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
) -> Vec<AccountInfo<'info>> {
    let freeze_nft_account_infos: Vec<AccountInfo<'info>> = vec![
        delegate.to_account_info(),
        nft_ta.to_account_info(),
        nft_master_edition.to_account_info(),
        nft_mint.to_account_info(),
        token_program.to_account_info(),
    ];
    freeze_nft_account_infos
}

fn create_thaw_nft_account_infos<'info>(
    nft_mint: AccountInfo<'info>,
    nft_master_edition: AccountInfo<'info>,
    nft_ta: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
) -> Vec<AccountInfo<'info>> {
    let freeze_nft_account_infos: Vec<AccountInfo<'info>> = vec![
        delegate.to_account_info(),
        nft_ta.to_account_info(),
        nft_master_edition.to_account_info(),
        nft_mint.to_account_info(),
        token_program.to_account_info(),
    ];
    freeze_nft_account_infos
}

/// freeze an nft
#[allow(clippy::too_many_arguments)]
pub fn freeze_nft<'info>(
    nft_mint: AccountInfo<'info>,
    nft_master_edition: AccountInfo<'info>,
    nft_ta: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    mpl_token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]; 1],
) -> ProgramResult {
    // delegate nft
    let cpi_accounts = Approve {
        to: nft_ta.to_account_info(),
        delegate: delegate.to_account_info(),
        authority: authority.to_account_info(),
    };
    let delegate_nft = CpiContext::new(token_program.to_account_info(), cpi_accounts);

    token::approve(delegate_nft, 1)?;

    // freeze nft
    let freeze_delegated_account_ix = freeze_delegated_account(
        *mpl_token_metadata_program.key,
        *delegate.key,
        *nft_ta.key,
        *nft_master_edition.key,
        *nft_mint.key,
    );

    invoke_signed(
        &freeze_delegated_account_ix,
        &create_freeze_nft_account_infos(
            nft_mint,
            nft_master_edition,
            nft_ta,
            delegate,
            token_program,
        ),
        signer_seeds,
    )?;

    Ok(())
}

/// unfreeze an nft
pub fn unfreeze_nft<'info>(
    nft_mint: AccountInfo<'info>,
    nft_master_edition: AccountInfo<'info>,
    nft_ta: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    mpl_token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]; 1],
) -> ProgramResult {
    let thaw_delegated_account_ix = thaw_delegated_account(
        *mpl_token_metadata_program.key,
        *delegate.key,
        *nft_ta.key,
        *nft_master_edition.key,
        *nft_mint.key,
    );

    invoke_signed(
        &thaw_delegated_account_ix,
        &create_thaw_nft_account_infos(
            nft_mint,
            nft_master_edition,
            nft_ta,
            delegate,
            token_program,
        ),
        signer_seeds,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn transfer_nft_metaplex<'info>(
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
    transfer_params: TransferParams<'info>,
) -> Result<(), anchor_lang::prelude::Error> {
    let cpi_program = mpl_token_metadata_program.to_account_info();
    let cpi_accounts = MetaplexTransfer {
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
    metaplex_transfer(cpi_ctx, transfer_params)
}

fn transfer_nft_token<'info>(
    from_nft_ta: AccountInfo<'info>,
    to_nft_ta: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
) -> CpiContext<'info, 'info, 'info, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: from_nft_ta.to_account_info(),
        to: to_nft_ta.to_account_info(),
        authority: authority.to_account_info(),
    };
    CpiContext::new(token_program.to_account_info(), cpi_accounts)
}

/// transfer an nft
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
    remaining_accounts: Vec<AccountInfo<'info>>,
    signer_seeds: &[&[&[u8]]; 1],
) -> Result<(), anchor_lang::prelude::Error> {
    let transfer_params = get_metaplex_transfer_params(remaining_accounts, None);
    let metadata = Metadata::deserialize(&mut &nft_metadata.data.borrow_mut()[..])?;
    if let Some(token_standard) = metadata.token_standard {
        if token_standard == TokenStandard::ProgrammableNonFungible {
            // do metaplex transfer because the nft is a pnft
            return transfer_nft_metaplex(
                authority,
                payer,
                to,
                nft_mint,
                nft_metadata,
                nft_edition,
                from_nft_ta,
                to_nft_ta,
                system_program,
                instructions_program,
                token_program,
                associated_token_program,
                mpl_token_metadata_program,
                transfer_params,
            );
        }
    }

    // do a normal token transfer in case the nft is not a pnft
    token::transfer(
        transfer_nft_token(from_nft_ta, to_nft_ta, authority, token_program)
            .with_signer(signer_seeds),
        1,
    )
}

/// transfer sol
/// amount in lamports
pub fn transfer_sol<'info>(
    from_account: AccountInfo<'info>,
    to_account: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    amount: u64,
) -> ProgramResult {
    invoke(
        &transfer(from_account.key, to_account.key, amount),
        &[
            from_account.to_account_info(),
            to_account.to_account_info(),
            system_program.to_account_info(),
        ],
    )
    .map_err(Into::into)
}
