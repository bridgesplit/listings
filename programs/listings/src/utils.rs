use anchor_lang::{
    prelude::{Account, AccountInfo, CpiContext, Error, Pubkey},
    solana_program::{
        entrypoint::ProgramResult, program::invoke_signed, system_instruction::transfer,
    },
    AccountDeserialize, ToAccountInfo,
};
use anchor_spl::token::TokenAccount;
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};

use crate::state::{Order, PROTOCOL_FEES_BPS};
use bridgesplit_program_utils::{
    bridgesplit_transfer, delegate_and_freeze, pnft::utils::PnftParams, thaw_and_revoke,
    BridgesplitTransfer, DelegateAndFreeze, ExtraDelegateParams, ExtraRevokeParams,
    ExtraTransferParams, ThawAndRevoke
};

#[allow(clippy::too_many_arguments)]
pub fn transfer_nft<'info>(
    authority: AccountInfo<'info>,
    token_owner: AccountInfo<'info>,
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
        token_owner: token_owner.to_account_info(),
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
    is_pnft: bool,
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
    delegate_and_freeze(
        cpi_ctx,
        is_pnft,
        signer_seeds,
        delegate_params,
        freeze_params,
    )
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
) -> Result<(), Error> {
    let cpi_program = mpl_token_metadata_program.to_account_info();
    let cpi_accounts = ThawAndRevoke {
        authority: delegate.to_account_info(),
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

fn get_pnft_params(ra: Vec<AccountInfo>) -> PnftParams {
    PnftParams {
        token_record: ra.get(0).cloned(),
        authorization_rules: ra.get(1).cloned(),
        authorization_rules_program: ra.get(2).cloned(),
        authorization_data: None,
    }
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


pub fn check_ovol_holder(remaining_accounts: Vec<AccountInfo>, owner: Pubkey) -> bool {
    let ovol_nft_ta_account_info = remaining_accounts.get(0);
    let ovol_nft_metadata_account_info = remaining_accounts.get(1);

    if let Some(ovol_nft_ta) = ovol_nft_ta_account_info {
        if let Some(ovol_nft_metadata) = ovol_nft_metadata_account_info {
            if let Ok(nft_ta) =
                TokenAccount::try_deserialize(&mut &ovol_nft_ta.data.borrow_mut()[..])
            {
                if let Ok(metadata) =
                    Metadata::safe_deserialize(&ovol_nft_metadata.data.borrow_mut()[..])
                {
                    if let Some(collection) = metadata.collection {
                        if (nft_ta.amount == 1)
                            && (nft_ta.owner == owner)
                            && collection.verified
                            && collection.key.to_string()
                                == "9jnJWH9F9t1xAgw5RGwswVKY4GvY2RXhzLSJgpBAhoaR"
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// result of parsing remaining accounts
pub struct ParsedRemainingAccounts<'info> {
    //params for pnft ix's
    pub pnft_params: PnftParams<'info>,
    // apply fee on listings
    pub fees_on: bool,
}


fn parse_pnft_accounts(remaining_accounts: Vec<AccountInfo>) -> PnftParams {

    let account_0 = remaining_accounts.get(0).unwrap();

    if account_0.key == &Pubkey::default() {
        return PnftParams {
            authorization_data: None,
            authorization_rules: None,
            authorization_rules_program: None,
            token_record: None
        }

    } else {
        return get_pnft_params(remaining_accounts);
    }
   
}

pub fn parse_remaining_accounts(
    remaining_accounts: Vec<AccountInfo>,
    initializer: Pubkey,
) -> ParsedRemainingAccounts {
    //first 3 are either default pubkeys or pnft accounts
    let pnft_params  = parse_pnft_accounts(remaining_accounts.clone());
    // last 2 either don't exist or are ovol accounts
    let fees_on = check_ovol_holder(remaining_accounts[3..].to_vec(), initializer);
    ParsedRemainingAccounts {
        pnft_params,
        fees_on,
    }
}

pub fn get_fees_on(order: Box<Account<'_, Order>>, ovol_fees_on: bool) -> bool {
    order.fees_on && ovol_fees_on
}

pub fn get_fee_amount(order_price: u64) -> u64 {
    (order_price.checked_mul(PROTOCOL_FEES_BPS))
        .unwrap()
        .checked_div(10000)
        .unwrap()
}
