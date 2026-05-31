import { NextResponse } from "next/server";

const DEMO_BATCH_ID = "DEMO-BATCH-001";

const demoBatch = {
  batchId: DEMO_BATCH_ID,
  drugName: "Covishield Vaccine",
  manufacturer: "Serum Institute of India",
  expiryDate: "2026-12-31",
  ipfsHash: "QmXyz123AbcIpfsHashGoesHere",
  currentOwner: "9yAQbF5gm3K9LzSq6Kb2bbmHnPdcYpkiY2Pobp25o9CZ",
  isCompromised: false,
  custodyCount: 3,
  createdAt: 1780224000,
  minTemp: 2,
  maxTemp: 8,
  custodyTrail: [
    {
      role: "Manufacturer",
      location: "Pune Factory",
      timestamp: "31 May, 09:10",
    },
    {
      role: "Distributor",
      location: "Mumbai Cold Hub",
      timestamp: "31 May, 12:45",
    },
    {
      role: "Pharmacy",
      location: "Delhi Store",
      timestamp: "31 May, 15:20",
    },
  ],
  temperatureLogs: [
    {
      label: "Factory dispatch",
      value: 4.8,
      status: "ok",
    },
    {
      label: "Highway transit",
      value: 5.6,
      status: "ok",
    },
    {
      label: "Cold room arrival",
      value: 4.1,
      status: "ok",
    },
  ],
};

export async function GET(
  request: Request,
  { params }: { params: { id: string } }
) {
  const id = params.id.toUpperCase();

  if (id === DEMO_BATCH_ID) {
    return NextResponse.json(demoBatch);
  }

  return NextResponse.json(
    { error: "Batch not found", demoBatchId: DEMO_BATCH_ID },
    { status: 404 }
  );
}
