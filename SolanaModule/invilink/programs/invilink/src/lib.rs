use anchor_lang::prelude::*;
use solana_program::hash::hash;
use anchor_lang::__private::base64;


declare_id!("EW1pWhJkpreYMn2FpYC3fZzqvLCgRdr79dh6E8SL7qwW");


// Stałe globalne
const MASTER_ACCOUNT: Pubkey = pubkey!("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");
const FEE_PERCENTAGE: u64 = 5; // 5% opłaty manipulacyjnej

// ---------------- FUNKCJE POZA CHAINEM ----------------

// Funkcja do tworzenia unikalnego ID dla eventu na podstawie nazwy, organizatora i czasu
// ID ma 12 znaków w BASE64 czyli  72 bity = 9 bajtów
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
        // Inicjujemy tablicę z domyślnymi wartościami (Pubkey::default())
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
            seeds = [b"event", event_id.as_bytes()], // uzywamy event_id jako części seedów
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
    
        // Autoryzacja – tylko zarejestrowani organizatorzy
        require!(
            ctx.accounts.organizers_pool.organizers.contains(ctx.accounts.organizer.key),
            ErrorCode::Unauthorized
        );
    
        // Weryfikujemy, czy podany event_id zgadza się z oczekiwanym wynikiem
        let expected_event_id = generate_event_id(&name, ctx.accounts.organizer.key);
        require!(event_id == expected_event_id, ErrorCode::InvalidEventId);
    
        // Ustawiamy dane eventu
        event.event_id = event_id.clone();
        event.organizer = *ctx.accounts.organizer.key;
        event.name = name;
        event.ticket_price = ticket_price;
        event.available_tickets = available_tickets;
        event.sold_tickets = 0;
        event.seating_type = 0; // Open-space
        event.active = false;
            
        // Aktualizacja rejestru – zapisujemy adres utworzonego eventu
        let count = registry.event_count as usize;
        require!(count < 10, ErrorCode::RegistryFull);
        registry.events[count] = event.key();
        registry.event_count += 1;
            
        // Dodajemy mapping EVENTID -> PDA eventu do słownika
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
        space = 9000, // Dostosuj do przewidywanej liczby eventów i rozmiaru par
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

    
#[derive(Accounts)]
#[instruction(event_id: String, name: String, ticket_price: u64, available_tickets: u64, seating_type: u8)]
pub struct CreateEventSeating<'info> {
    #[account(
        init,
        payer = organizer,
        seeds = [b"event", event_id.as_bytes()], // tutaj również używamy event_id
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
    seating_type: u8,
) -> Result<()> {
    require!(seating_type == 1 || seating_type == 2, ErrorCode::InvalidSeatingType);

    let event = &mut ctx.accounts.event;
    let registry = &mut ctx.accounts.registry;
    let seating_map = &mut ctx.accounts.seating_map;

    require!(
        ctx.accounts.organizers_pool.organizers.contains(ctx.accounts.organizer.key),
        ErrorCode::Unauthorized
    );

    // Weryfikujemy, czy podany event_id zgadza się z oczekiwanym wynikiem
    let expected_event_id = generate_event_id(&name, ctx.accounts.organizer.key);
    require!(event_id == expected_event_id, ErrorCode::InvalidEventId);

    // Ustawiamy dane eventu
    event.event_id = event_id.clone();
    event.organizer = *ctx.accounts.organizer.key;
    event.name = name.clone();
    event.ticket_price = ticket_price;
    event.available_tickets = available_tickets;
    event.sold_tickets = 0;
    event.seating_type = seating_type;
    event.active = false;

    // Inicjalizacja seating mapy
    seating_map.event_id = event.event_id.clone();
    seating_map.sections = Vec::new();
    seating_map.total_seats = 0;

    // Aktualizacja rejestru
    let count = registry.event_count as usize;
    require!(count < 10, ErrorCode::RegistryFull);
    registry.events[count] = event.key();
    registry.event_count += 1;

    Ok(())
}


    // Modyfikacja eventu dozwolona tylko, gdy event nie jest aktywny
    pub fn update_event(
        ctx: Context<UpdateEvent>,
        new_name: Option<String>,
        new_ticket_price: Option<u64>,
        new_available_tickets: Option<u64>,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        // Aktualizacja jest dozwolona tylko, gdy event jest nieaktywny
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
        // Jeśli zmieniamy z open-space na seating – inicjujemy nowe konto.
        // Jeśli zmieniamy z seating na open-space – to konto zostanie zamknięte.
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


    pub fn update_event_seating_type(
        ctx: Context<UpdateEventSeatingType>,
        new_seating_type: u8,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
    
        // Jeśli są sprzedane bilety, dopuszczamy tylko zmianę z numerowanych (1) na mieszane (2)
        if event.sold_tickets > 0 {
            if event.seating_type == 1 && new_seating_type == 2 {
                // dozwolona zmiana
            } else {
                return Err(ErrorCode::CannotChangeSeatingType.into());
            }
        }
    
        // Brak sprzedanych biletów – możemy zmieniać typ eventu
        // Jeśli zmieniamy z open-space (0) na seating (1 lub 2),
        // automatycznie inicjujemy nowe konto Seating Map.
        if event.seating_type == 0 && (new_seating_type == 1 || new_seating_type == 2) {
            let seating_map = &mut ctx.accounts.new_seating_map;
            seating_map.event_id = event.event_id.clone();
            seating_map.sections = Vec::new();
            seating_map.total_seats = 0;
        }
        // Jeśli zmieniamy z seating (1 lub 2) na open-space (0),
        // konto Seating Map zostanie zamknięte (dzięki atrybutowi close w kontekście).
        // (Front-end musi przekazać konto Seating Map w tym przypadku.)
        
        event.seating_type = new_seating_type;
        Ok(())
    }
    

    // Funkcja aktywująca event – po jej wywołaniu bilety można kupić
    pub fn activate_event(ctx: Context<ActivateEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        // Można aktywować event tylko, gdy jest on nieaktywny
        require!(!event.active, ErrorCode::EventIsActive);
        event.active = true;
        Ok(())
    }

    pub fn deactivate_event(ctx: Context<DeactivateEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        // Upewniamy się, że event jest już aktywny, aby móc go dezaktywować
        require!(event.active, ErrorCode::EventNotActive);
        event.active = false;
        Ok(())
    }
    
    pub fn delete_event(ctx: Context<DeleteEvent>) -> Result<()> {
        let event = &mut ctx.accounts.event;
        let registry = &mut ctx.accounts.registry;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        // Usuwamy event z rejestru: szukamy jego pozycji w tablicy
        let pos = registry
          .events
          .iter()
          .position(|&x| x == event.key())
          .ok_or(ErrorCode::InvalidTicket)?;
        // Przesuwamy elementy, aby nadpisać usuwany element
        for i in pos..((registry.event_count as usize) - 1) {
            registry.events[i] = registry.events[i + 1];
        }
        // Ustawiamy ostatni element jako domyślny i dekrementujemy licznik
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
    
        // Blokada, jeśli event jest aktywny lub seating_type=0
        require!(!event.active, ErrorCode::EventIsActive);
        require!(event.seating_type != 0, ErrorCode::InvalidSeatingType);
    
        // Nowa sekcja nie może mieć 0 wierszy
        require!(rows > 0 && seats_per_row > 0, ErrorCode::InvalidSeating);
    
        let new_seats_count = (rows as u64) * (seats_per_row as u64);
        let updated_total = seating_map
            .total_seats
            .checked_add(new_seats_count)
            .ok_or(ErrorCode::InvalidSeating)?;
        require!(updated_total <= event.available_tickets, ErrorCode::InvalidSeating);
    
        // Ustawiamy identyfikator eventu
        section_account.event_id = seating_map.event_id.clone();
        // Zamiast indeksu zapisujemy nazwę sekcji – dzięki temu użytkownik nie musi podawać liczbowego indeksu
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

        // Blokada, jeśli event jest aktywny lub seating_type=0
        require!(!event.active, ErrorCode::EventIsActive);
        require!(event.seating_type != 0, ErrorCode::InvalidSeatingType);

        // Obliczamy stare i nowe seats
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
            .ok_or(ErrorCode::InvalidSeating)?;
        let updated_total = updated_total
            .checked_add(new_count)
            .ok_or(ErrorCode::InvalidSeating)?;
        // Nie przekraczamy available_tickets
        require!(updated_total <= event.available_tickets, ErrorCode::InvalidSeating);

        // Aktualizujemy typ sekcji, jeśli podano
        if let Some(t) = new_section_type {
            section.section_type = t;
        }

        section.rows = new_rows_val;
        section.seats_per_row = new_seats_val;
        section.seat_status = vec![0; (new_rows_val as usize) * (new_seats_val as usize)];

        // Zapisujemy nową łączną liczbę miejsc
        seating_map.total_seats = updated_total;
        Ok(())
    }

    pub fn remove_seating_section(ctx: Context<RemoveSeatingSection>) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        let seating_section = &ctx.accounts.seating_section;
        let event = &ctx.accounts.event;

        // Blokada: event musi być nieaktywny oraz seating_type != 0
        require!(!event.active, ErrorCode::EventIsActive);
        require!(event.seating_type != 0, ErrorCode::InvalidSeatingType);

        // Upewnij się, że w sekcji nie ma sprzedanych ani zarezerwowanych miejsc
        require!(
            seating_section.seat_status.iter().all(|&s| s == 0),
            ErrorCode::CannotRemoveSectionWithTickets
        );

        // Oblicz liczbę miejsc w usuwanej sekcji
        let section_seats = (seating_section.rows as u64)
            .checked_mul(seating_section.seats_per_row as u64)
            .ok_or(ErrorCode::InvalidSeating)?;
        seating_map.total_seats = seating_map
            .total_seats
            .checked_sub(section_seats)
            .ok_or(ErrorCode::InvalidSeating)?;

        // Usuń klucz sekcji z wektora w seating_map
        if let Some(pos) = seating_map.sections.iter().position(|&x| x == seating_section.key()) {
            seating_map.sections.remove(pos);
        } else {
            return Err(ErrorCode::InvalidSeating.into());
        }
        // Konto seating_section zostanie zamknięte automatycznie dzięki atrybutowi `close`
        Ok(())
    }

    /// Funkcja emituje zdarzenie zawierające informacje o mapie miejsc.
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
    
    // Funkcja mint_ticket tworzy nowe konto biletu (TicketNFT).
    // Dla eventów z seatingiem (typ 1 lub 2) wymaga podania konta SeatingSectionAccount,
    // aby zarezerwować miejsce (oznaczając je jako sprzedane).
    pub fn mint_ticket(
        ctx: Context<MintTicket>,
        ticket_id: String,
        event_id: String,
        section_name: String,  // teraz jako String, nie Option<String>
        row: Option<u8>,
        seat: Option<u8>,
    ) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &mut ctx.accounts.event;
        // Sprawdzenie, czy event_id przekazany w argumencie odpowiada eventowi.
        require!(event.event_id == event_id, ErrorCode::InvalidTicket);
        // Możesz odkomentować poniższą linię, jeśli chcesz wymusić aktywność eventu.
        // require!(event.active, ErrorCode::EventNotActive);
    
        if event.seating_type == 1 || event.seating_type == 2 {
            let seating_section = ctx
                .accounts
                .seating_section
                .as_mut()
                .ok_or(ErrorCode::InvalidSeating)?;
            require!(seating_section.event_id == event.event_id, ErrorCode::InvalidSeating);
            // Bezpośrednio porównujemy section_name
            require!(seating_section.section_name == section_name, ErrorCode::InvalidSeating);
    
            if seating_section.section_type == 1 {
                // Numerowane miejsca:
                let row_val = row.ok_or(ErrorCode::InvalidSeating)?;
                let seat_val = seat.ok_or(ErrorCode::InvalidSeating)?;
                let index = (row_val as usize) * (seating_section.seats_per_row as usize)
                    + (seat_val as usize);
                require!(index < seating_section.seat_status.len(), ErrorCode::InvalidSeating);
                require!(seating_section.seat_status[index] == 0, ErrorCode::SeatAlreadyTaken);
                seating_section.seat_status[index] = 2;
            } else {
                // Stojące lub mieszane – rezerwujemy pierwsze wolne miejsce.
                let pos = seating_section
                    .seat_status
                    .iter()
                    .position(|&s| s == 0)
                    .ok_or(ErrorCode::SeatNotReserved)?;
                seating_section.seat_status[pos] = 2;
            }
        }
        // Ustawienie danych biletu
        ticket.ticket_id = ticket_id;
        ticket.event_id = event_id;
        ticket.owner = *ctx.accounts.buyer.key;
        ticket.row = row;
        ticket.seat = seat;
        ticket.used = false;
        // Zwiększenie liczby sprzedanych biletów
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
        require!(
            ctx.accounts.authority.key() == MASTER_ACCOUNT,
            ErrorCode::Unauthorized
        );
    
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
    #[account(mut)]
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

    // Dla eventów z seatingiem (1 lub 2)
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
    pub event_count: u32,           // liczba zapisanych eventów
    pub events: [Pubkey; 10],     // stała tablica dla maksymalnie 100 eventów
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
    pub total_seats: u64, // <--- do zliczania sumy miejsc
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

// ================= FUNKCJE POMOCNICZE =================
#[derive(Accounts)]
pub struct CloseTargetAccount<'info> {
    /// CHECK: To konto musi być MASTER_ACCOUNT. Sprawdzamy to ręcznie w kodzie.
    #[account(signer)]
    pub authority: Signer<'info>,

    /// CHECK: Konto, które ma zostać zamknięte (dowolne konto pod kontrolą programu)
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
