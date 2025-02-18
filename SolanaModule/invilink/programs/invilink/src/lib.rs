use anchor_lang::prelude::*;

declare_id!("9puQE2ab3LJY9HUFTUe9YfB2yWjTZ56Fx5FxoNHgvThv");

// Stałe globalne
const MASTER_ACCOUNT: Pubkey = pubkey!("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");
const FEE_PERCENTAGE: u64 = 5; // 5% opłaty manipulacyjnej

#[program]
pub mod invilink {
    use super::*;

    // ---------------- Inicjalizacja ----------------

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

    pub fn initialize_event_registry(ctx: Context<InitializeEventRegistry>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.events = Vec::new();
        Ok(())
    }

    // ---------------- Zarządzanie organizatorami ----------------

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

    // ---------------- Zarządzanie eventami ----------------

    pub fn create_event(
        ctx: Context<CreateEvent>,
        event_id: String,
        name: String,
        ticket_price: u64,
        available_tickets: u64,
        seating_type: u8, // 0 = open-space, 1 = numerowane, 2 = mieszane
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
        event.sold_tickets = 0;
        event.seating_type = seating_type;
        event.active = true;
        ctx.accounts.registry.events.push(event.key());
        Ok(())
    }

    pub fn update_event(
        ctx: Context<UpdateEvent>,
        new_name: Option<String>,
        new_ticket_price: Option<u64>,
        new_available_tickets: Option<u64>,
        new_seating_type: Option<u8>,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        require!(event.active, ErrorCode::EventNotActive);
        if let Some(name) = new_name {
            event.name = name;
        }
        if let Some(price) = new_ticket_price {
            event.ticket_price = price;
        }
        if let Some(available) = new_available_tickets {
            require!(available >= event.sold_tickets, ErrorCode::InvalidTicketQuantity);
            event.available_tickets = available;
        }
        if let Some(seating) = new_seating_type {
            event.seating_type = seating;
        }
        Ok(())
    }

    pub fn delete_event(ctx: Context<DeleteEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        event.active = false;
        Ok(())
    }

    // ---------------- Konfiguracja miejsc (seating) ----------------

    pub fn initialize_seating(
        ctx: Context<InitializeSeating>,
        event_id: String,
    ) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        seating_map.event_id = event_id;
        seating_map.sections = Vec::new();
        Ok(())
    }

    pub fn initialize_seating_section(
        ctx: Context<InitializeSeatingSection>,
        section_index: u8,
        section_type: u8,
        rows: u8,
        seats_per_row: u8,
    ) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        let section_account = &mut ctx.accounts.seating_section;
    
        section_account.event_id = seating_map.event_id.clone();
        section_account.section_index = section_index;
        section_account.section_type = section_type;
        section_account.rows = rows;
        section_account.seats_per_row = seats_per_row;
        section_account.seat_status = vec![0; (rows as usize) * (seats_per_row as usize)];
    
        seating_map.sections.push(section_account.key());
    
        Ok(())
    }

    pub fn update_seating_section(
        ctx: Context<UpdateSeatingSection>,
        new_rows: Option<u8>,
        new_seats_per_row: Option<u8>,
        new_section_type: Option<u8>,
    ) -> Result<()> {
        let section = &mut ctx.accounts.seating_section;
        if let Some(new_type) = new_section_type {
            section.section_type = new_type;
        }
        if let Some(rows) = new_rows {
            require!(rows > 0, ErrorCode::InvalidSeating);
            section.rows = rows;
        }
        if let Some(seats) = new_seats_per_row {
            require!(seats > 0, ErrorCode::InvalidSeating);
            section.seats_per_row = seats;
        }
        let total = (section.rows as usize) * (section.seats_per_row as usize);
        section.seat_status = vec![0; total];
        Ok(())
    }
    
    // ---------------- Mintowanie biletu ----------------
    
    // Funkcja mint_ticket tworzy nowe konto biletu (TicketNFT).
    // Dla eventów z seatingiem (typ 1 lub 2) wymaga podania konta SeatingSectionAccount,
    // aby zarezerwować miejsce (oznaczając je jako sprzedane).
    pub fn mint_ticket(
        ctx: Context<MintTicket>,
        ticket_id: String,
        event_id: String,
        section_index: Option<u8>,
        row: Option<u8>,
        seat: Option<u8>,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &mut ctx.accounts.event;
        //Czy event_id przekazany w argumencie odpowiada eventowi.
        require!(event.event_id == event_id, ErrorCode::InvalidTicket);
        
        // Jeśli event wymaga rezerwacji miejsc (numerowane lub mieszane)
        if event.seating_type == 1 || event.seating_type == 2 {
            let seating_section = ctx.accounts.seating_section.as_mut().ok_or(ErrorCode::InvalidSeating)?;
            require!(seating_section.event_id == event.event_id, ErrorCode::InvalidSeating);
            if let Some(sec_index) = section_index {
                require!(seating_section.section_index == sec_index, ErrorCode::InvalidSeating);
            }
            if seating_section.section_type == 1 {
                // Dla sekcji numerowanych wymagamy podania konkretnego rzędu i miejsca.
                let row_val = row.ok_or(ErrorCode::InvalidSeating)?;
                let seat_val = seat.ok_or(ErrorCode::InvalidSeating)?;
                let index = (row_val as usize) * (seating_section.seats_per_row as usize) + (seat_val as usize);
                require!(index < seating_section.seat_status.len(), ErrorCode::InvalidSeating);
                require!(seating_section.seat_status[index] == 0, ErrorCode::SeatAlreadyTaken);
                seating_section.seat_status[index] = 2; // oznacz jako sprzedane
            } else {
                // Dla sekcji stojących – znajdź pierwszy wolny element.
                let pos = seating_section.seat_status.iter().position(|&s| s == 0).ok_or(ErrorCode::SeatNotReserved)?;
                seating_section.seat_status[pos] = 2;
            }
        }
        // Ustaw dane biletu.
        ticket.ticket_id = ticket_id;
        ticket.event_id = event_id;
        ticket.owner = *ctx.accounts.buyer.key;
        ticket.row = row;
        ticket.seat = seat;
        ticket.used = false;
        // Zwiększ liczbę sprzedanych biletów.
        event.sold_tickets = event.sold_tickets.checked_add(1).ok_or(ErrorCode::InvalidTicket)?;
        Ok(())
    }

    // ---------------- Operacje biletowe ----------------

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
        require!(ctx.accounts.owner.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);

        let lamports_to_withdraw = ctx.accounts.fee_pool.total_fees;
        let fee_pool_account_info = ctx.accounts.fee_pool.to_account_info();
        let owner_account_info = ctx.accounts.owner.to_account_info();

        {
            let mut fee_pool_lamports = fee_pool_account_info.try_borrow_mut_lamports()?;
            let mut owner_lamports = owner_account_info.try_borrow_mut_lamports()?;
            **fee_pool_lamports -= lamports_to_withdraw;
            **owner_lamports += lamports_to_withdraw;
        }

        ctx.accounts.fee_pool.total_fees = 0;

        Ok(())
    }
}

