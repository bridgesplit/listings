use crate::state::{Order, Tracker, Wallet};
use anchor_lang::{
    prelude::{msg, Account, AccountInfo},
    solana_program::{
        entrypoint::ProgramResult, program::invoke_signed, system_instruction::transfer,
    },
    Key, ToAccountInfo,
};

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

// !! DO NOT EDIT THE FOLLOWING CODE !! //

pub fn print_webhook_logs_for_order(
    order: Box<Account<'_, Order>>,
    wallet: Box<Account<'_, Wallet>>,
) -> ProgramResult {
    msg!(
        "Edit order account: &{:?}&, new order data is: &{{\"version\": {}, \"nonce\": \"{}\", \"market\": \"{}\", \"owner\": \"{}\", \"wallet\": {{\"version\": {}, \"owner\": \"{}\", \"balance\": {}, \"activeBids\": {}, \"address\": \"{}\"}}, \"side\": {}, \"size\": {}, \"price\": {}, \"state\": {}, \"initTime\": {}, \"lastEditTime\": {}, \"address\": \"{}\"}}",
        order.key(),
        order.version,
        order.nonce,
        order.market,
        order.owner,
        wallet.version,
        wallet.owner,
        wallet.balance,
        wallet.active_bids,
        wallet.key(),
        order.side,
        order.size,
        order.price,
        order.state,
        order.init_time,
        order.last_edit_time,
        order.key(),
    );
    Ok(())
}

pub fn print_webhook_logs_for_wallet(wallet: Box<Account<'_, Wallet>>) -> ProgramResult {
    msg!(
        "Edit wallet account: &{:?}&, new wallet data is: &{{\"version\": {}, \"owner\": \"{}\", \"balance\": {}, \"activeBids\": {}, \"address\": \"{}\"}}",
        wallet.key(),
        wallet.version,
        wallet.owner,
        wallet.balance,
        wallet.active_bids,
        wallet.key(),
    );
    Ok(())
}

pub fn print_webhook_logs_for_tracker(tracker: Box<Account<'_, Tracker>>) -> ProgramResult {
    msg!(
        "Edit tracker account: &{:?}&, new tracker data is: &{{\"version\": {}, \"market\": \"{}\", \"order\": \"{}\", \"owner\": \"{}\", \"nftMint\": \"{}\", \"address\": \"{}\", \"nftMetadata\": null}}",
        tracker.key(),
        tracker.version,
        tracker.market,
        tracker.order,
        tracker.owner,
        tracker.nft_mint,
        tracker.key(),
    );
    Ok(())
}
