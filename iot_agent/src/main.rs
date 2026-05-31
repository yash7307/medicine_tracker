// ============================================================
// MEDICINE TRACKER — Rust IoT Cold Chain Agent
// Simulates a sensor device that reads temperature every 30s
// and writes a signed transaction to Solana blockchain.
// Run: cargo run -- --batch-id BATCH001 --interval 30
// ============================================================

use std::time::Duration;
use std::thread;

// NOTE: In a real project, add these to Cargo.toml:
// solana-sdk = "1.18"
// solana-client = "1.18"
// anchor-client = "0.29"
// rand = "0.8"
// clap = { version = "4", features = ["derive"] }

// ---- Simulated types (replace with real solana-sdk imports) ----
type Signature = String;

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub batch_id:    String,
    pub temperature: f32,        // Celsius
    pub humidity:    f32,        // %
    pub gps_lat:     f64,
    pub gps_lng:     f64,
    pub timestamp:   u64,
}

#[derive(Debug)]
pub struct IotAgent {
    pub batch_id:     String,
    pub wallet_path:  String,
    pub rpc_url:      String,
    pub min_temp:     f32,
    pub max_temp:     f32,
    pub interval_sec: u64,
}

impl IotAgent {
    pub fn new(batch_id: &str, interval_sec: u64) -> Self {
        IotAgent {
            batch_id:     batch_id.to_string(),
            wallet_path:  "~/.config/solana/id.json".to_string(),
            rpc_url:      "https://api.devnet.solana.com".to_string(),
            min_temp:     2.0,   // vaccines: 2–8°C cold chain
            max_temp:     8.0,
            interval_sec,
        }
    }

    // Simulate reading from a hardware sensor (DHT22 / DS18B20)
    // In production: use rppal crate for Raspberry Pi GPIO
    pub fn read_sensor(&self) -> SensorReading {
        // Simulate realistic cold chain temperature with occasional breach
        let base_temp: f32 = 5.0;
        let noise: f32 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() % 100) as f32 / 100.0 * 6.0 - 3.0;

        let temperature = base_temp + noise;

        SensorReading {
            batch_id:    self.batch_id.clone(),
            temperature,
            humidity:    60.0 + noise,
            gps_lat:     28.6139,  // Delhi coordinates (demo)
            gps_lng:     77.2090,
            timestamp:   std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    // Check if reading is a temperature breach
    pub fn is_breach(&self, reading: &SensorReading) -> bool {
        reading.temperature < self.min_temp || reading.temperature > self.max_temp
    }

    // Send temperature log transaction to Solana
    // In production: builds and signs real Anchor instruction
    pub fn send_to_chain(&self, reading: &SensorReading) -> Result<Signature, String> {
        // --- PRODUCTION CODE (uncomment when using real solana-sdk) ---
        // let payer = read_keypair_file(&self.wallet_path)
        //     .map_err(|e| format!("Failed to read wallet: {}", e))?;
        //
        // let client = RpcClient::new_with_commitment(
        //     self.rpc_url.clone(),
        //     CommitmentConfig::confirmed(),
        // );
        //
        // let program_id = Pubkey::from_str("YOUR_PROGRAM_ID").unwrap();
        // let temp_as_i32 = (reading.temperature * 10.0) as i32; // store as integer
        // let gps = format!("{},{}", reading.gps_lat, reading.gps_lng);
        //
        // Build Anchor instruction for log_temperature(temp_as_i32, gps)
        // let sig = client.send_and_confirm_transaction(&tx)?;
        // return Ok(sig.to_string());

        // --- SIMULATION (for testing without real Solana) ---
        let sim_sig = format!(
            "SIM_TX_{}_{}",
            reading.timestamp,
            reading.batch_id
        );
        Ok(sim_sig)
    }

    // Main loop — runs forever, logs every interval_sec seconds
    pub fn run(&self) {
        println!("IoT Agent started for batch: {}", self.batch_id);
        println!("RPC: {}", self.rpc_url);
        println!("Temp range: {}–{}°C", self.min_temp, self.max_temp);
        println!("Logging every {}s\n{}", self.interval_sec, "-".repeat(50));

        let mut log_count = 0u64;
        let mut breach_count = 0u64;

        loop {
            let reading = self.read_sensor();
            let breach = self.is_breach(&reading);

            if breach {
                breach_count += 1;
                println!(
                    "[BREACH] Temp: {:.1}°C | GPS: {},{} | Time: {}",
                    reading.temperature, reading.gps_lat, reading.gps_lng, reading.timestamp
                );
            } else {
                println!(
                    "[OK]     Temp: {:.1}°C | GPS: {},{} | Time: {}",
                    reading.temperature, reading.gps_lat, reading.gps_lng, reading.timestamp
                );
            }

            match self.send_to_chain(&reading) {
                Ok(sig) => println!("  -> TX: {}", sig),
                Err(e)  => println!("  -> ERROR sending tx: {}", e),
            }

            log_count += 1;
            println!(
                "  -> Total logs: {} | Breaches: {}\n",
                log_count, breach_count
            );

            thread::sleep(Duration::from_secs(self.interval_sec));
        }
    }
}

fn main() {
    // Parse CLI args manually (replace with clap for production)
    let args: Vec<String> = std::env::args().collect();

    let batch_id = args.iter()
        .position(|a| a == "--batch-id")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("BATCH001");

    let interval: u64 = args.iter()
        .position(|a| a == "--interval")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let agent = IotAgent::new(batch_id, interval);
    agent.run();
}

// ============================================================
// Cargo.toml dependencies for this file:
// [dependencies]
// solana-sdk    = "1.18"
// solana-client = "1.18"
// anchor-client = "0.29"
// clap          = { version = "4", features = ["derive"] }
// ============================================================
