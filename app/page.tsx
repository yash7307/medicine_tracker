"use client";

import { useEffect, useState } from "react";

type CustodyStep = {
  role: string;
  location: string;
  timestamp: string;
};

type TemperatureLog = {
  label: string;
  value: number;
  status: "ok" | "breach";
};

interface BatchData {
  batchId: string;
  drugName: string;
  manufacturer: string;
  expiryDate: string;
  ipfsHash: string;
  currentOwner: string;
  isCompromised: boolean;
  custodyCount: number;
  createdAt: number;
  minTemp: number;
  maxTemp: number;
  custodyTrail: CustodyStep[];
  temperatureLogs: TemperatureLog[];
}

const DEMO_BATCH_ID = "DEMO-BATCH-001";
const RUST_API_URL = process.env.NEXT_PUBLIC_API_URL ?? "";

async function fetchBatchFromChain(batchId: string): Promise<BatchData | null> {
  const normalizedId = batchId.trim().toUpperCase();

  try {
    const response = await fetch(`${RUST_API_URL}/api/batches/${encodeURIComponent(normalizedId)}`);
    if (!response.ok) return null;
    return (await response.json()) as BatchData;
  } catch (e) {
    console.error("Rust API fetch error:", e);
    return null;
  }
}

export default function VerifyPage() {
  const [batchId, setBatchId] = useState(DEMO_BATCH_ID);
  const [result, setResult] = useState<BatchData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    void verify(DEMO_BATCH_ID);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const verify = async (nextBatchId = batchId) => {
    const requestedId = nextBatchId.trim();
    if (!requestedId) {
      setError("Enter a batch ID or use the demo batch.");
      return;
    }

    setLoading(true);
    setError("");
    setResult(null);
    setBatchId(requestedId.toUpperCase());

    const data = await fetchBatchFromChain(requestedId);
    if (data) {
      setResult(data);
    } else {
      setError("Batch not found. Start the Rust API and use DEMO-BATCH-001 to see the complete demo flow.");
    }
    setLoading(false);
  };

  const fillDemo = () => {
    setBatchId(DEMO_BATCH_ID);
    void verify(DEMO_BATCH_ID);
  };

  return (
    <main className="page-shell">
      <section className="hero-band">
        <div className="hero-content">
          <div className="brand-row">
            <span className="brand-mark">MT</span>
            <span>Medicine Tracker</span>
          </div>
          <h1>Verify medicine authenticity in seconds.</h1>
          <p>
            Check batch registration, custody movement, and cold-chain temperature status from one clean dashboard.
          </p>
          <div className="search-panel">
            <label htmlFor="batch-id">Batch ID</label>
            <div className="search-row">
              <input
                id="batch-id"
                type="text"
                placeholder={DEMO_BATCH_ID}
                value={batchId}
                onChange={(e) => setBatchId(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && verify()}
              />
              <button className="primary-button" onClick={() => verify()} disabled={loading}>
                <span className="button-icon" aria-hidden="true">🔍</span>
                {loading ? "Checking…" : "Verify"}
              </button>
            </div>
            <button className="demo-button" onClick={fillDemo} disabled={loading}>
              Use demo batch: {DEMO_BATCH_ID}
            </button>
          </div>
        </div>

        <aside className="signal-panel" aria-label="Live status summary">
          <div>
            <span className="panel-label">Devnet status</span>
            <strong>Ready for demo</strong>
          </div>
          <div className="metric-grid">
            <Metric label="Custody hops" value={result?.custodyCount.toString() ?? "3"} />
            <Metric label="Temp range" value="2–8 °C" />
            <Metric label="Risk score" value={result?.isCompromised ? "High" : "Low"} />
          </div>
        </aside>
      </section>

      <section className="content-grid">
        <div className="result-area">
          {error && <div className="alert">{error}</div>}

          {!result && !error && (
            <div className="empty-state">
              <span className="empty-icon">QR</span>
              <h2>Start with the demo batch</h2>
              <p>Use {DEMO_BATCH_ID} to see a complete authentic batch result with custody and temperature logs.</p>
            </div>
          )}

          {result && <ResultCard result={result} />}
        </div>

        <aside className="side-panel">
          <h2>Demo batch ID</h2>
          <button className="copy-card" onClick={fillDemo}>
            <span>{DEMO_BATCH_ID}</span>
            <small>Click to verify</small>
          </button>
          <div className="mini-list">
            <span>Registered manufacturer</span>
            <span>Cold-chain logs included</span>
            <span>Solana Explorer link ready</span>
          </div>
        </aside>
      </section>
    </main>
  );
}

function ResultCard({ result }: { result: BatchData }) {
  return (
    <article className="result-card">
      <div className={result.isCompromised ? "status-strip danger" : "status-strip safe"}>
        <span className="status-dot" />
        <div>
          <strong>{result.isCompromised ? "Cold-chain breach detected" : "Authentic batch verified"}</strong>
          <p>{result.isCompromised ? "Do not dispense this medicine." : "Registration and cold-chain data look healthy."}</p>
        </div>
      </div>

      <div className="detail-grid">
        <Info label="Drug" value={result.drugName} />
        <Info label="Batch ID" value={result.batchId} />
        <Info label="Manufacturer" value={result.manufacturer} />
        <Info label="Expiry" value={result.expiryDate} />
        <Info label="Temperature window" value={`${result.minTemp}–${result.maxTemp} °C`} />
        <Info label="Registered" value={new Date(result.createdAt * 1000).toLocaleDateString("en-IN")} />
      </div>

      <div className="section-row">
        <div>
          <h2>Custody trail</h2>
          <div className="timeline">
            {result.custodyTrail.map((step) => (
              <div className="timeline-item" key={`${step.role}-${step.timestamp}`}>
                <span className="timeline-node" />
                <div>
                  <strong>{step.role}</strong>
                  <p>{step.location}</p>
                  <small>{step.timestamp}</small>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div>
          <h2>Temperature logs</h2>
          <div className="temp-list">
            {result.temperatureLogs.map((log) => (
              <div className="temp-item" key={log.label}>
                <span>{log.label}</span>
                <strong>{log.value.toFixed(1)} °C</strong>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="link-row">
        <a href={`https://ipfs.io/ipfs/${result.ipfsHash}`} target="_blank" rel="noreferrer">
          View certificate
        </a>
        <a href={`https://explorer.solana.com/address/${result.currentOwner}?cluster=devnet`} target="_blank" rel="noreferrer">
          Open Solana Explorer
        </a>
      </div>
    </article>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-tile">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
