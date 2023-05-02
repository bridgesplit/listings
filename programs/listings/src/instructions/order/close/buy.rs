use anchor_lang::Key;
use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
pub struct CloseBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = Order::is_active(order.state),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.as_ref(),
        initializer.key().as_ref()],
        bump,
        close = initializer,
    )]
    pub order: Box<Account<'info, Order>>,
}

pub fn handler(ctx: Context<CloseBuyOrder>) -> ProgramResult {
    msg!("Close buy order account: {}", ctx.accounts.order.key());
    ctx.accounts.order.state = OrderState::Closed.into();

    Order::emit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        OrderEditType::Close,
    );
    Ok(())
}
