use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

use crate::{state::*, utils::transfer_sol};

use super::EditOrderData;

#[derive(Accounts)]
#[instruction(data: EditOrderData)]
pub struct EditBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        // cannot increase size of order if it is already filled/cancelled
        constraint = Order::validate_edit_side(data.side, order.state),
        seeds = [ORDER_SEED.as_ref(),
        order.nonce.as_ref(),
        order.market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EditBuyOrder>, data: EditOrderData) -> ProgramResult {
    msg!("Edit buy order");

    // edit the order with size
    Order::edit(&mut ctx.accounts.order, data.price, data.side);

    if data.side.is_increase() {
        // transfer sol from owner to order account if size is increased
        transfer_sol(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.order.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.order.price,
        )?;
    } else {
        // transfer sol from order account to owner if size is decreased
        transfer_sol(
            ctx.accounts.order.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.order.price,
        )?;
    }
    Ok(())
}
