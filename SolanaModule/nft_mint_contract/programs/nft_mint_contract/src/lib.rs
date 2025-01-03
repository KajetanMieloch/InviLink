use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

declare_id!("Hh9NSEH8cZv8Vhq5PhN88CKBndPQnDCzc513V9B1xeZH");

#[program]
pub mod nft_mint_contract {
    use super::*;

    /// Mint tokens to a specific account
    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    /// Transfer tokens between accounts
    pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    /// CHECK: This account is verified by the token program's CPI calls
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: This account is verified by the token program's CPI calls
    #[account(mut)]
    pub to: AccountInfo<'info>,

    /// The signer who pays for the minting
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    /// CHECK: This account is verified by the token program's CPI calls
    #[account(mut)]
    pub from: AccountInfo<'info>,

    /// CHECK: This account is verified by the token program's CPI calls
    #[account(mut)]
    pub to: AccountInfo<'info>,

    /// Signer authorizing the transfer
    pub signer: Signer<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,
}