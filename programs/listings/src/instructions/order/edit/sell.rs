use crate::state::*;
use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use super::EditSellOrderData;

// sell order can only have its price edited

#[derive(Accounts)]
#[instruction(data: EditSellOrderData)]
pub struct EditSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = data.new_price > 0,
        constraint = Order::is_active(order.state),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.key().as_ref(),
        order.owner.as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<EditSellOrder>, data: EditSellOrderData) -> ProgramResult {
    msg!("Edit sell order: {}", ctx.accounts.order.key());
    // update the sell order account
    Order::edit_sell(
        &mut ctx.accounts.order,
        data.new_price,
        ctx.accounts.clock.unix_timestamp,
    );

    Order::emit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        OrderEditType::Edit,
    );
    Ok(())
}
