use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use bridgesplit_program_utils::anchor_lang;
use bridgesplit_program_utils::{
    compressed_transfer,
    mpl_bubblegum::{cpi::accounts::Transfer, program::Bubblegum},
};
use vault::utils::lamport_transfer;

use crate::{instructions::compressed::CompressedFillOrderData, state::*, utils::get_fee_amount};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct CompressedFillBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == buyer.key(),
    )]
    /// CHECK: constraint check
    pub buyer: UncheckedAccount<'info>,
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
    /// CHECK: constraint
    #[account(
        mut,
        constraint = treasury.key().to_string() == PROTOCOL_TREASURY
    )]
    pub treasury: AccountInfo<'info>,
    /// CHECK: checked in cpi
    pub tree_authority: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    pub compression_program: UncheckedAccount<'info>,
    pub mpl_bubblegum: Program<'info, Bubblegum>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> CompressedFillBuyOrder<'info> {
    pub fn transfer_compressed_nft(
        &self,
        ra: Vec<AccountInfo<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            tree_authority: self.tree_authority.to_account_info(),
            leaf_owner: self.initializer.to_account_info(),
            leaf_delegate: self.initializer.to_account_info(),
            new_leaf_owner: self.buyer.to_account_info(),
            merkle_tree: self.merkle_tree.to_account_info(),
            log_wrapper: self.log_wrapper.to_account_info(),
            compression_program: self.compression_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let ctx = CpiContext::new(self.mpl_bubblegum.to_account_info(), cpi_accounts)
            .with_remaining_accounts(ra);
        compressed_transfer(ctx, &[], root, data_hash, creator_hash, nonce, index)
    }
}

/// seller is initializer and is transferring the nft to buyer who is the owner of the order account
/// buyer is the owner of the order account and is transferring sol to seller via bidding wallet
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CompressedFillBuyOrder<'info>>,
    data: CompressedFillOrderData,
) -> ProgramResult {
    // edit wallet account to decrease balance
    msg!("Edit wallet balance: {}", ctx.accounts.wallet.key());
    Wallet::edit_balance(&mut ctx.accounts.wallet, false, ctx.accounts.order.price);

    ctx.accounts.transfer_compressed_nft(
        ctx.remaining_accounts.to_vec(),
        data.root,
        data.data_hash,
        data.creator_hash,
        data.index as u64,
        data.index,
    )?;

    // transfer sol from buyer to seller
    lamport_transfer(
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.order.price,
    )?;

    let fee_amount = get_fee_amount(ctx.accounts.order.price);
    // transfer fee to treasury
    lamport_transfer(
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.treasury.to_account_info(),
        fee_amount,
    )?;

    // edit order
    let price = ctx.accounts.order.price;
    let size = ctx.accounts.order.size;

    Order::edit_buy(
        &mut ctx.accounts.order,
        price,
        size - 1,
        ctx.accounts.clock.unix_timestamp,
    );

    if size == 1 {
        // close order account
        msg!(
            "Close buy order account: {}: {}",
            ctx.accounts.order.key(),
            ctx.accounts.market.pool_mint
        );
        emit_cpi!(Order::get_edit_event(
            &mut ctx.accounts.order.clone(),
            ctx.accounts.order.key(),
            ctx.accounts.market.pool_mint,
            OrderEditType::FillAndClose,
        ));
        ctx.accounts.order.state = OrderState::Closed.into();
        ctx.accounts
            .order
            .close(ctx.accounts.buyer.to_account_info())?;
    } else {
        emit_cpi!(Order::get_edit_event(
            &mut ctx.accounts.order.clone(),
            ctx.accounts.order.key(),
            ctx.accounts.market.pool_mint,
            OrderEditType::Fill,
        ));
        msg!("Filled buy order: {}", ctx.accounts.order.key());
    }

    Ok(())
}
