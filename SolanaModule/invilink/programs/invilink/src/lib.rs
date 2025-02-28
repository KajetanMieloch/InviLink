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

declare_id!("8bM5zjY3CMCwCw7A7vUVVgB3RcSxBzDKjJTJtAyTa2BN");

// Stałe globalne
const MASTER_ACCOUNT: Pubkey = pubkey!("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");
const FEE_PERCENTAGE: u64 = 5; // 5%

// ---------------- FUNKCJE POMOCNICZE POZA CHAINEM ----------------

fn generate_event_id(name: &str, organizer: &Pubkey) -> String {
    let seed = b"339562";
    let mut buffer = [0u8; 128];
    let mut pos = 0;
    buffer[pos..pos + seed.len()].copy_from_slice(seed);
    pos += seed.len();
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    buffer[pos..pos + name_len].copy_from_slice(name_bytes);
    pos += name_len;
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

    // Inicjalizacja puli opłat
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let fee_pool = &mut ctx.accounts.fee_pool;
        fee_pool.owner = MASTER_ACCOUNT;
        fee_pool.total_fees = 0;
        Ok(())
    }

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
        registry.events = [Pubkey::default(); 10];
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

    // Aktualizacja eventu
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

    // Aktualizacja konfiguracji sekcji miejsc
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

    // Mintowanie biletu NFT
    pub fn mint_ticket_nft(
        ctx: Context<MintTicketNft>,
        event_id: String,
        event_name: String,
        section_name: String,
        row: u8,
        seat: u8,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;
        require!(event.event_id == event_id, ErrorCode::InvalidTicket);
        require!(event.active, ErrorCode::EventNotActive);

        // Generacja ticket_id przy użyciu funkcji pomocniczej
        let ticket_id = generate_ticket_id(
            ctx.accounts.buyer.key,
            &event_id,
            &event_name,
            &section_name,
            row,
            seat,
        );

        // Mintujemy 1 jednostkę NFT do konta tokenowego
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::mint_to(cpi_ctx, 1)?;

        // Konfiguracja metadanych NFT
        let title = format!("{} Ticket - {} - Row {} Seat {}", event_name, section_name, row, seat);
        let symbol = "TICKET".to_string();
        let uri = format!("https://example.com/metadata/{}", ticket_id);

        let creators = vec![Creator {
            address: event.organizer,
            verified: false,
            share: 100,
        }];

        let data_v2 = DataV2 {
            name: title,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creators),
            collection: None,
            uses: None,
        };

        // Tworzenie konta metadanych NFT przy użyciu CPI
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.buyer.to_account_info(),
            payer: ctx.accounts.buyer.to_account_info(),
            update_authority: ctx.accounts.buyer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_metadata_program.to_account_info(), cpi_accounts);
        create_metadata_accounts_v3(cpi_ctx, data_v2, true, true, None)?;

        // Aktualizacja liczby sprzedanych biletów
        event.sold_tickets = event.sold_tickets.checked_add(1).ok_or(ErrorCode::InvalidTicket)?;
        Ok(())
    }


    pub fn mint_test_nft(
        ctx: Context<MintTestNft>,
        event_id: String,
        event_name: String,
        section_name: String,
        row: u8,
        seat: u8,
    ) -> Result<()> {
        // Mintujemy 1 jednostkę NFT
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::mint_to(cpi_ctx, 1)?;
    
        // Używamy krótkiej nazwy, aby nie przekroczyć limitu (32 bajty)
        let title = "Test Ticket".to_string();
        let symbol = "TEST".to_string();
        let uri = format!("https://example.com/metadata/{}-{}-{}-{}-{}", event_id, event_name, section_name, row, seat);
    
        let creators = vec![mpl_token_metadata::types::Creator {
            address: ctx.accounts.buyer.key(),
            verified: true,
            share: 100,
        }];
    
        let data_v2 = mpl_token_metadata::types::DataV2 {
            name: title,
            symbol,
            uri,
            seller_fee_basis_points: 0,
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
        Ok(())
    }
    
}

//
// KONTEKSTY (ACCOUNTS)
//

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
#[instruction(event_id: String, row: u8, seat: u8)]
pub struct MintTicketNft<'info> {
    #[account(mut)]
    pub event: Account<'info, EventNFT>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        init,
        payer = buyer,
        seeds = [b"mint", event_id.as_bytes(), &[row], &[seat]],
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
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(mut)]
    /// CHECK: Metadane są walidowane przez CPI.
    pub metadata: AccountInfo<'info>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
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
pub struct MintTestNft<'info> {
    /// Konto kupującego – NFT trafi na ten adres
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        init,
        payer = buyer,
        seeds = [
            b"test_mint",
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
    /// CHECK: Konto metadanych – walidowane przez CPI
    pub metadata: AccountInfo<'info>,
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
    pub seating_type: u8,
    pub active: bool,
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SeatingSectionInput {
    pub section_type: u8,
    pub rows: u8,
    pub seats_per_row: u8,
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
}
