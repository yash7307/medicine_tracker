use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWxTWqkZPKC9anDgRPR9H7YfBoUn");

// ============================================================
// MEDICINE TRACKER - Solana Smart Contract (Rust + Anchor)
// Modules: 1) Register Batch  2) Transfer Custody
//          3) Log Temperature  4) Verify / Read Audit Trail
// ============================================================

#[program]
pub mod medicine_tracker {
    use super::*;

    // ----------------------------------------------------------
    // MODULE 1: Register a new drug batch (Manufacturer only)
    // ----------------------------------------------------------
    pub fn register_batch(
        ctx: Context<RegisterBatch>,
        batch_id: String,
        drug_name: String,
        manufacturer: String,
        expiry_date: String,
        ipfs_hash: String,       // IPFS hash of certificate/lab report
        min_temp: i32,           // minimum allowed temperature (°C)
        max_temp: i32,           // maximum allowed temperature (°C)
    ) -> Result<()> {
        require!(batch_id.len() <= 32, ErrorCode::StringTooLong);
        require!(drug_name.len() <= 64, ErrorCode::StringTooLong);

        let batch = &mut ctx.accounts.drug_batch;
        batch.batch_id        = batch_id;
        batch.drug_name       = drug_name;
        batch.manufacturer    = manufacturer;
        batch.expiry_date     = expiry_date;
        batch.ipfs_hash       = ipfs_hash;
        batch.min_temp        = min_temp;
        batch.max_temp        = max_temp;
        batch.current_owner   = ctx.accounts.authority.key();
        batch.is_compromised  = false;
        batch.custody_count   = 0;
        batch.created_at      = Clock::get()?.unix_timestamp;

        emit!(BatchRegistered {
            batch_id: batch.batch_id.clone(),
            manufacturer: batch.manufacturer.clone(),
            timestamp: batch.created_at,
        });

        msg!("Batch {} registered on-chain.", batch.batch_id);
        Ok(())
    }

    // ----------------------------------------------------------
    // MODULE 2: Transfer custody (Manufacturer→Distributor→Pharmacy)
    // ----------------------------------------------------------
    pub fn transfer_custody(
        ctx: Context<TransferCustody>,
        new_owner_name: String,   // human-readable role name
        location: String,
    ) -> Result<()> {
        let batch = &mut ctx.accounts.drug_batch;

        // Only current owner can transfer
        require!(
            batch.current_owner == ctx.accounts.authority.key(),
            ErrorCode::Unauthorized
        );
        require!(!batch.is_compromised, ErrorCode::BatchCompromised);

        let log = &mut ctx.accounts.custody_log;
        log.batch_id      = batch.batch_id.clone();
        log.from_owner    = batch.current_owner;
        log.to_owner      = ctx.accounts.new_owner.key();
        log.to_owner_name = new_owner_name.clone();
        log.location      = location.clone();
        log.timestamp     = Clock::get()?.unix_timestamp;

        // Update batch owner
        batch.current_owner  = ctx.accounts.new_owner.key();
        batch.custody_count += 1;

        emit!(CustodyTransferred {
            batch_id: batch.batch_id.clone(),
            to: new_owner_name,
            location,
            timestamp: log.timestamp,
        });

        msg!("Custody transferred. Count: {}", batch.custody_count);
        Ok(())
    }

    // ----------------------------------------------------------
    // MODULE 3: Log temperature (IoT agent calls this)
    // ----------------------------------------------------------
    pub fn log_temperature(
        ctx: Context<LogTemperature>,
        temperature: i32,        // in °C (multiply floats by 10 before sending)
        gps_location: String,
    ) -> Result<()> {
        let batch = &mut ctx.accounts.drug_batch;

        // Only current owner's device can log
        require!(
            batch.current_owner == ctx.accounts.authority.key(),
            ErrorCode::Unauthorized
        );

        let temp_log = &mut ctx.accounts.temp_log;
        temp_log.batch_id    = batch.batch_id.clone();
        temp_log.temperature = temperature;
        temp_log.location    = gps_location.clone();
        temp_log.timestamp   = Clock::get()?.unix_timestamp;

        // Check temperature breach
        let breach = temperature < batch.min_temp || temperature > batch.max_temp;
        temp_log.is_breach   = breach;

        if breach {
            batch.is_compromised = true;
            emit!(TemperatureBreach {
                batch_id: batch.batch_id.clone(),
                temperature,
                location: gps_location,
                timestamp: temp_log.timestamp,
            });
            msg!("ALERT: Temperature breach detected! Batch flagged as compromised.");
        } else {
            msg!("Temp log OK: {}°C", temperature);
        }

        Ok(())
    }

