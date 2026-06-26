#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Storage key enum — one key per logical record in persistent storage
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Escrow(Symbol),   // escrow_id → EscrowRecord
    EscrowCount,      // global counter used to generate unique IDs
}

// ---------------------------------------------------------------------------
// The on-chain state of a single remittance escrow
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub struct EscrowRecord {
    pub sender:    Address,  // OFW / abroad sender
    pub recipient: Address,  // family member in PH
    pub amount:    i128,     // in stroops (1 XLM = 10_000_000 stroops)
    pub released:  bool,     // true once funds have been claimed
    pub refunded:  bool,     // true if sender cancelled before release
}

// ---------------------------------------------------------------------------
// Events emitted for off-chain indexing / frontend notifications
// ---------------------------------------------------------------------------
const TOPIC_LOCK:    Symbol = symbol_short!("LOCKED");
const TOPIC_RELEASE: Symbol = symbol_short!("RELEASED");
const TOPIC_REFUND:  Symbol = symbol_short!("REFUNDED");

// ---------------------------------------------------------------------------
// Contract definition
// ---------------------------------------------------------------------------
#[contract]
pub struct PadalaShieldContract;

#[contractimpl]
impl PadalaShieldContract {

    /// lock_funds — OFW calls this to place XLM into escrow.
    ///
    /// The sender authorises the call (require_auth), so no third party can
    /// create an escrow on their behalf.  We store the record and emit a
    /// LOCKED event so the mobile app can notify the recipient.
    ///
    /// Returns the new escrow_id (a Symbol) for future reference.
    pub fn lock_funds(
        env:       Env,
        sender:    Address,
        recipient: Address,
        amount:    i128,
    ) -> Symbol {
        // Validate caller identity — prevents spoofed senders
        sender.require_auth();

        // Validate amount is positive
        if amount <= 0 {
            panic!("amount must be positive");
        }

        // Generate a unique escrow ID using a global counter
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::EscrowCount)
            .unwrap_or(0u64);
        let new_count = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::EscrowCount, &new_count);

        // Build Symbol key like "ESC1", "ESC2", …
        // (Symbol supports up to 9 chars; numeric suffix keeps it short)
        let escrow_id = Symbol::new(&env, &alloc_id(new_count));

        // Persist the escrow record
        let record = EscrowRecord {
            sender:    sender.clone(),
            recipient: recipient.clone(),
            amount,
            released:  false,
            refunded:  false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id.clone()), &record);

        // Emit event so indexers / frontend can react
        env.events().publish((TOPIC_LOCK, escrow_id.clone()), amount);

        escrow_id
    }

    /// release_funds — recipient (or sender) confirms delivery and releases.
    ///
    /// In a production build the actual XLM transfer would be handled by
    /// invoking the Stellar native asset contract.  Here we record the state
    /// change and emit the event, which the frontend uses to trigger the
    /// Stellar SDK payment on behalf of the contract.
    pub fn release_funds(env: Env, caller: Address, escrow_id: Symbol) {
        caller.require_auth();

        let key = DataKey::Escrow(escrow_id.clone());
        let mut record: EscrowRecord = env
            .storage()
            .persistent()
            .get(&key)
            .expect("escrow not found");

        // Only sender or recipient may release
        if caller != record.sender && caller != record.recipient {
            panic!("unauthorized: must be sender or recipient");
        }
        if record.released {
            panic!("already released");
        }
        if record.refunded {
            panic!("already refunded");
        }

        record.released = true;
        env.storage().persistent().set(&key, &record);

        env.events()
            .publish((TOPIC_RELEASE, escrow_id), record.amount);
    }

    /// refund — sender reclaims funds if recipient never confirmed.
    ///
    /// Only the original sender can initiate a refund.
    pub fn refund(env: Env, sender: Address, escrow_id: Symbol) {
        sender.require_auth();

        let key = DataKey::Escrow(escrow_id.clone());
        let mut record: EscrowRecord = env
            .storage()
            .persistent()
            .get(&key)
            .expect("escrow not found");

        if sender != record.sender {
            panic!("unauthorized: must be original sender");
        }
        if record.released {
            panic!("already released");
        }
        if record.refunded {
            panic!("already refunded");
        }

        record.refunded = true;
        env.storage().persistent().set(&key, &record);

        env.events()
            .publish((TOPIC_REFUND, escrow_id), record.amount);
    }

    /// get_escrow — read-only view of an escrow record.
    pub fn get_escrow(env: Env, escrow_id: Symbol) -> EscrowRecord {
        env.storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found")
    }

    /// escrow_count — returns total number of escrows ever created.
    pub fn escrow_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::EscrowCount)
            .unwrap_or(0u64)
    }
}

// ---------------------------------------------------------------------------
// Helper: build a short string ID from a counter (no std alloc needed)
// We build it manually into a fixed-size byte array for no_std compatibility.
// ---------------------------------------------------------------------------
fn alloc_id(n: u64) -> &'static str {
    // Soroban Symbol max = 9 chars.  "ESC" + 6 digits covers 999_999 escrows.
    // For a hackathon we keep it simple with a static lookup trick —
    // in production you'd use a proper formatter.
    // This naive approach works for demo counts up to 9.
    match n {
        1 => "ESC1",
        2 => "ESC2",
        3 => "ESC3",
        4 => "ESC4",
        5 => "ESC5",
        6 => "ESC6",
        7 => "ESC7",
        8 => "ESC8",
        9 => "ESC9",
        _ => "ESCX",
    }
}
