use anchor_lang::prelude::*;
use solana_program::hash::hash;
use anchor_lang::__private::base64;

declare_id!("EW1pWhJkpreYMn2FpYC3fZzqvLCgRdr79dh6E8SL7qwW");

// Stałe globalne
const MASTER_ACCOUNT: Pubkey = pubkey!("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");
const FEE_PERCENTAGE: u64 = 5; // 5% opłaty manipulacyjnej

// ---------------- FUNKCJE POZA CHAINEM ----------------

fn generate_event_id(name: &str, organizer: &Pubkey) -> String {
    let seed = b"339562";
    let mut data = Vec::new();
    data.extend_from_slice(seed);
    data.extend_from_slice(name.as_bytes());
    data.extend_from_slice(&organizer.to_bytes());
    let hash_result = hash(&data);
    let encoded = base64::encode(hash_result);
    encoded.chars().take(12).collect()
}

// ---------------- PROGRAM NA CHAINIE ----------------

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
        registry.event_count = 0;
        registry.events = [Pubkey::default(); 10];
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

    #[derive(Accounts)]
    #[instruction(event_id: String, name: String, ticket_price: u64, available_tickets: u64)]
    pub struct CreateEventOpen<'info> {
        #[account(
            init,
            payer = organizer,
            seeds = [b"event", event_id.as_bytes()],
            bump,
            space = 1024
        )]
        pub event: Account<'info, EventNFT>,
        #[account(mut)]
        pub organizers_pool: Account<'info, OrganizersPool>,
        #[account(mut)]
        pub registry: Account<'info, EventRegistry>,
        #[account(mut)]
        pub event_dictionary: Account<'info, EventDictionary>,
        #[account(mut)]
        pub organizer: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
    
    pub fn create_event_open(
        ctx: Context<CreateEventOpen>,
        event_id: String,
        name: String,
        ticket_price: u64,
        available_tickets: u64,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        let registry = &mut ctx.accounts.registry;
        let dict = &mut ctx.accounts.event_dictionary;

        require!(
            ctx.accounts.organizers_pool.organizers.contains(ctx.accounts.organizer.key),
            ErrorCode::Unauthorized
        );

        let expected_event_id = generate_event_id(&name, ctx.accounts.organizer.key);
        require!(event_id == expected_event_id, ErrorCode::InvalidEventId);

        event.event_id = event_id.clone();
        event.organizer = *ctx.accounts.organizer.key;
        event.name = name;
        event.ticket_price = ticket_price;
        event.available_tickets = available_tickets;
        event.sold_tickets = 0;
        // Teraz wszystkie eventy traktujemy jako numerowane:
        event.seating_type = 1;
        event.active = false;
            
        let count = registry.event_count as usize;
        require!(count < 10, ErrorCode::RegistryFull);
        registry.events[count] = event.key();
        registry.event_count += 1;
            
        let mapping = EventMapping {
            event_id: event.event_id.clone(),
            event_pda: event.key(),
        };
        dict.events.push(mapping);
            
        Ok(())
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone)]
    pub struct EventMapping {
        pub event_id: String,
        pub event_pda: Pubkey,
    }

    #[account]
    pub struct EventDictionary {
        pub events: Vec<EventMapping>,
    }

    #[derive(Accounts)]
    pub struct InitializeEventDictionary<'info> {
        #[account(
            init,
            payer = payer,
            space = 9000,
            seeds = [b"event_dictionary"],
            bump,
        )]
        pub event_dictionary: Account<'info, EventDictionary>,
        #[account(mut)]
        pub payer: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    pub fn initialize_event_dictionary(ctx: Context<InitializeEventDictionary>) -> Result<()> {
        let dict = &mut ctx.accounts.event_dictionary;
        dict.events = Vec::new();
        Ok(())
    }

    // Funkcja tworząca event z seating mapą – już nie przyjmuje parametru seating_type
    #[derive(Accounts)]
    #[instruction(event_id: String, name: String, ticket_price: u64, available_tickets: u64)]
    pub struct CreateEventSeating<'info> {
        #[account(
            init,
            payer = organizer,
            seeds = [b"event", event_id.as_bytes()],
            bump,
            space = 1024
        )]
        pub event: Account<'info, EventNFT>,
        #[account(
            init,
            payer = organizer,
            seeds = [b"seating_map", event_id.as_bytes()],
            bump,
            space = 3280
        )]
        pub seating_map: Account<'info, SeatingMap>,
        #[account(mut)]
        pub organizers_pool: Account<'info, OrganizersPool>,
        #[account(mut)]
        pub registry: Account<'info, EventRegistry>,
        #[account(mut)]
        pub organizer: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    pub fn create_event_seating(
        ctx: Context<CreateEventSeating>,
        event_id: String,
        name: String,
        ticket_price: u64,
        available_tickets: u64,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        let registry = &mut ctx.accounts.registry;
        let seating_map = &mut ctx.accounts.seating_map;

        require!(
            ctx.accounts.organizers_pool.organizers.contains(ctx.accounts.organizer.key),
            ErrorCode::Unauthorized
        );

        let expected_event_id = generate_event_id(&name, ctx.accounts.organizer.key);
        require!(event_id == expected_event_id, ErrorCode::InvalidEventId);

        event.event_id = event_id.clone();
        event.organizer = *ctx.accounts.organizer.key;
        event.name = name.clone();
        event.ticket_price = ticket_price;
        event.available_tickets = available_tickets;
        event.sold_tickets = 0;
        // Ustawiamy zawsze numerowany seating
        event.seating_type = 1;
        event.active = false;

        seating_map.event_id = event.event_id.clone();
        seating_map.sections = Vec::new();
        seating_map.total_seats = 0;

        let count = registry.event_count as usize;
        require!(count < 10, ErrorCode::RegistryFull);
        registry.events[count] = event.key();
        registry.event_count += 1;

        Ok(())
    }

    // Modyfikacja eventu – bez zmian w tym fragmencie
    pub fn update_event(
        ctx: Context<UpdateEvent>,
        new_name: Option<String>,
        new_ticket_price: Option<u64>,
        new_available_tickets: Option<u64>,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        require!(!event.active, ErrorCode::EventIsActive);
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
        Ok(())
    }

    #[derive(Accounts)]
    pub struct UpdateEventSeatingType<'info> {
        #[account(mut)]
        pub event: Account<'info, EventNFT>,
        #[account(
            init_if_needed,
            payer = organizer,
            seeds = [b"seating_map", event.event_id.as_bytes()],
            bump,
            space = 3280,
        )]
        pub new_seating_map: Account<'info, SeatingMap>,
        #[account(mut)]
        pub organizer: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    // W tej funkcji jedynie ustawiamy nowy typ – ale teraz wszystkie eventy mają seating_type = 1,
    // więc funkcja może być zbędna lub po prostu ignorować zmianę.
    pub fn update_event_seating_type(
        ctx: Context<UpdateEventSeatingType>,
        new_seating_type: u8,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        if event.sold_tickets > 0 {
            if event.seating_type == 1 && new_seating_type == 1 {
                // pozostaje taki sam
            } else {
                return Err(ErrorCode::CannotChangeSeatingType.into());
            }
        }
        // Nawet jeśli próbujesz zmienić, zawsze ustawiamy seating_type = 1.
        event.seating_type = 1;
        Ok(())
    }
    
    pub fn activate_event(ctx: Context<ActivateEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        require!(!event.active, ErrorCode::EventIsActive);
        event.active = true;
        Ok(())
    }

    pub fn deactivate_event(ctx: Context<DeactivateEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        require!(event.active, ErrorCode::EventNotActive);
        event.active = false;
        Ok(())
    }
    
    pub fn delete_event(ctx: Context<DeleteEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        let registry = &mut ctx.accounts.registry;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        let pos = registry.events.iter().position(|&x| x == event.key()).ok_or(ErrorCode::InvalidTicket)?;
        for i in pos..((registry.event_count as usize) - 1) {
            registry.events[i] = registry.events[i + 1];
        }
        let count = registry.event_count as usize;
        registry.events[count - 1] = Pubkey::default();
        registry.event_count -= 1;
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
        section_name: String,
        section_type: u8,
        rows: u8,
        seats_per_row: u8,
    ) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        let section_account = &mut ctx.accounts.seating_section;
        let event = &ctx.accounts.event;
    
        require!(!event.active, ErrorCode::EventIsActive);
        require!(event.seating_type != 0, ErrorCode::InvalidSeatingType);
    
        require!(rows > 0 && seats_per_row > 0, ErrorCode::InvalidSeating);
    
        let new_seats_count = (rows as u64) * (seats_per_row as u64);
        let updated_total = seating_map
            .total_seats
            .checked_add(new_seats_count)
            .ok_or(ErrorCode::InvalidSeating)?;
        require!(updated_total <= event.available_tickets, ErrorCode::InvalidSeating);
    
        section_account.event_id = seating_map.event_id.clone();
        section_account.section_name = section_name.clone();
        section_account.section_type = section_type;
        section_account.rows = rows;
        section_account.seats_per_row = seats_per_row;
        section_account.seat_status = vec![0; (rows as usize) * (seats_per_row as usize)];
    
        seating_map.sections.push(section_account.key());
        seating_map.total_seats = updated_total;
        Ok(())
    }
    
    pub fn update_seating_section(
        ctx: Context<UpdateSeatingSection>,
        new_rows: Option<u8>,
        new_seats_per_row: Option<u8>,
        new_section_type: Option<u8>,
    ) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        let section = &mut ctx.accounts.seating_section;
        let event = &ctx.accounts.event;

        require!(!event.active, ErrorCode::EventIsActive);
        require!(event.seating_type != 0, ErrorCode::InvalidSeatingType);

        let old_seats_count = (section.rows as u64) * (section.seats_per_row as u64);
        let mut new_rows_val = section.rows;
        let mut new_seats_val = section.seats_per_row;

        if let Some(r) = new_rows {
            require!(r > 0, ErrorCode::InvalidSeating);
            new_rows_val = r;
        }
        if let Some(s) = new_seats_per_row {
            require!(s > 0, ErrorCode::InvalidSeating);
            new_seats_val = s;
        }
        let new_count = (new_rows_val as u64) * (new_seats_val as u64);

        let updated_total = seating_map
            .total_seats
            .checked_sub(old_seats_count)
            .ok_or(ErrorCode::InvalidSeating)?
            .checked_add(new_count)
            .ok_or(ErrorCode::InvalidSeating)?;
        require!(updated_total <= event.available_tickets, ErrorCode::InvalidSeating);

        if let Some(t) = new_section_type {
            section.section_type = t;
        }

        section.rows = new_rows_val;
        section.seats_per_row = new_seats_val;
        section.seat_status = vec![0; (new_rows_val as usize) * (new_seats_val as usize)];

        seating_map.total_seats = updated_total;
        Ok(())
    }

    pub fn remove_seating_section(ctx: Context<RemoveSeatingSection>) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        let seating_section = &ctx.accounts.seating_section;
        let event = &ctx.accounts.event;

        require!(!event.active, ErrorCode::EventIsActive);
        require!(event.seating_type != 0, ErrorCode::InvalidSeatingType);

        require!(
            seating_section.seat_status.iter().all(|&s| s == 0),
            ErrorCode::CannotRemoveSectionWithTickets
        );

        let section_seats = (seating_section.rows as u64)
            .checked_mul(seating_section.seats_per_row as u64)
            .ok_or(ErrorCode::InvalidSeating)?;
        seating_map.total_seats = seating_map
            .total_seats
            .checked_sub(section_seats)
            .ok_or(ErrorCode::InvalidSeating)?;

        if let Some(pos) = seating_map.sections.iter().position(|&x| x == seating_section.key()) {
            seating_map.sections.remove(pos);
        } else {
            return Err(ErrorCode::InvalidSeating.into());
        }
        Ok(())
    }

    pub fn emit_seating_map_details(ctx: Context<EmitSeatingMapDetails>) -> Result<()> {
        let seating_map = &ctx.accounts.seating_map;
        emit!(SeatingMapDetails {
            event_id: seating_map.event_id.clone(),
            total_seats: seating_map.total_seats,
            sections: seating_map.sections.clone(),
        });
        Ok(())
    }

    // ---------------- Mintowanie biletu ----------------
    
    // Uproszczona funkcja mint_ticket – zakładamy zawsze numerowane miejsca.
    pub fn mint_ticket(
        ctx: Context<MintTicket>,
        ticket_id: String,
        event_id: String,
        section_name: String,
        row: u8,
        seat: u8,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &mut ctx.accounts.event;
        require!(event.event_id == event_id, ErrorCode::InvalidTicket);

        let seating_section = ctx
            .accounts
            .seating_section
            .as_mut()
            .ok_or(ErrorCode::InvalidSeating)?;
        require!(seating_section.event_id == event.event_id, ErrorCode::InvalidSeating);
        require!(seating_section.section_name == section_name, ErrorCode::InvalidSeating);

        let index = (row as usize) * (seating_section.seats_per_row as usize) + (seat as usize);
        require!(index < seating_section.seat_status.len(), ErrorCode::InvalidSeating);
        require!(seating_section.seat_status[index] == 0, ErrorCode::SeatAlreadyTaken);
        seating_section.seat_status[index] = 2;

        ticket.ticket_id = ticket_id;
        ticket.event_id = event_id;
        ticket.owner = *ctx.accounts.buyer.key;
        ticket.row = Some(row);
        ticket.seat = Some(seat);
        ticket.used = false;

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

    // ----------------- Funkcje pomocnicze -----------------

    pub fn close_target_account(ctx: Context<CloseTargetAccount>) -> Result<()> {
        require!(ctx.accounts.authority.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);
    
        let closable = &mut ctx.accounts.closable_account;
        let authority = &mut ctx.accounts.authority;
    
        **authority.to_account_info().lamports.borrow_mut() += closable.lamports();
        **closable.to_account_info().lamports.borrow_mut() = 0;
    
        Ok(())
    }
}

