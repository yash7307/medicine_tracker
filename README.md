# Medicine Tracker

Medicine Tracker is a Rust-first capstone project for verifying medicine authenticity and cold-chain safety. It combines a Solana Anchor smart contract, a Rust HTTP verification API, a simulated Rust IoT cold-chain agent, MongoDB-ready batch storage, and a Next.js dashboard.

The current demo works locally with `DEMO-BATCH-001`. The Rust API returns the verified batch details, custody trail, and temperature logs, while the frontend presents the result in a pharmacy/patient-friendly dashboard.

## Features

- Rust Solana Anchor smart contract for registering drug batches, transferring custody, logging temperature, and verifying batches.
- Rust API backend with MongoDB support and a built-in demo fallback.
- Rust IoT agent that simulates cold-chain temperature readings and transaction logging.
- Next.js frontend for batch lookup, status display, custody timeline, temperature logs, IPFS certificate links, and Solana Explorer links.
- Demo batch ID: `DEMO-BATCH-001`.
- MongoDB Atlas-ready configuration through environment variables.

## Tech Stack

| Layer | Technology |
| --- | --- |
| Smart contract | Rust, Solana Anchor |
| Verification API | Rust, MongoDB Rust driver |
| IoT agent | Rust |
| Frontend | Next.js 14, React, TypeScript |
| Database | MongoDB Atlas or local MongoDB |
| Blockchain target | Solana devnet |

## Project Structure

```text
medicine_tracker/
  app/                         Next.js app router frontend
  demo_api/                    Rust verification API with MongoDB lookup
  docs/mongodb.md              MongoDB document shape and setup notes
  iot_agent/                   Rust simulated cold-chain sensor agent
  programs/medicine_tracker/   Solana Anchor smart contract
  tests/                       Anchor TypeScript tests
  Anchor.toml                  Anchor workspace config
  Cargo.toml                   Solana + IoT Rust workspace
  package.json                 Frontend scripts
```

## Quick Start

Install dependencies:

```bash
npm install
```

Start the Rust API:

```bash
npm run api
```

Start the frontend in another terminal:

```bash
npm run dev
```

Open:

```text
http://127.0.0.1:3000
```

Use this demo batch:

```text
DEMO-BATCH-001
```

## MongoDB Setup

The API uses demo fallback mode unless `MONGODB_URI` is set.

Create a local `.env` or set these in your shell:

```bash
MONGODB_URI=mongodb+srv://yashsahu7104_db_user:<db_password>@cluster0.y9sjxu8.mongodb.net/?appName=Cluster0
MONGODB_DATABASE=medicine_tracker
API_ADDRESS=127.0.0.1:8080
```

Do not commit real passwords. `.env` and `.env.local` are ignored.

Expected collection:

```text
medicine_tracker.batches
```

See [docs/mongodb.md](docs/mongodb.md) for the document format.

## API Endpoints

Health check:

```text
GET http://127.0.0.1:8080/health
```

Verify a batch:

```text
GET http://127.0.0.1:8080/api/batches/DEMO-BATCH-001
```

If MongoDB is configured, the API checks MongoDB first. If the batch is not found and the requested ID is `DEMO-BATCH-001`, it returns the built-in demo data.

## Verification Commands

Frontend:

```bash
npm run build
npm run typecheck
```

Rust Solana + IoT workspace:

```bash
cargo build
```

Rust API:

```bash
cargo test --manifest-path demo_api/Cargo.toml
cargo build --manifest-path demo_api/Cargo.toml
```

## Solana / Anchor Notes

Install Solana CLI and Anchor before deploying the smart contract.

```bash
anchor build
anchor deploy
anchor test
```

The app currently uses a valid placeholder devnet program ID for local compilation. For a real deployment, replace the program ID in:

- `programs/medicine_tracker/src/lib.rs`
- `Anchor.toml`

## IoT Agent

Run the simulated cold-chain logger:

```bash
cargo run -p iot_agent -- --batch-id DEMO-BATCH-001 --interval 30
```

The current agent simulates readings and transaction signatures. It is structured so real Solana client calls can be added later.

## Full Project Roadmap

- Add authenticated manufacturer, distributor, pharmacy, and patient roles.
- Add batch creation and custody-transfer APIs backed by MongoDB.
- Add QR code generation and scan flow.
- Connect the Rust API to deployed Solana accounts for live on-chain reads.
- Add IoT ingestion endpoint for temperature logs.
- Add admin dashboard for manufacturers and regulators.
- Add deployment configuration for frontend and Rust API.

## License

MIT
