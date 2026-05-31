use mongodb::{
    bson::doc,
    sync::{Client, Collection},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const DEMO_BATCH_ID: &str = "DEMO-BATCH-001";
const DEFAULT_DATABASE: &str = "medicine_tracker";
const BATCH_COLLECTION: &str = "batches";

#[derive(Clone)]
struct AppState {
    store: Option<MongoStore>,
}

#[derive(Clone)]
struct MongoStore {
    batches: Collection<BatchData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CustodyStep {
    role: String,
    location: String,
    timestamp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TemperatureLog {
    label: String,
    value: f32,
    status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchData {
    batch_id: String,
    drug_name: String,
    manufacturer: String,
    expiry_date: String,
    ipfs_hash: String,
    current_owner: String,
    is_compromised: bool,
    custody_count: u8,
    created_at: i64,
    min_temp: i32,
    max_temp: i32,
    custody_trail: Vec<CustodyStep>,
    temperature_logs: Vec<TemperatureLog>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    ok: bool,
    database: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: &'static str,
    demo_batch_id: &'static str,
}

fn main() -> std::io::Result<()> {
    let address = env::var("API_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let state = AppState {
        store: match MongoStore::from_env() {
            Ok(store) => {
                println!("MongoDB configured. Reading batches from collection '{BATCH_COLLECTION}'.");
                Some(store)
            }
            Err(message) => {
                println!("{message}");
                println!("Continuing with built-in demo batch fallback.");
                None
            }
        },
    };

    let listener = TcpListener::bind(&address)?;
    println!("Medicine Tracker Rust API running at http://{address}");
    println!("Demo endpoint: http://{address}/api/batches/{DEMO_BATCH_ID}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &state),
            Err(error) => eprintln!("Connection error: {error}"),
        }
    }

    Ok(())
}

impl MongoStore {
    fn from_env() -> Result<Self, String> {
        let uri = env::var("MONGODB_URI")
            .map_err(|_| "MONGODB_URI is not set, so MongoDB lookup is disabled.".to_string())?;
        let database_name = env::var("MONGODB_DATABASE").unwrap_or_else(|_| DEFAULT_DATABASE.to_string());
        let client = Client::with_uri_str(uri).map_err(|error| format!("MongoDB connection failed: {error}"))?;
        let batches = client.database(&database_name).collection::<BatchData>(BATCH_COLLECTION);

        Ok(Self { batches })
    }

    fn find_batch(&self, batch_id: &str) -> Option<BatchData> {
        self.batches
            .find_one(doc! { "batchId": batch_id })
            .run()
            .map_err(|error| eprintln!("MongoDB lookup failed: {error}"))
            .ok()
            .flatten()
    }
}

fn handle_connection(mut stream: TcpStream, state: &AppState) {
    let mut buffer = [0; 2048];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(bytes_read) => bytes_read,
        Err(error) => {
            eprintln!("Request read error: {error}");
            return;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let Some(request_line) = request.lines().next() else {
        write_response(&mut stream, 400, "Bad Request", "text/plain", "Bad request");
        return;
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    match (method, path) {
        ("OPTIONS", _) => write_response(&mut stream, 204, "No Content", "text/plain", ""),
        ("GET", "/health") => {
            let database = if state.store.is_some() { "configured" } else { "demo_fallback" };
            let body = serde_json::to_string(&HealthResponse { ok: true, database })
                .unwrap_or_else(|_| r#"{"ok":true,"database":"unknown"}"#.to_string());
            write_response(&mut stream, 200, "OK", "application/json", &body);
        }
        ("GET", path) if path.starts_with("/api/batches/") => {
            let batch_id = normalize_batch_id(path.trim_start_matches("/api/batches/"));
            match verify_batch(&batch_id, state.store.as_ref()) {
                Some(body) => write_response(&mut stream, 200, "OK", "application/json", &body),
                None => {
                    let body = serde_json::to_string(&ErrorResponse {
                        error: "Batch not found",
                        demo_batch_id: DEMO_BATCH_ID,
                    })
                    .unwrap_or_else(|_| r#"{"error":"Batch not found","demoBatchId":"DEMO-BATCH-001"}"#.to_string());
                    write_response(&mut stream, 404, "Not Found", "application/json", &body);
                }
            }
        }
        _ => write_response(
            &mut stream,
            404,
            "Not Found",
            "application/json",
            r#"{"error":"Not found"}"#,
        ),
    }
}

fn verify_batch(batch_id: &str, store: Option<&MongoStore>) -> Option<String> {
    if let Some(batch) = store.and_then(|store| store.find_batch(batch_id)) {
        return serde_json::to_string(&batch).ok();
    }

    if batch_id == DEMO_BATCH_ID {
        return serde_json::to_string(&demo_batch()).ok();
    }

    None
}

fn demo_batch() -> BatchData {
    BatchData {
        batch_id: DEMO_BATCH_ID.to_string(),
        drug_name: "Covishield Vaccine".to_string(),
        manufacturer: "Serum Institute of India".to_string(),
        expiry_date: "2026-12-31".to_string(),
        ipfs_hash: "QmXyz123AbcIpfsHashGoesHere".to_string(),
        current_owner: "9yAQbF5gm3K9LzSq6Kb2bbmHnPdcYpkiY2Pobp25o9CZ".to_string(),
        is_compromised: false,
        custody_count: 3,
        created_at: 1780224000,
        min_temp: 2,
        max_temp: 8,
        custody_trail: vec![
            CustodyStep {
                role: "Manufacturer".to_string(),
                location: "Pune Factory".to_string(),
                timestamp: "31 May, 09:10".to_string(),
            },
            CustodyStep {
                role: "Distributor".to_string(),
                location: "Mumbai Cold Hub".to_string(),
                timestamp: "31 May, 12:45".to_string(),
            },
            CustodyStep {
                role: "Pharmacy".to_string(),
                location: "Delhi Store".to_string(),
                timestamp: "31 May, 15:20".to_string(),
            },
        ],
        temperature_logs: vec![
            TemperatureLog {
                label: "Factory dispatch".to_string(),
                value: 4.8,
                status: "ok".to_string(),
            },
            TemperatureLog {
                label: "Highway transit".to_string(),
                value: 5.6,
                status: "ok".to_string(),
            },
            TemperatureLog {
                label: "Cold room arrival".to_string(),
                value: 4.1,
                status: "ok".to_string(),
            },
        ],
    }
}

fn normalize_batch_id(value: &str) -> String {
    value.trim().replace("%20", " ").to_uppercase()
}

fn write_response(stream: &mut TcpStream, status: u16, reason: &str, content_type: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.as_bytes().len()
    );

    if let Err(error) = stream.write_all(response.as_bytes()) {
        eprintln!("Response write error: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_batch_verifies() {
        let json = verify_batch(DEMO_BATCH_ID, None).expect("demo batch should exist");

        assert!(json.contains(r#""batchId":"DEMO-BATCH-001""#));
        assert!(json.contains(r#""drugName":"Covishield Vaccine""#));
        assert!(json.contains(r#""isCompromised":false"#));
    }

    #[test]
    fn unknown_batch_is_not_found() {
        assert!(verify_batch("UNKNOWN", None).is_none());
    }
}
