use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata as MetadataAccount,
};
use mpl_token_metadata::types::{DataV2, Creator};
use solana_program::hash::hash;
use solana_program::pubkey;
use anchor_spl::metadata::Metadata;
use base64::{engine::general_purpose, Engine as _};
// Dodajemy importy niezbędne do transferu lamportów
use solana_program::program::invoke;
use solana_program::system_instruction;

declare_id!("2Yh2Jud5p81cVVM5Si2S53YcmtgErkuCTsX8RBhZ91ab");

// Stałe globalne
const MASTER_ACCOUNT: Pubkey = pubkey!("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");
const FEE_PERCENTAGE: u64 = 5; // 5%

// ---------------- FUNKCJE POMOCNICZE POZA CHAINEM ----------------

fn generate_event_id(name: &str, event_date: i64, organizer: &Pubkey) -> String {
    let seed = b"339562";
    let mut buffer = [0u8; 128];
    let mut pos = 0;
    buffer[pos..pos + seed.len()].copy_from_slice(seed);
    pos += seed.len();
    
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    buffer[pos..pos + name_len].copy_from_slice(name_bytes);
    pos += name_len;
    
    // Dodajemy datę – 8 bajtów, little-endian
    let date_bytes = event_date.to_le_bytes();
    buffer[pos..pos + date_bytes.len()].copy_from_slice(&date_bytes);
    pos += date_bytes.len();
    
    let pubkey_bytes = organizer.to_bytes();
    buffer[pos..pos + pubkey_bytes.len()].copy_from_slice(&pubkey_bytes);
    pos += pubkey_bytes.len();
    
    let hash_result = hash(&buffer[..pos]);
    let encoded = general_purpose::URL_SAFE_NO_PAD.encode(hash_result.as_ref());
    encoded.chars().take(12).collect()
}

fn generate_ticket_id(
    buyer: &Pubkey, 
    event_id: &str, 
    event_name: &str, 
    section_name: &str, 
    row: u8, 
    seat: u8
) -> String {
    let mut buffer = [0u8; 256];
    let mut pos = 0;
    let buyer_bytes = buyer.to_bytes();
    buffer[pos..pos + buyer_bytes.len()].copy_from_slice(&buyer_bytes);
    pos += buyer_bytes.len();
    let event_id_bytes = event_id.as_bytes();
    buffer[pos..pos + event_id_bytes.len()].copy_from_slice(event_id_bytes);
    pos += event_id_bytes.len();
    let event_name_bytes = event_name.as_bytes();
    buffer[pos..pos + event_name_bytes.len()].copy_from_slice(event_name_bytes);
    pos += event_name_bytes.len();
    let section_bytes = section_name.as_bytes();
    buffer[pos..pos + section_bytes.len()].copy_from_slice(section_bytes);
    pos += section_bytes.len();
    buffer[pos] = row;
    pos += 1;
    buffer[pos] = seat;
    pos += 1;
    let hash_result = hash(&buffer[..pos]);
    let encoded = base64::encode(hash_result.as_ref());
    encoded.chars().take(12).collect()
}

// ---------------- PROGRAM NA CHAINIE ----------------

#[program]
pub mod invilink {
    use super::*;

    // Inicjalizacja puli organizatorów
    pub fn initialize_organizers_pool(ctx: Context<InitializeOrganizersPool>) -> Result<()> {
        let organizers_pool = &mut ctx.accounts.organizers_pool;
        organizers_pool.organizers = Vec::new();
        Ok(())
    }

    // Inicjalizacja rejestru eventów
    pub fn initialize_event_registry(ctx: Context<InitializeEventRegistry>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.event_count = 0;
        registry.events = Vec::new();
        Ok(())
    }

