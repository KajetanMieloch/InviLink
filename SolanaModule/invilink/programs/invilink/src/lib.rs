use anchor_lang::prelude::*;

declare_id!("DqZf5dE14GCM541qRBNipykFFHDMe2DKxshWk2Q4McMU");

const MASTER_ACCOUNT: Pubkey = pubkey!("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");
const FEE_PERCENTAGE: u64 = 5; // 5% opłaty manipulacyjnej

#[program]
pub mod invilink {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let fee_pool = &mut ctx.accounts.fee_pool;
        fee_pool.owner = MASTER_ACCOUNT;
        fee_pool.total_fees = 0;
        Ok(())
    }

    pub fn initialize_organizers_pool(ctx: Context<InitializeOrganizersPool>) -> Result<()> {
        let organizers_pool = &mut ctx.accounts.organizers_pool;
        organizers_pool.organizers = Vec::new();
        Ok(())
    }

    pub fn add_organizer(ctx: Context<AddOrganizer>, new_organizer: Pubkey) -> Result<()> {
        let organizers = &mut ctx.accounts.organizers_pool;
        require!(ctx.accounts.signer.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);
        require!(!organizers.organizers.contains(&new_organizer), ErrorCode::AlreadyRegistered);
        organizers.organizers.push(new_organizer);
        Ok(())
    }

    pub fn remove_organizer(ctx: Context<RemoveOrganizer>, organizer_to_remove: Pubkey) -> Result<()> {
        let organizers = &mut ctx.accounts.organizers_pool;
        require!(ctx.accounts.signer.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);
        let index = organizers.organizers.iter().position(|x| *x == organizer_to_remove);
        require!(index.is_some(), ErrorCode::OrganizerNotFound);
        organizers.organizers.remove(index.unwrap());
        Ok(())
    }

    pub fn create_event(
        ctx: Context<CreateEvent>,
        event_id: String,
        name: String,
        ticket_price: u64,
        available_tickets: u64,
        seating_type: u8,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(
            ctx.accounts.organizers_pool.organizers.contains(ctx.accounts.organizer.key),
            ErrorCode::Unauthorized
        );
        event.event_id = event_id;
        event.organizer = *ctx.accounts.organizer.key;
        event.name = name;
        event.ticket_price = ticket_price;
        event.available_tickets = available_tickets;
        event.seating_type = seating_type;
        Ok(())
    }

    pub fn initialize_seating(
        ctx: Context<InitializeSeating>,
        event_id: String,
        rows: u8,
        seats_per_row: u8,
    ) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        require!(rows > 0 && seats_per_row > 0, ErrorCode::InvalidSeating);
        seating_map.event_id = event_id;
        seating_map.rows = rows;
        seating_map.seats_per_row = seats_per_row;
        seating_map.seat_status = vec![0; (rows as usize) * (seats_per_row as usize)];
        Ok(())
    }

    pub fn reserve_seat(
        ctx: Context<ReserveSeat>,
        _event_id: String,
        row: u8,
        seat: u8,
    ) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        let index = (row as usize) * (seating_map.seats_per_row as usize) + (seat as usize);
        require!(seating_map.seat_status[index] == 0, ErrorCode::SeatAlreadyTaken);
        seating_map.seat_status[index] = 1; // 1 = zarezerwowane
        Ok(())
    }

    pub fn release_expired_seats(ctx: Context<ReleaseSeats>, _event_id: String) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        for seat in seating_map.seat_status.iter_mut() {
            if *seat == 1 {
                *seat = 0; // zwolnienie rezerwacji
            }
        }
        Ok(())
    }

    pub fn mint_ticket(
        ctx: Context<MintTicket>,
        ticket_id: String,
        event_id: String,
        row: Option<u8>,
        seat: Option<u8>,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &ctx.accounts.event;

        // Jeśli wydarzenie ma miejsca numerowane, sprawdzamy, czy miejsce jest zarezerwowane
        if event.seating_type == 1 || event.seating_type == 2 {
            // Używamy as_mut(), aby uzyskać mutable referencję do konta seating_map
            let seating_map = ctx.accounts.seating_map.as_mut().unwrap();
            let index = (row.unwrap() as usize) * (seating_map.seats_per_row as usize) + (seat.unwrap() as usize);
            require!(seating_map.seat_status[index] == 1, ErrorCode::SeatNotReserved);
            seating_map.seat_status[index] = 2; // oznacz jako sprzedane
        }

        ticket.ticket_id = ticket_id;
        ticket.event_id = event_id;
        ticket.owner = *ctx.accounts.buyer.key;
        ticket.row = row;
        ticket.seat = seat;
        ticket.used = false;

        Ok(())
    }

    pub fn sell_ticket(ctx: Context<SellTicket>, sale_price: u64) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.owner == *ctx.accounts.seller.key, ErrorCode::Unauthorized);

        let fee = sale_price * FEE_PERCENTAGE / 100;
        let seller_revenue = sale_price - fee;

        let buyer_account_info = ctx.accounts.buyer.to_account_info();
        let mut buyer_lamports = buyer_account_info.try_borrow_mut_lamports()?;
        require!(**buyer_lamports >= sale_price, ErrorCode::InsufficientFunds);
        **buyer_lamports -= sale_price;

        let fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
        **fee_pool_lamports += fee;
        ctx.accounts.fee_pool.total_fees += fee;

        let seller_account_info = ctx.accounts.seller.to_account_info();
        let mut seller_lamports = seller_account_info.try_borrow_mut_lamports()?;
        **seller_lamports += seller_revenue;

        ticket.owner = *ctx.accounts.buyer.key;

        Ok(())
    }

    // Dodajemy parametr ticket_price, aby obliczyć opłatę transferową
    pub fn transfer_ticket(ctx: Context<TransferTicket>, new_owner: Pubkey, ticket_price: u64) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.owner == *ctx.accounts.current_owner.key, ErrorCode::Unauthorized);

        let fee = ticket_price * FEE_PERCENTAGE / 100;

        let owner_account_info = ctx.accounts.current_owner.to_account_info();
        let mut owner_lamports = owner_account_info.try_borrow_mut_lamports()?;
        require!(**owner_lamports >= fee, ErrorCode::InsufficientFunds);
        **owner_lamports -= fee;

        let fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
        **fee_pool_lamports += fee;
        ctx.accounts.fee_pool.total_fees += fee;

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

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        // Sprawdzamy, czy wywołujący to MASTER_ACCOUNT
        require!(ctx.accounts.owner.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);

        let lamports_to_withdraw = ctx.accounts.fee_pool.total_fees;
        // Pobieramy AccountInfo przed modyfikacją
        let fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let owner_account_info = ctx.accounts.owner.to_account_info();

        {
            let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
            let mut owner_lamports = owner_account_info.try_borrow_mut_lamports()?;
            **fee_pool_lamports -= lamports_to_withdraw;
            **owner_lamports += lamports_to_withdraw;
        }

        // Po zakończeniu operacji na lamportach możemy zaktualizować stan konta
        ctx.accounts.fee_pool.total_fees = 0;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8)]
    pub fee_pool: Account<'info, FeePool>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeOrganizersPool<'info> {
    #[account(init, payer = payer, space = 8 + (32 * 100), seeds = [b"organizers_pool"], bump)]
    pub organizers_pool: Account<'info, OrganizersPool>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
pub struct FeePool {
    pub owner: Pubkey,   // Master Account
    pub total_fees: u64, // Zgromadzone opłaty
}

#[account]
pub struct OrganizersPool {
    pub organizers: Vec<Pubkey>,
}

#[derive(Accounts)]
pub struct AddOrganizer<'info> {
    #[account(mut)]
    pub organizers_pool: Account<'info, OrganizersPool>,
    #[account(signer)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveOrganizer<'info> {
    #[account(mut)]
    pub organizers_pool: Account<'info, OrganizersPool>,
    #[account(signer)]
    pub signer: Signer<'info>,
}

