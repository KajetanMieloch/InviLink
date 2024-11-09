use anchor_lang::prelude::*;

declare_id!("EEnUgo8XJYHQkQ14biGpu21KCtESb3h5AJ4iFFJsQHVA");

#[program]
pub mod solana_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