    // ----------------------------------------------------------
    // MODULE 4: Verify batch (Pharmacy / Patient — read only)
    // ----------------------------------------------------------
    pub fn verify_batch(ctx: Context<VerifyBatch>) -> Result<()> {
        let batch = &ctx.accounts.drug_batch;

        emit!(BatchVerified {
            batch_id:        batch.batch_id.clone(),
            drug_name:       batch.drug_name.clone(),
            manufacturer:    batch.manufacturer.clone(),
            current_owner:   batch.current_owner,
            is_compromised:  batch.is_compromised,
            custody_count:   batch.custody_count,
            timestamp:       Clock::get()?.unix_timestamp,
        });

        if batch.is_compromised {
            msg!("WARNING: This batch has a cold chain breach. Do NOT use.");
        } else {
            msg!("Batch {} is AUTHENTIC and safe.", batch.batch_id);
        }

        Ok(())
    }
}

// ============================================================
// ACCOUNT STRUCTS (on-chain data layout)
// ============================================================

#[account]
pub struct DrugBatch {
    pub batch_id:       String,   // 36 bytes
    pub drug_name:      String,   // 68 bytes
    pub manufacturer:   String,   // 68 bytes
    pub expiry_date:    String,   // 16 bytes
    pub ipfs_hash:      String,   // 68 bytes
    pub min_temp:       i32,      // 4 bytes
    pub max_temp:       i32,      // 4 bytes
    pub current_owner:  Pubkey,   // 32 bytes
    pub is_compromised: bool,     // 1 byte
    pub custody_count:  u8,       // 1 byte
    pub created_at:     i64,      // 8 bytes
}

#[account]
pub struct CustodyLog {
    pub batch_id:      String,    // 36 bytes
    pub from_owner:    Pubkey,    // 32 bytes
    pub to_owner:      Pubkey,    // 32 bytes
    pub to_owner_name: String,    // 68 bytes
    pub location:      String,    // 68 bytes
    pub timestamp:     i64,       // 8 bytes
}

#[account]
pub struct TempLog {
    pub batch_id:    String,      // 36 bytes
    pub temperature: i32,         // 4 bytes
    pub location:    String,      // 68 bytes
    pub is_breach:   bool,        // 1 byte
    pub timestamp:   i64,         // 8 bytes
}

// ============================================================
// CONTEXT STRUCTS (who can call what, account validation)
// ============================================================

#[derive(Accounts)]
#[instruction(batch_id: String)]
pub struct RegisterBatch<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 36 + 68 + 68 + 16 + 68 + 4 + 4 + 32 + 1 + 1 + 8 + 64,
        seeds = [b"batch", batch_id.as_bytes()],
        bump
    )]
    pub drug_batch: Account<'info, DrugBatch>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferCustody<'info> {
    #[account(mut)]
    pub drug_batch: Account<'info, DrugBatch>,
    #[account(
        init,
        payer = authority,
        space = 8 + 36 + 32 + 32 + 68 + 68 + 8 + 32,
    )]
    pub custody_log: Account<'info, CustodyLog>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: new owner's public key, validated by current_owner check
    pub new_owner: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LogTemperature<'info> {
    #[account(mut)]
    pub drug_batch: Account<'info, DrugBatch>,
    #[account(
        init,
        payer = authority,
        space = 8 + 36 + 4 + 68 + 1 + 8 + 32,
    )]
    pub temp_log: Account<'info, TempLog>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyBatch<'info> {
    pub drug_batch: Account<'info, DrugBatch>,
}

// ============================================================
// EVENTS (emitted on-chain, read by frontend)
// ============================================================

#[event]
pub struct BatchRegistered {
    pub batch_id:     String,
    pub manufacturer: String,
    pub timestamp:    i64,
}

#[event]
pub struct CustodyTransferred {
    pub batch_id:  String,
    pub to:        String,
    pub location:  String,
    pub timestamp: i64,
}

#[event]
pub struct TemperatureBreach {
    pub batch_id:    String,
    pub temperature: i32,
    pub location:    String,
    pub timestamp:   i64,
}

#[event]
pub struct BatchVerified {
    pub batch_id:       String,
    pub drug_name:      String,
    pub manufacturer:   String,
    pub current_owner:  Pubkey,
    pub is_compromised: bool,
    pub custody_count:  u8,
    pub timestamp:      i64,
}

// ============================================================
// ERRORS
// ============================================================

#[error_code]
pub enum ErrorCode {
    #[msg("You are not the current owner of this batch.")]
    Unauthorized,
    #[msg("This batch has been compromised due to a temperature breach.")]
    BatchCompromised,
    #[msg("String exceeds maximum allowed length.")]
    StringTooLong,
}