    // Dodanie nowego organizatora
    pub fn add_organizer(ctx: Context<AddOrganizer>, new_organizer: Pubkey) -> Result<()> {
        let organizers = &mut ctx.accounts.organizers_pool;
        require!(ctx.accounts.signer.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);
        require!(!organizers.organizers.contains(&new_organizer), ErrorCode::AlreadyRegistered);
        organizers.organizers.push(new_organizer);
        Ok(())
    }
    // Dodanie nowego walidatora
    pub fn add_validator(ctx: Context<AddValidator>, validator: Pubkey) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        if event.validators.contains(&validator) {
            return Err(ErrorCode::ValidatorAlreadyAdded.into());
        }
        event.validators.push(validator);
        Ok(())
    }
    
    // Usunięcie organizatora
    pub fn remove_organizer(ctx: Context<RemoveOrganizer>, organizer_to_remove: Pubkey) -> Result<()> {
        let organizers = &mut ctx.accounts.organizers_pool;
        require!(ctx.accounts.signer.key() == MASTER_ACCOUNT, ErrorCode::Unauthorized);
        let index = organizers.organizers.iter().position(|x| *x == organizer_to_remove);
        require!(index.is_some(), ErrorCode::OrganizerNotFound);
        organizers.organizers.remove(index.unwrap());
        Ok(())
    }

    // Zarządzanie eventami – wyłącznie przez create_event_seating
    #[derive(Accounts)]
    #[instruction(
        event_id: String,
        name: String,
        available_tickets: u64
    )]
    pub struct CreateEventSeating<'info> {
        #[account(
            init,
            payer = organizer,
            seeds = [b"event", event_id.as_bytes()],
            bump,
            space = 1400
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
        event_date: i64,
        available_tickets: u64,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        let registry = &mut ctx.accounts.registry;
        let seating_map = &mut ctx.accounts.seating_map;
    
        require!(
            ctx.accounts.organizers_pool.organizers.contains(ctx.accounts.organizer.key),
            ErrorCode::Unauthorized
        );
    
        require!(event_date > Clock::get()?.unix_timestamp, ErrorCode::InvalidEventDate);
        require!(available_tickets > 0, ErrorCode::InvalidTicketQuantity);

        let expected_event_id = generate_event_id(&name, event_date, ctx.accounts.organizer.key);
        require!(event_id == expected_event_id, ErrorCode::InvalidEventId);
    
        event.event_id = event_id.clone();
        event.organizer = *ctx.accounts.organizer.key;
        event.name = name.clone();
        event.event_date = event_date;
        event.available_tickets = available_tickets;
        event.sold_tickets = 0;
        event.seating_type = 1;
        event.active = false;
        event.validators = Vec::new();
    
        // Inicjalizacja mapy miejsc
        seating_map.event_id = event.event_id.clone();
        seating_map.organizer = event.organizer;
        seating_map.sections = Vec::new();
        seating_map.total_seats = 0;
    
        registry.events.push(event.key());
        registry.event_count = registry.events.len() as u32;    
        Ok(())
    }
    
    // Aktualizacja eventu
    pub fn update_event(
        ctx: Context<UpdateEvent>,
        new_name: Option<String>,
        new_date: Option<i64>,
        new_available_tickets: Option<u64>,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.organizer == *ctx.accounts.organizer.key, ErrorCode::Unauthorized);
        require!(!event.active, ErrorCode::EventIsActive);
    
        if let Some(name) = new_name {
            event.name = name;
        }
        if let Some(date) = new_date {
            require!(date > Clock::get()?.unix_timestamp, ErrorCode::InvalidEventDate);
            event.event_date = date;
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
        // Dodany warunek: nie da się deaktywować eventu, jeśli sprzedano jakiekolwiek bilety
        require!(event.sold_tickets == 0, ErrorCode::EventCannotDeactivate);
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

    // Inicjalizacja mapy miejsc
    pub fn initialize_seating(ctx: Context<InitializeSeating>, event_id: String) -> Result<()> {
        let seating_map = &mut ctx.accounts.seating_map;
        seating_map.event_id = event_id;
        seating_map.sections = Vec::new();
        Ok(())
    }

    // Inicjalizacja sekcji miejsc
    pub fn initialize_seating_section(
        ctx: Context<InitializeSeatingSection>,
        section_name: String,
        section_type: u8,
        rows: u8,
        seats_per_row: u8,
        ticket_price: u64, //cena w danej sekcji niezależna od ceny eventu
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
        section_account.ticket_price = ticket_price; // Ustawiamy cenę sekcji
        section_account.seat_status = vec![0; (rows as usize) * (seats_per_row as usize)];
    
        seating_map.sections.push(section_account.key());
        seating_map.total_seats = updated_total;
        Ok(())
    }
    
    // Aktualizacja konfiguracji sekcji miejsc
    pub fn update_seating_section(
        ctx: Context<UpdateSeatingSection>,
        new_rows: Option<u8>,
        new_seats_per_row: Option<u8>,
        new_section_type: Option<u8>,
        new_ticket_price: Option<u64>, // NOWY parametr
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
        // Aktualizacja ceny – jeśli podano nową cenę, zmieniamy
        if let Some(new_price) = new_ticket_price {
            section.ticket_price = new_price;
        }
    
        section.rows = new_rows_val;
        section.seats_per_row = new_seats_val;
        section.seat_status = vec![0; (new_rows_val as usize) * (new_seats_val as usize)];
    
        seating_map.total_seats = updated_total;
        Ok(())
    }
    
    // Usunięcie sekcji miejsc
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
    
    // Emitowanie szczegółów mapy miejsc jako event
    pub fn emit_seating_map_details(ctx: Context<EmitSeatingMapDetails>) -> Result<()> {
        let seating_map = &ctx.accounts.seating_map;
        emit!(SeatingMapDetails {
            event_id: seating_map.event_id.clone(),
            total_seats: seating_map.total_seats,
            sections: seating_map.sections.clone(),
        });
        Ok(())
    }
    
    pub fn mint_ticket_nft(
        ctx: Context<MintTicketNft>,
        event_id: String,
        event_name: String,
        section_name: String,
        row: u8,
        seat: u8,
        ipfs_uri: String, // nowy parametr – dynamiczne URI z IPFS
    ) -> Result<()> {
        // 1. Sprawdzenie, czy konto SeatingMap zawiera właściwy event_id
        require!(
            ctx.accounts.seating_map.event_id == event_id,
            ErrorCode::InvalidEventId
        );
        // 2. Sprawdzenie, czy event jest aktywny
        require!(
            ctx.accounts.event.active,
            ErrorCode::EventNotActive
        );
        // 2a. Sprawdzenie, czy wydarzenie nie minęło (zakaz zakupu biletu po dacie wydarzenia)
        let clock = Clock::get()?;
        let adjusted_timestamp = clock.unix_timestamp - 86400; // odejmujemy 1 dzień (86400 sekund)
        require!(
            adjusted_timestamp < ctx.accounts.event.event_date,
            ErrorCode::EventAlreadyOccurred
        );
        
        // 3. Sprawdzenie, czy miejsce w danej sekcji istnieje
        let seating_section = &mut ctx.accounts.seating_section;
        require!(
            (row as usize) < (seating_section.rows as usize),
            ErrorCode::InvalidSeating
        );
        require!(
            (seat as usize) < (seating_section.seats_per_row as usize),
            ErrorCode::InvalidSeating
        );
        let seat_index = (row as usize) * (seating_section.seats_per_row as usize) + (seat as usize);
        require!(
            seat_index < seating_section.seat_status.len(),
            ErrorCode::InvalidSeating
        );
        require!(
            seating_section.seat_status[seat_index] == 0,
            ErrorCode::SeatAlreadyTaken
        );

        // Pobieramy cenę biletu z konta sekcji
        let price = seating_section.ticket_price;
        if ctx.accounts.buyer.lamports() < price {
            return Err(ErrorCode::InsufficientFunds.into());
        }
        // Obliczamy opłatę (5%) i kwotę dla organizatora (95%)
        let fee = price.checked_mul(FEE_PERCENTAGE).unwrap().checked_div(100).unwrap();
        let organizer_amount = price.checked_sub(fee).unwrap();
    
        {
            let buyer_info = ctx.accounts.buyer.to_account_info();
            let organizer_info = ctx.accounts.organizer_wallet.to_account_info();
            let master_info = ctx.accounts.master_account.to_account_info();
            let system_program_info = ctx.accounts.system_program.to_account_info();
    
            // Transfer 95% ceny do organizatora
            let ix_to_organizer = system_instruction::transfer(
                buyer_info.key,
                organizer_info.key,
                organizer_amount,
            );
            invoke(
                &ix_to_organizer,
                &[buyer_info.clone(), organizer_info.clone(), system_program_info.clone()],
            )?;
    
            // Transfer 5% ceny do MASTER_ACCOUNT
            let ix_to_master = system_instruction::transfer(
                buyer_info.key,
                master_info.key,
                fee,
            );
            invoke(
                &ix_to_master,
                &[buyer_info.clone(), master_info.clone(), system_program_info.clone()],
            )?;
        }
    
        // Mintujemy NFT – mintujemy 1 jednostkę tokena
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::mint_to(cpi_ctx, 1)?;
    
        // Generacja ticket_id przy użyciu funkcji pomocniczej (niezmieniona)
        let ticket_id = generate_ticket_id(
            ctx.accounts.buyer.key,
            &event_id,
            &event_name,
            &section_name,
            row,
            seat,
        );
    
        // Ustalamy metadane – dynamicznie pobieramy URI przekazane przez parametr ipfs_uri
        let title = format!("Invilink Ticket");
        let symbol = "INVI".to_string();
        let uri = ipfs_uri; // TU używamy przekazanego dynamicznego URI
    
        let creators = vec![mpl_token_metadata::types::Creator {
            address: ctx.accounts.buyer.key(),
            verified: true,
            share: 100,
        }];
    
        let data_v2 = mpl_token_metadata::types::DataV2 {
            name: title,
            symbol,
            uri,
            seller_fee_basis_points: 500,
            creators: Some(creators),
            collection: None,
            uses: None,
        };
    
        let cpi_accounts_meta = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.buyer.to_account_info(),
            payer: ctx.accounts.buyer.to_account_info(),
            update_authority: ctx.accounts.buyer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
    
        let cpi_ctx_meta = CpiContext::new(ctx.accounts.token_metadata_program.to_account_info(), cpi_accounts_meta);
        create_metadata_accounts_v3(cpi_ctx_meta, data_v2, true, true, None)?;
    
        // Aktualizacja stanu miejsca – oznaczamy miejsce jako zajęte
        seating_section.seat_status[seat_index] = 1;
        // Aktualizacja liczby sprzedanych biletów
        ctx.accounts.event.sold_tickets = ctx.accounts.event.sold_tickets.checked_add(1)
            .ok_or(ErrorCode::InvalidTicket)?;
        Ok(())
    }
    
    // ---------------- WALIDACJA BILETU BEZ ticket_mint ----------------
    // Walidacja oparta na: event_id, section, row, seat (np. przekazywane z URL)

    /// Inicjalizacja konta TicketStatus – podczas tworzenia biletu ustawiamy used = false, activated = false
    #[derive(Accounts)]
    #[instruction(event_id: String, section: String, row: u8, seat: u8, event: Pubkey)]
    pub struct InitializeTicketStatus<'info> {
        #[account(
            init_if_needed,
            payer = payer,
            seeds = [b"ticket_status", event_id.as_bytes(), section.as_bytes(), &[row], &[seat]],
            bump,
            space = 50 // 8 (discriminator) + 32 (event) + 1 (used) + 1 (activated) + 8 (timestamp)
        )]
        pub ticket_status: Account<'info, TicketStatus>,
        #[account(mut)]
        pub payer: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
    
    pub fn initialize_ticket_status(
        ctx: Context<InitializeTicketStatus>,
        event_id: String,
        section: String,
        row: u8,
        seat: u8,
        event: Pubkey,
    ) -> Result<()> {
        let ticket_status = &mut ctx.accounts.ticket_status;
        ticket_status.event = event;
        ticket_status.used = false;
        ticket_status.activated = false;
        ticket_status.activation_timestamp = 0;
        Ok(())
    }

        /// Kontekst dla aktywacji biletu – może być wywoływany np. przez właściciela biletu
    #[derive(Accounts)]
    #[instruction(event_id: String, section: String, row: u8, seat: u8)]
    pub struct ActivateTicket<'info> {
        #[account(
            mut,
            seeds = [b"ticket_status", event_id.as_bytes(), section.as_bytes(), &[row], &[seat]],
            bump,
        )]
        pub ticket_status: Account<'info, TicketStatus>,
        #[account(signer)]
        /// CHECK: Konto użytkownika – weryfikacja może być dokonana na podstawie logiki aplikacji
        pub user: AccountInfo<'info>,
    }

    /// Funkcja aktywująca bilet – ustawia flagę activated na true i zapisuje aktualny czas
    pub fn activate_ticket(
        ctx: Context<ActivateTicket>,
        event_id: String,
        section: String,
        row: u8,
        seat: u8,
    ) -> Result<()> {
        let ticket_status = &mut ctx.accounts.ticket_status;
        // Jeśli bilet był już użyty, nie można go aktywować
        require!(!ticket_status.used, ErrorCode::TicketAlreadyUsed);
        let current_ts = Clock::get()?.unix_timestamp;
        ticket_status.activated = true;
        ticket_status.activation_timestamp = current_ts;
        Ok(())
    }
    

 /// Kontekst walidacji biletu – wywoływany przez walidatora
