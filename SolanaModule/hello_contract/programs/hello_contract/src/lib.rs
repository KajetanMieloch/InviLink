use anchor_lang::prelude::*;

declare_id!("5iHT8dpa6TJssXi3VXGAa1W7nVzGxT18dj6PocxW81m9");

#[program]
pub mod hello_contract {
    use super::*;

    pub fn say_hello(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn create_pda(ctx: Context<CreatePDA>) -> Result<()> {
        msg!("PDA created: {:?}", ctx.accounts.pda.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct CreatePDA<'info> {
    #[account(
        init, // Tworzymy nowe konto
        payer = user, // Użytkownik płaci za transakcję
        seeds = [b"my-pda".as_ref()], // Unikalny seed do generowania PDA
        bump, // Automatyczne zarządzanie "bump" w PDA
        space = 8 + 32 // Przestrzeń na dane (8 bajtów dla "discriminator" + 32 bajty na klucz publiczny)
    )]
    /// CHECK: This PDA is derived using a deterministic seed and is safe because the program enforces it.
    pub pda: AccountInfo<'info>, // Konto PDA
    #[account(mut)]
    pub user: Signer<'info>, // Konto użytkownika, które płaci za transakcję
    pub system_program: Program<'info, System>, // Program systemowy Solany
}
