use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use bridgesplit_program_utils::{
    compressed_transfer,
    mpl_bubblegum::{cpi::accounts::Transfer, program::Bubblegum},
};
use vault::state::{Appraisal, APPRAISAL_SEED};

use crate::{instructions::compressed::CompressedOrderData, state::*};

#[derive(Accounts)]
#[instruction(data: CompressedOrderData)]
pub struct CompressedInitSellOrder<'info> {
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
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED.as_ref(),
        market.pool_mint.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        constraint = data.price > 0,
        init,
        seeds = [ORDER_SEED.as_ref(),
        data.order_nonce.as_ref(),
        market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Order>()
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        seeds = [APPRAISAL_SEED, market.pool_mint.as_ref(), data.mint_id.as_ref()],
        bump,
        seeds::program = vault::ID,
    )]
    pub appraisal: Box<Account<'info, Appraisal>>,
    /// CHECK: checked in cpi
    pub tree_authority: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    pub compression_program: UncheckedAccount<'info>,
    pub mpl_bubblegum: Program<'info, Bubblegum>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> CompressedInitSellOrder<'info> {
    pub fn transfer_compressed_nft(
        &self,
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
            new_leaf_owner: self.wallet.to_account_info(),
            merkle_tree: self.merkle_tree.to_account_info(),
            log_wrapper: self.log_wrapper.to_account_info(),
            compression_program: self.compression_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let ctx = CpiContext::new(self.mpl_bubblegum.to_account_info(), cpi_accounts);
        compressed_transfer(ctx, root, data_hash, creator_hash, nonce, index)
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CompressedInitSellOrder<'info>>,
    data: CompressedOrderData,
) -> ProgramResult {
    msg!("Initialize a new sell order: {}", ctx.accounts.order.key());

    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.wallet.key(),
        data.order_nonce,
        data.mint_id,
        ctx.accounts.clock.unix_timestamp,
        OrderSide::CompressedSell.into(),
        1, // always 1
        data.price,
        OrderState::Ready.into(),
    );

    ctx.accounts.transfer_compressed_nft(
        data.root,
        data.data_hash,
        data.creator_hash,
        data.nonce,
        data.index,
    )?;

    Order::emit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.pool_mint,
        OrderEditType::Init,
    );

    Ok(())
}