#[derive(Accounts)]
#[instruction(event_id: String, section: String, row: u8, seat: u8)]
pub struct ValidateTicket<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(
        mut,
        seeds = [b"ticket_status", event_id.as_bytes(), section.as_bytes(), &[row], &[seat]],
        bump,
    )]
    pub ticket_status: Account<'info, TicketStatus>,
    #[account(signer)]
    /// CHECK: Konto walidatora – weryfikujemy je w programie, porównując z listą
    pub validator: AccountInfo<'info>,
}

/// Walidacja biletu – sprawdzamy, czy bilet został aktywowany i czy okres aktywacji (5 min) nie wygasł.
pub fn validate_ticket(ctx: Context<ValidateTicket>, event_id: String, section: String, row: u8, seat: u8) -> Result<()> {
    let event = &ctx.accounts.event;
    let validator = ctx.accounts.validator.key();
    // Weryfikujemy, czy walidator znajduje się na liście zatwierdzonych
    require!(event.validators.contains(&validator), ErrorCode::NotValidator);
    
    let ticket_status = &mut ctx.accounts.ticket_status;
    // Sprawdzamy, czy bilet został aktywowany
    if !ticket_status.activated {
        return Err(ErrorCode::TicketNotActivated.into());
    }
    let current_ts = Clock::get()?.unix_timestamp;
    // Jeśli od momentu aktywacji minęło więcej niż 5 minut (300 sekund), to traktujemy aktywację jako wygasłą
    if current_ts - ticket_status.activation_timestamp > 300 {
        ticket_status.activated = false; // automatycznie deaktywujemy
        return Err(ErrorCode::TicketActivationExpired.into());
    }
    // Sprawdzamy, czy bilet nie był już wcześniej użyty
    require!(!ticket_status.used, ErrorCode::TicketAlreadyUsed);
    
    // Oznaczamy bilet jako użyty
    ticket_status.used = true;
    
    // Emitujemy event walidacji (opcjonalnie)
    emit!(TicketValidated {
        event: event.key(),
        validator,
        timestamp: current_ts,
    });
    
    Ok(())
}
}

