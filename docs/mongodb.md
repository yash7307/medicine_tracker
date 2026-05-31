# MongoDB setup

The Rust API reads MongoDB configuration from environment variables.

```powershell
$env:MONGODB_URI="mongodb+srv://yashsahu7104_db_user:<db_password>@cluster0.y9sjxu8.mongodb.net/?appName=Cluster0"
$env:MONGODB_DATABASE="medicine_tracker"
cargo run --manifest-path demo_api/Cargo.toml
```

Expected collection:

`medicine_tracker.batches`

Example document:

```json
{
  "batchId": "DEMO-BATCH-001",
  "drugName": "Covishield Vaccine",
  "manufacturer": "Serum Institute of India",
  "expiryDate": "2026-12-31",
  "ipfsHash": "QmXyz123AbcIpfsHashGoesHere",
  "currentOwner": "9yAQbF5gm3K9LzSq6Kb2bbmHnPdcYpkiY2Pobp25o9CZ",
  "isCompromised": false,
  "custodyCount": 3,
  "createdAt": 1780224000,
  "minTemp": 2,
  "maxTemp": 8,
  "custodyTrail": [
    {
      "role": "Manufacturer",
      "location": "Pune Factory",
      "timestamp": "31 May, 09:10"
    }
  ],
  "temperatureLogs": [
    {
      "label": "Factory dispatch",
      "value": 4.8,
      "status": "ok"
    }
  ]
}
```

If `MONGODB_URI` is not set, the API still serves the built-in demo batch.