// ================= KONTEKSTY (Accounts) =================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [b"fee_pool"],
        bump,
        space = 48
    )]
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
    #[account(init, payer = payer, space = 10240, seeds = [b"event_registry"], bump)]
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
pub struct UpdateEvent<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(signer)]
    pub organizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ActivateEvent<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(signer)]
    /// CHECK: Konto organizatora jest walidowane poprzez porównanie klucza z polem event.organizer.
    pub organizer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DeactivateEvent<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(signer)]
    /// CHECK: Konto organizatora jest walidowane poprzez porównanie klucza z polem event.organizer.
    pub organizer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DeleteEvent<'info> {
    #[account(mut, close = organizer)]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub registry: Account<'info, EventRegistry>,
    #[account(signer)]
    /// CHECK: Konto organizatora jest walidowane poprzez porównanie klucza z polem event.organizer.
    pub organizer: AccountInfo<'info>,
}


#[derive(Accounts)]
#[instruction(event_id: String)]
pub struct InitializeSeating<'info> {
    #[account(
        init,
        payer = organizer,
        seeds = [b"seating_map", event_id.as_bytes()],
        bump,
        space = 3280
    )]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(section_name: String)]
pub struct InitializeSeatingSection<'info> {
    #[account(mut)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(
        init,
        payer = organizer,
        seeds = [b"seating_section", event.key().as_ref(), section_name.as_bytes()],
        bump,
        space = 10084
    )]
    pub seating_section: Account<'info, SeatingSectionAccount>,
    #[account(
        constraint = event.organizer == *organizer.key @ ErrorCode::Unauthorized
    )]
    pub event: Account<'info, EventNFT>,
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
    #[account(
        constraint = event.organizer == *organizer.key @ ErrorCode::Unauthorized
    )]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub organizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveSeatingSection<'info> {
    #[account(mut)]
    pub seating_map: Account<'info, SeatingMap>,
    /// Konto sekcji, która ma zostać usunięta.
    /// Atrybut `close = organizer` spowoduje, że środki z tego konta zostaną przekazane organizatorowi przy zamykaniu.
    #[account(mut, close = organizer)]
    pub seating_section: Account<'info, SeatingSectionAccount>,
    #[account(
        constraint = event.organizer == *organizer.key @ ErrorCode::Unauthorized
    )]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmitSeatingMapDetails<'info> {
    pub seating_map: Account<'info, SeatingMap>,
}