// ---------------- KONTEKSTY (ACCOUNTS) ----------------

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
pub struct AddValidator<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(signer)]
    /// CHECK: Konto organizatora – jego klucz jest porównywany z event.organizer.
    pub organizer: Signer<'info>,
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
pub struct CloseTargetAccount<'info> {
    /// CHECK: To konto musi być MASTER_ACCOUNT. Sprawdzamy to ręcznie w kodzie.
    #[account(signer)]
    pub authority: Signer<'info>,
    /// CHECK: To konto jest zamykane, a jego bezpieczeństwo wynika z faktu, że cały proces zamykania
    /// jest kontrolowany przez MASTER_ACCOUNT, więc nie ma potrzeby dodatkowej walidacji.
    #[account(mut)]
    pub closable_account: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(event_id: String, event_name: String, section_name: String, row: u8, seat: u8)]
pub struct MintTicketNft<'info> {
    #[account(mut, seeds = [b"event", event_id.as_bytes()], bump)]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [b"seating_map", event_id.as_bytes()], bump)]
    pub seating_map: Account<'info, SeatingMap>,
    #[account(mut)]
    pub seating_section: Account<'info, SeatingSectionAccount>,
    #[account(
        init,
        payer = buyer,
        seeds = [
            b"mint_ticket",
            event_id.as_bytes(),
            event_name.as_bytes(),
            section_name.as_bytes(),
            &[row],
            &[seat]
        ],
        bump,
        mint::decimals = 0,
        mint::authority = buyer,
        mint::freeze_authority = buyer,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: Metadane są walidowane przez CPI.
    pub metadata: AccountInfo<'info>,
    #[account(mut, address = MASTER_ACCOUNT)]
    /// CHECK: Konto MASTER_ACCOUNT (stałe) – przekazywane będą opłaty.
    pub master_account: AccountInfo<'info>,
    #[account(mut, address = event.organizer)]
    /// CHECK: Konto organizatora, do którego trafi 95% ceny.
    pub organizer_wallet: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// ---------------- STRUKTURY KONTA ----------------

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
    pub events: Vec<Pubkey>,
}

