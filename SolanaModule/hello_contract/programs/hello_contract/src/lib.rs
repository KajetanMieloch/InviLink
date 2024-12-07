use anchor_lang::prelude::*;

declare_id!("5iHT8dpa6TJssXi3VXGAa1W7nVzGxT18dj6PocxW81m9");

#[program]
pub mod hello_contract {
    use super::*;

    pub fn handle_payment(ctx: Context<HandlePayment>) -> Result<()> {
        let user = ctx.accounts.user.key();
        let lamports = **ctx.accounts.user.lamports.borrow();
        msg!("Payment received from user: {:?}", user);
        msg!("User balance after payment: {}", lamports);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct HandlePayment<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // Konto użytkownika wysyłającego SOL
    pub system_program: Program<'info, System>, // Program systemowy Solany
}
