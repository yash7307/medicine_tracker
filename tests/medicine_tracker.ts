// ============================================================
// MEDICINE TRACKER — Anchor Tests (TypeScript)
// Run: anchor test
// ============================================================

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MedicineTracker } from "../target/types/medicine_tracker";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("medicine_tracker", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.MedicineTracker as Program<MedicineTracker>;

  // Test wallets
  const manufacturer  = provider.wallet;
  const distributor   = Keypair.generate();
  const pharmacy      = Keypair.generate();

  const BATCH_ID = "BATCH-2024-001";

  // Derive PDA for drug batch account
  const [drugBatchPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("batch"), Buffer.from(BATCH_ID)],
    program.programId
  );

  // ---- TEST 1: Register a drug batch ----
  it("registers a drug batch on-chain", async () => {
    await program.methods
      .registerBatch(
        BATCH_ID,
        "Covishield Vaccine",
        "Serum Institute of India",
        "2026-12-31",
        "QmXyz123AbcIpfsHashGoesHere",
        2,    // min 2°C
        8     // max 8°C
      )
      .accounts({
        drugBatch:     drugBatchPda,
        authority:     manufacturer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const batch = await program.account.drugBatch.fetch(drugBatchPda);
    assert.equal(batch.batchId, BATCH_ID);
    assert.equal(batch.drugName, "Covishield Vaccine");
    assert.equal(batch.isCompromised, false);
    assert.equal(batch.custodyCount, 0);
    console.log("  Batch registered:", batch.batchId);
    console.log("  Owner:", batch.currentOwner.toString());
  });

  // ---- TEST 2: Transfer custody to distributor ----
  it("transfers custody to distributor", async () => {
    const custodyLogAccount = Keypair.generate();

    // Airdrop SOL to distributor for gas
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(distributor.publicKey, 1e9)
    );

    await program.methods
      .transferCustody("Blue Dart Distributor", "Mumbai Warehouse")
      .accounts({
        drugBatch:    drugBatchPda,
        custodyLog:   custodyLogAccount.publicKey,
        authority:    manufacturer.publicKey,
        newOwner:     distributor.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([custodyLogAccount])
      .rpc();

    const batch = await program.account.drugBatch.fetch(drugBatchPda);
    assert.equal(batch.custodyCount, 1);
    assert.equal(batch.currentOwner.toString(), distributor.publicKey.toString());
    console.log("  Custody transferred to:", distributor.publicKey.toString().slice(0, 8) + "...");
  });

  // ---- TEST 3: Log a normal temperature ----
  it("logs a normal temperature reading", async () => {
    const tempLogAccount = Keypair.generate();

    await program.methods
      .logTemperature(
        50,             // 5.0°C stored as 50 (x10 to avoid floats)
        "19.0760,72.8777"  // Mumbai GPS
      )
      .accounts({
        drugBatch:     drugBatchPda,
        tempLog:       tempLogAccount.publicKey,
        authority:     distributor.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([tempLogAccount, distributor])
      .rpc();

    const log = await program.account.tempLog.fetch(tempLogAccount.publicKey);
    assert.equal(log.isBreach, false);
    console.log("  Temp logged:", log.temperature / 10, "°C | Breach:", log.isBreach);
  });

  // ---- TEST 4: Log a temperature breach ----
  it("detects a temperature breach and flags batch", async () => {
    const tempLogAccount = Keypair.generate();

    await program.methods
      .logTemperature(
        150,            // 15.0°C — too hot, breach!
        "19.0760,72.8777"
      )
      .accounts({
        drugBatch:     drugBatchPda,
        tempLog:       tempLogAccount.publicKey,
        authority:     distributor.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([tempLogAccount, distributor])
      .rpc();

    const log   = await program.account.tempLog.fetch(tempLogAccount.publicKey);
    const batch = await program.account.drugBatch.fetch(drugBatchPda);
    assert.equal(log.isBreach, true);
    assert.equal(batch.isCompromised, true);
    console.log("  BREACH detected! Batch compromised:", batch.isCompromised);
  });

  // ---- TEST 5: Verify batch (patient/pharmacy scan) ----
  it("verifies batch and returns full status", async () => {
    await program.methods
      .verifyBatch()
      .accounts({ drugBatch: drugBatchPda })
      .rpc();

    const batch = await program.account.drugBatch.fetch(drugBatchPda);
    console.log("  Verification result:");
    console.log("    Drug:", batch.drugName);
    console.log("    Manufacturer:", batch.manufacturer);
    console.log("    Custody transfers:", batch.custodyCount);
    console.log("    Compromised:", batch.isCompromised);

    assert.equal(batch.batchId, BATCH_ID);
  });
});
