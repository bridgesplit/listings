use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::{instructions::order::InitOrderData, state::*};

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
pub struct CompressedInitBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        // make sure bidding wallet has enough balance to place the order
        constraint = wallet.balance >= data.price * data.size,
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
        constraint = data.price > 0 && data.size > 0,
        init,
        seeds = [ORDER_SEED.as_ref(),
        data.nonce.as_ref(),
        market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Order>()
    )]
    pub order: Box<Account<'info, Order>>,
    /// CHECK: can be anything
    pub nft_mint: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<CompressedInitBuyOrder>, data: InitOrderData) -> ProgramResult {
    msg!("Initialize a new buy order: {}", ctx.accounts.order.key());

    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.wallet.key(),
        data.nonce,
        ctx.accounts.nft_mint.key(),
        ctx.accounts.clock.unix_timestamp,
        OrderSide::CompressedBuy.into(),
        data.size,
        data.price,
        OrderState::Ready.into(),
    );

    Order::emit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.pool_mint,
        OrderEditType::Init,
    );

    Ok(())
}
