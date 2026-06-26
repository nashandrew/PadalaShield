# PadalaShield 🛡️
> **Escrow-protected remittance for Filipino OFWs — on Stellar.**

---

## Problem

An OFW nurse in Riyadh sends ₱8,000 (~$140 USD) via a remittance agent every month. The agent charges 5–8% in fees, takes 2–3 days to settle, and the family in Bulacan has no proof the money is "on the way" until cash arrives at a pawnshop window. If the agent goes offline or misroutes the transfer, the OFW has zero recourse.

## Solution

PadalaShield replaces the agent with a Soroban smart contract escrow. The OFW locks USDC/XLM into the contract from any mobile wallet abroad. The contract holds funds on-chain and notifies the recipient via a lightweight web app. The recipient taps **"Confirm Received"** — the contract releases funds instantly to their Stellar wallet. If no confirmation arrives within 72 hours, the OFW can refund in one tap. Stellar settles in under 5 seconds for a fraction of a cent.

---

## Stellar Features Used

| Feature | Why |
|---|---|
| **XLM / USDC transfers** | Core value movement — sub-cent fees vs 5–8% agent cut |
| **Soroban smart contracts** | Trustless escrow logic enforced on-chain, no intermediary |
| **Trustlines** | Recipient opts into USDC before funds can land in their wallet |
| **Events** | Contract emits `LOCKED`, `RELEASED`, `REFUNDED` — frontend listens and pushes SMS alerts |

---

## Target Users

| | |
|---|---|
| **Senders** | ~2.2M OFWs in the Middle East & Asia; remit monthly; use GCash or Coins.ph already |
| **Recipients** | Families in provinces (Bulacan, Pampanga, Cebu); own a smartphone; receive via pawnshop today |
| **Pain** | ~₱400–640 lost per transaction in fees + 2–3 day wait + zero delivery confirmation |

---

## Core Feature (MVP) — Demo in < 2 minutes

```
[OFW abroad]  → calls lock_funds(sender, recipient, 50_000_000)  → escrow created on-chain (ESC1)
[Recipient]   → web app shows "₱2,800 is waiting for you" notification
[Recipient]   → taps Confirm → calls release_funds(recipient, ESC1) → funds land in wallet
[Explorer]    → Stellar Expert shows the on-chain trail — immutable proof of transfer
```

---

## Why This Wins

Remittances to the Philippines totalled **$38B in 2024** — the largest single source of foreign income for the country. PadalaShield cuts cost, adds delivery proof, and runs on infrastructure (GCash ↔ Stellar bridge) that already exists. Judges see a real, named user with a real monthly pain and a demo that closes end-to-end in 90 seconds.

---

## Optional Edge — AI Integration

An on-page AI assistant (powered by Claude) reads the OFW's Stellar transaction history and surfaces: "You usually send on the 15th. Your family hasn't confirmed last month's transfer. Want to resend?" — turning PadalaShield into a proactive financial companion.

---

## Vision & Purpose

PadalaShield is not a crypto experiment — it is infrastructure for the 10 million Filipinos who depend on remittances to pay rent, school fees, and medicine. By putting escrow logic on Stellar we remove the most exploitative layer of the global remittance stack and hand proof-of-payment back to the families who need it most.

---

## Prerequisites

- Rust `>=1.74` with `wasm32-unknown-unknown` target
- Stellar CLI `>=21.0.0`
- A Freighter wallet set to **Testnet**

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli
```

---

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled Wasm lands at:
```
target/wasm32-unknown-unknown/release/padala_shield.wasm
```

---

## Test

```bash
cargo test
```

Expect **5 passing tests**:
1. Happy path — lock + release
2. Unauthorized release panics
3. Storage state correct after lock
4. Sender refund works
5. Double-release guard panics

---

## Deploy to Testnet

```bash
# 1. Create and fund an identity
stellar keys generate --global my-key --network testnet
stellar keys fund my-key --network testnet

# 2. Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/padala_shield.wasm \
  --source my-key \
  --network testnet
```

Copy the `C...` Contract ID from the output.

Verify at: `https://stellar.expert/explorer/testnet/contract/<CONTRACT_ID>`

---

## Sample CLI Invocations

### Lock funds (OFW creates escrow)
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source my-key \
  --network testnet \
  -- lock_funds \
  --sender GABC...1234 \
  --recipient GXYZ...5678 \
  --amount 50000000
```

### Release funds (recipient confirms)
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source recipient-key \
  --network testnet \
  -- release_funds \
  --caller GXYZ...5678 \
  --escrow_id ESC1
```

### Refund (OFW reclaims if not confirmed)
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source my-key \
  --network testnet \
  -- refund \
  --sender GABC...1234 \
  --escrow_id ESC1
```

### Read escrow state
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- get_escrow \
  --escrow_id ESC1
```

---

## Timeline

| Phase | Scope |
|---|---|
| Day 1 | Soroban contract + 5 tests passing locally |
| Day 2 | Deploy to testnet; build React frontend with Freighter wallet connect |
| Day 3 | Wire events → SMS/push notifications; polish demo flow |
| Demo  | 90-second end-to-end: lock → notify → release → Explorer proof |

---

## License

MIT © 2026 PadalaShield Contributors