#[derive(Accounts)]
pub struct MintTicket<'info> {
    #[account(init, payer = buyer, space = 200)]
    pub ticket: Account<'info, TicketNFT>,
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    // Dla wszystkich eventów wymagamy konta sekcji
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
    pub event_count: u32,
    pub events: [Pubkey; 10],
}

#[account]
pub struct EventNFT {
    pub event_id: String,
    pub organizer: Pubkey,
    pub name: String,
    pub ticket_price: u64,
    pub available_tickets: u64,
    pub sold_tickets: u64,
    // Wszystkie eventy są numerowane
    pub seating_type: u8,
    pub active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SeatingSection {
    pub section_type: u8,  // dla numerowanych miejsc oczekujemy section_type == 1
    pub rows: u8,
    pub seats_per_row: u8,
    pub seat_status: Vec<u8>,
}

#[account]
pub struct SeatingMap {
    pub event_id: String,
    pub sections: Vec<Pubkey>,
    pub total_seats: u64,
}

#[event]
pub struct SeatingMapDetails {
    pub event_id: String,
    pub total_seats: u64,
    pub sections: Vec<Pubkey>,
}

#[account]
pub struct SeatingSectionAccount {
    pub event_id: String,
    pub section_name: String,
    pub section_type: u8,
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

// ================= FUNKCJE POMOCNICZE =================

#[derive(Accounts)]
pub struct CloseTargetAccount<'info> {
    /// CHECK: To konto musi być MASTER_ACCOUNT. Sprawdzamy to ręcznie w kodzie.
    #[account(signer)]
    pub authority: Signer<'info>,
    /// CHECK: To konto jest zamykane, a jego bezpieczeństwo wynika z faktu, że cały proces zamykania
    /// jest kontrolowany przez MASTER_ACCOUNT, więc nie ma potrzeby dodatkowej walidacji.
    #[account(mut)]
    pub closable_account: AccountInfo<'info>,
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
    #[msg("Event is active and cannot be updated.")]
    EventIsActive,
    #[msg("New available tickets cannot be less than the number of sold tickets.")]
    InvalidTicketQuantity,
    #[msg("Cannot update seating configuration after tickets have been sold.")]
    CannotUpdateSeatingAfterSales,
    #[msg("Event registry is full.")]
    RegistryFull,
    #[msg("Cannot remove section: some tickets are sold or reserved.")]
    CannotRemoveSectionWithTickets,
    #[msg("Cannot change event type because tickets have already been sold. Only allowed to change from numbered to mixed.")]
    CannotChangeSeatingType,
    #[msg("Invalid event ID.")]
    InvalidEventId,
}