#[account]
pub struct EventNFT {
    pub event_id: String,
    pub organizer: Pubkey,
    pub name: String,
    pub ticket_price: u64,
    pub available_tickets: u64,
    pub seating_type: u8, // 0 = open-space, 1 = numerowane, 2 = mieszane
}

#[derive(Accounts)]
pub struct CreateEvent<'info> {
    #[account(init, payer = organizer, space = 300)]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub organizers_pool: Account<'info, OrganizersPool>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct SeatingMap {
    pub event_id: String,
    pub rows: u8,
    pub seats_per_row: u8,
    pub seat_status: Vec<u8>, // 0: wolne, 1: zarezerwowane, 2: sprzedane
}

#[derive(Accounts)]
pub struct InitializeSeating<'info> {
    #[account(init, payer = organizer, space = 8 + 64 + 2 + 2 + 2000)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReserveSeat<'info> {
    #[account(mut)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReleaseSeats<'info> {
    #[account(mut)]
    pub seating_map: Account<'info, SeatingMap>,
}

#[account]
pub struct TicketNFT {
    pub ticket_id: String,
    pub event_id: String,
    pub owner: Pubkey,
    pub row: Option<u8>,  // Jeśli dotyczy miejsc numerowanych
    pub seat: Option<u8>, // Jeśli dotyczy miejsc numerowanych
    pub used: bool,       // Czy bilet został użyty?
}

#[derive(Accounts)]
pub struct MintTicket<'info> {
    #[account(init, payer = buyer, space = 200)]
    pub ticket: Account<'info, TicketNFT>,
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub seating_map: Option<Account<'info, SeatingMap>>, // Opcjonalne, gdy miejsca numerowane
    #[account(mut)]
    pub buyer: Signer<'info>,
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
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub fee_pool: Account<'info, FeePool>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized operation.")]
    Unauthorized,
    #[msg("This organizer is already registered.")]
    AlreadyRegistered,
    #[msg("This organizer is not found in the list.")]
    OrganizerNotFound,
    #[msg("Invalid seating type.")]
    InvalidSeatingType,
    #[msg("This seat was not reserved.")]
    SeatNotReserved,
    #[msg("Insufficient funds.")]
    InsufficientFunds,
    #[msg("Invalid ticket ID.")]
    InvalidTicket,
    #[msg("Ticket has already been used.")]
    TicketAlreadyUsed,
    #[msg("Only the administrator can withdraw fees.")]
    AdminOnly,
    #[msg("Invalid seating configuration.")]
    InvalidSeating,
    #[msg("This seat is already taken.")]
    SeatAlreadyTaken,
}