#[account]
pub struct EventNFT {
    pub event_id: String,
    pub organizer: Pubkey,
    pub name: String,
    pub event_date: i64, // np. UNIX timestamp
    pub ticket_price: u64,
    pub available_tickets: u64,
    pub sold_tickets: u64,
    pub seating_type: u8,
    pub active: bool,
    pub validators: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SeatingSection {
    pub section_type: u8,
    pub rows: u8,
    pub seats_per_row: u8,
    pub seat_status: Vec<u8>,
}

#[account]
pub struct SeatingMap {
    pub event_id: String,
    pub organizer: Pubkey, // adres organizatora, pobieramy go przy tworzeniu eventu
    pub active: bool,      // stan eventu – true, gdy event jest aktywny
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
    pub ticket_price: u64, // NOWE – cena biletu dla tej sekcji
    pub seat_status: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SeatingSectionInput {
    pub section_type: u8,
    pub rows: u8,
    pub seats_per_row: u8,
}

#[account]
pub struct TicketStatus {
    pub event: Pubkey,            // event, do którego bilet się odnosi
    pub used: bool,               // czy bilet został wykorzystany
    pub activated: bool,          // czy bilet został aktywowany
    pub activation_timestamp: i64 // moment aktywacji (unix timestamp)
}

#[event]
pub struct TicketValidated {
    pub event: Pubkey,
    pub validator: Pubkey,
    pub timestamp: i64,
}

// ---------------- ERROR CODE ----------------

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
    #[msg("Event cannot be deactivated because tickets have been sold.")]
    EventCannotDeactivate,
    #[msg("Event has already occurred.")]
    EventAlreadyOccurred,
    #[msg("Validator already added.")]
    ValidatorAlreadyAdded,
    #[msg("Caller is not a validator for this event.")]
    NotValidator,
    #[msg("Ticket not activated. Activate first.")]
    TicketNotActivated,
    #[msg("Ticket activation expired.")]
    TicketActivationExpired,
    #[msg("Cannot create event with past date.")]
    InvalidEventDate,
}
