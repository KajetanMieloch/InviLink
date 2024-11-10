use anchor_lang::prelude::*;

declare_id!("54hULPg3P66Ps3dqtkbYUjAZsUjgnykmoKCvTFovFePv");

#[program]
pub mod hello_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let user_address = ctx.accounts.user.key();
        msg!("Witaj: {:?}", user_address);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
}