// ================= KONTEKSTY (Accounts) =================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 48)]
    pub fee_pool: Account<'info, FeePool>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeOrganizersPool<'info> {
    #[account(init, payer = payer, space = 3212, seeds = [b"organizers_pool"], bump)]
    pub organizers_pool: Account<'info, OrganizersPool>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeEventRegistry<'info> {
    #[account(init, payer = payer, space = 32012)]
    pub registry: Account<'info, EventRegistry>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
pub struct CreateEvent<'info> {
    #[account(init, payer = organizer, space = 350)]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub organizers_pool: Account<'info, OrganizersPool>,
    #[account(mut)]
    pub registry: Account<'info, EventRegistry>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateEvent<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(signer)]
    pub organizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeleteEvent<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(signer)]
    pub organizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeSeating<'info> {
    #[account(init, payer = organizer, space = 3280)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeSeatingSection<'info> {
    #[account(mut)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(init, payer = organizer, space = 10084)]
    pub seating_section: Account<'info, SeatingSectionAccount>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateSeatingSection<'info> {
    #[account(mut)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(mut)]
    pub seating_section: Account<'info, SeatingSectionAccount>,
    #[account(signer)]
    pub organizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintTicket<'info> {
    // Inicjujemy nowe konto biletu – alokujemy 200 bajtów.
    #[account(init, payer = buyer, space = 200)]
    pub ticket: Account<'info, TicketNFT>,
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    // Dla eventów z rezerwacją miejsc (numerowanych/mieszanych)
    #[account(mut)]
    pub seating_section: Option<Account<'info, SeatingSectionAccount>>,
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

// ================= STRUKTURY KONTA =================

#[account]
pub struct FeePool {
    pub owner: Pubkey,
    pub total_fees: u64,
}

#[account]
pub struct OrganizersPool {
    pub organizers: Vec<Pubkey>,
}

#[account]
pub struct EventRegistry {
    pub events: Vec<Pubkey>,
}

#[account]
pub struct EventNFT {
    pub event_id: String,
    pub organizer: Pubkey,
    pub name: String,
    pub ticket_price: u64,
    pub available_tickets: u64,
    pub sold_tickets: u64,
    pub seating_type: u8, // 0 = open-space, 1 = numerowane, 2 = mieszane
    pub active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SeatingSection {
    pub section_type: u8,  // 0 = stojące, 1 = siedzące
    pub rows: u8,
    pub seats_per_row: u8,
    pub seat_status: Vec<u8>, // 0: wolne, 1: zarezerwowane, 2: sprzedane
}

#[account]
pub struct SeatingMap {
    pub event_id: String, 
    pub sections: Vec<Pubkey>,
}

#[account]
pub struct SeatingSectionAccount {
    pub event_id: String,
    pub section_index: u8, 
    pub section_type: u8,  // 0 = stojące, 1 = siedzące
    pub rows: u8,
    pub seats_per_row: u8,
    pub seat_status: Vec<u8>,
}

#[account]
pub struct TicketNFT {
    pub ticket_id: String,
    pub event_id: String,
    pub owner: Pubkey,
    pub row: Option<u8>,
    pub seat: Option<u8>,
    pub used: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SeatingSectionInput {
    pub section_type: u8,
    pub rows: u8,
    pub seats_per_row: u8,
}

// ================= ERROR CODE =================

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
    #[msg("Event is not active.")]
    EventNotActive,
    #[msg("New available tickets cannot be less than the number of sold tickets.")]
    InvalidTicketQuantity,
    #[msg("Cannot update seating configuration after tickets have been sold.")]
    CannotUpdateSeatingAfterSales,
}
