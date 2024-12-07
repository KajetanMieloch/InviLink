use anchor_lang::prelude::*;

declare_id!("4TVmiAwUHSg8muywpq4v2rd4fGrE28KeeHbVC8o8GHAC");

#[program]
pub mod hello_contract {
    use super::*;

    pub fn say_hello(ctx: Context<SayHello>) -> Result<()> {
        // Pobieramy klucz publiczny nadawcy
        let sender = ctx.accounts.sender.key();
        // Wyświetlamy wiadomość w logach
        msg!("Hello {}", sender);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SayHello<'info> {
    #[account(mut)]
    pub sender: Signer<'info>, // Wymagamy podpisu od nadawcy
}
