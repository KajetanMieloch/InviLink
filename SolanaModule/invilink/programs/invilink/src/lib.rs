use anchor_lang::prelude::*;

declare_id!("98cFPPr2S2UjSthTxmjNsPei9Ty6vFcsLkyWLsTz4CTY");

const FEE_PERCENTAGE: u64 = 5; // Stała dla opłat manipulacyjnych (5%).

#[program]
pub mod invilink {
    use super::*;

    pub fn mint_ticket(
        ctx: Context<MintTicket>,
        event_id: String,
        ticket_id: String,
        price: u64,
        attributes: String,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        ticket.owner = *ctx.accounts.owner.key;
        ticket.event_id = event_id;
        ticket.ticket_id = ticket_id;
        ticket.price = price;
        ticket.used = false;
        ticket.attributes = attributes;
        Ok(())
    }

    pub fn sell_ticket(ctx: Context<SellTicket>, sale_price: u64) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.owner == *ctx.accounts.seller.key, ErrorCode::Unauthorized);

        let fee = sale_price * FEE_PERCENTAGE / 100;
        let seller_revenue = sale_price - fee;

        // Pobierz lamports kupującego
        let mut buyer_account_info = ctx.accounts.buyer.to_account_info();
        let mut buyer_lamports = buyer_account_info.try_borrow_mut_lamports()?;
        require!(**buyer_lamports >= sale_price, ErrorCode::InsufficientFunds);
        **buyer_lamports -= sale_price;

        // Aktualizuj FeePool
        let mut fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
        **fee_pool_lamports += fee;
        let fee_pool = &mut ctx.accounts.fee_pool;
        fee_pool.total_fees += fee;

        // Przekaż pozostałe środki sprzedawcy
        let mut seller_account_info = ctx.accounts.seller.to_account_info();
        let mut seller_lamports = seller_account_info.try_borrow_mut_lamports()?;
        **seller_lamports += seller_revenue;

        ticket.owner = *ctx.accounts.buyer.key;
        ticket.price = sale_price;

        Ok(())
    }

    pub fn transfer_ticket(ctx: Context<TransferTicket>, new_owner: Pubkey) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.owner == *ctx.accounts.current_owner.key, ErrorCode::Unauthorized);

        let fee = ticket.price * FEE_PERCENTAGE / 100;

        let mut owner_account_info = ctx.accounts.current_owner.to_account_info();
        let mut owner_lamports = owner_account_info.try_borrow_mut_lamports()?;
        require!(**owner_lamports >= fee, ErrorCode::InsufficientFunds);
        **owner_lamports -= fee;

        // Aktualizuj FeePool
        let mut fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
        **fee_pool_lamports += fee;
        let fee_pool = &mut ctx.accounts.fee_pool;
        fee_pool.total_fees += fee;

        ticket.owner = new_owner;

        Ok(())
    }

    pub fn validate_ticket(ctx: Context<ValidateTicket>, ticket_id: String) -> Result<()> {
        let ticket = &ctx.accounts.ticket;
        require!(ticket.ticket_id == ticket_id, ErrorCode::InvalidTicket);
        require!(!ticket.used, ErrorCode::TicketAlreadyUsed);
        Ok(())
    }

    pub fn mark_ticket_used(ctx: Context<MarkTicketUsed>, ticket_id: String) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.ticket_id == ticket_id, ErrorCode::InvalidTicket);
        require!(!ticket.used, ErrorCode::TicketAlreadyUsed);
        ticket.used = true;
        Ok(())
    }

    pub fn initialize_fee_pool(ctx: Context<InitializeFeePool>, owner: Pubkey) -> Result<()> {
        let fee_pool = &mut ctx.accounts.fee_pool;
        fee_pool.owner = owner;
        fee_pool.total_fees = 0;
        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        // Skopiowanie wartości `total_fees` do zmiennej lokalnej, aby uniknąć jednoczesnego borrow
        let lamports_to_withdraw = ctx.accounts.fee_pool.total_fees;
    
        // Tworzymy lokalne kopie AccountInfo przed ich modyfikacją
        let fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let owner_account_info = ctx.accounts.owner.to_account_info();
    
        // Mutowalne borrow do lamports
        let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
        let mut owner_lamports = owner_account_info.try_borrow_mut_lamports()?;
    
        // Aktualizujemy lamports
        **fee_pool_lamports -= lamports_to_withdraw;
        **owner_lamports += lamports_to_withdraw;
    
        // Następnie aktualizujemy pole `total_fees`
        let fee_pool = &mut ctx.accounts.fee_pool;
        fee_pool.total_fees = 0;
    
        Ok(())
    }
    
    
}

#[derive(Accounts)]
pub struct MintTicket<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 64 + 64 + 64 + 1)]
    pub ticket: Account<'info, TicketNFT>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellTicket<'info> {
    #[account(mut)]
    pub ticket: Account<'info, TicketNFT>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub fee_pool: Account<'info, FeePool>,
}

#[derive(Accounts)]
pub struct TransferTicket<'info> {
    #[account(mut)]
    pub ticket: Account<'info, TicketNFT>,
    #[account(mut)]
    pub current_owner: Signer<'info>,
    #[account(mut)]
    pub fee_pool: Account<'info, FeePool>,
}

#[derive(Accounts)]
pub struct ValidateTicket<'info> {
    #[account()]
    pub ticket: Account<'info, TicketNFT>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct MarkTicketUsed<'info> {
    #[account(mut)]
    pub ticket: Account<'info, TicketNFT>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeFeePool<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8)]
    pub fee_pool: Account<'info, FeePool>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub fee_pool: Account<'info, FeePool>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[account]
pub struct TicketNFT {
    pub owner: Pubkey,
    pub event_id: String,
    pub ticket_id: String,
    pub price: u64,
    pub used: bool,
    pub attributes: String,
}

#[account]
pub struct FeePool {
    pub total_fees: u64, // Suma zgromadzonych opłat manipulacyjnych
    pub owner: Pubkey,   // Właściciel puli opłat
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized operation.")]
    Unauthorized,
    #[msg("Insufficient funds.")]
    InsufficientFunds,
    #[msg("Invalid ticket ID.")]
    InvalidTicket,
    #[msg("Ticket has already been used.")]
    TicketAlreadyUsed,
}
