#[cfg(test)]
mod tests {
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Symbol};

    use crate::{PadalaShieldContract, PadalaShieldContractClient};

    // -----------------------------------------------------------------------
    // Test 1 — Happy path: OFW locks funds, recipient releases successfully
    // -----------------------------------------------------------------------
    #[test]
    fn test_lock_and_release_happy_path() {
        let env       = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaShieldContract);
        let client      = PadalaShieldContractClient::new(&env, &contract_id);

        let sender    = Address::generate(&env);
        let recipient = Address::generate(&env);
        let amount    = 50_000_000_i128; // 5 XLM in stroops

        // OFW locks 5 XLM into escrow
        let escrow_id: Symbol = client.lock_funds(&sender, &recipient, &amount);

        // Recipient confirms receipt and releases funds
        client.release_funds(&recipient, &escrow_id);

        // The record should now show released = true
        let record = client.get_escrow(&escrow_id);
        assert!(record.released,  "escrow should be marked released");
        assert!(!record.refunded, "escrow should not be refunded");
        assert_eq!(record.amount, amount);
    }

    // -----------------------------------------------------------------------
    // Test 2 — Edge case: third-party cannot release an escrow they don't own
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "unauthorized")]
    fn test_unauthorized_release_panics() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaShieldContract);
        let client      = PadalaShieldContractClient::new(&env, &contract_id);

        let sender     = Address::generate(&env);
        let recipient  = Address::generate(&env);
        let intruder   = Address::generate(&env); // random third party
        let amount     = 10_000_000_i128;

        let escrow_id = client.lock_funds(&sender, &recipient, &amount);

        // Intruder tries to release — must panic
        client.release_funds(&intruder, &escrow_id);
    }

    // -----------------------------------------------------------------------
    // Test 3 — State verification: storage reflects correct values after lock
    // -----------------------------------------------------------------------
    #[test]
    fn test_storage_state_after_lock() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaShieldContract);
        let client      = PadalaShieldContractClient::new(&env, &contract_id);

        let sender    = Address::generate(&env);
        let recipient = Address::generate(&env);
        let amount    = 100_000_000_i128; // 10 XLM

        let escrow_id = client.lock_funds(&sender, &recipient, &amount);

        let record = client.get_escrow(&escrow_id);

        // Verify every stored field
        assert_eq!(record.sender,    sender);
        assert_eq!(record.recipient, recipient);
        assert_eq!(record.amount,    amount);
        assert!(!record.released,  "should not be released yet");
        assert!(!record.refunded,  "should not be refunded yet");

        // Counter should be 1 after the first lock
        let count = client.escrow_count();
        assert_eq!(count, 1u64);
    }

    // -----------------------------------------------------------------------
    // Test 4 — Sender refund: OFW reclaims funds before recipient confirms
    // -----------------------------------------------------------------------
    #[test]
    fn test_sender_can_refund() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaShieldContract);
        let client      = PadalaShieldContractClient::new(&env, &contract_id);

        let sender    = Address::generate(&env);
        let recipient = Address::generate(&env);
        let amount    = 30_000_000_i128;

        let escrow_id = client.lock_funds(&sender, &recipient, &amount);
        client.refund(&sender, &escrow_id);

        let record = client.get_escrow(&escrow_id);
        assert!(record.refunded,  "should be refunded");
        assert!(!record.released, "should not be released");
    }

    // -----------------------------------------------------------------------
    // Test 5 — Double-release guard: releasing an already-released escrow panics
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "already released")]
    fn test_double_release_panics() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaShieldContract);
        let client      = PadalaShieldContractClient::new(&env, &contract_id);

        let sender    = Address::generate(&env);
        let recipient = Address::generate(&env);
        let amount    = 20_000_000_i128;

        let escrow_id = client.lock_funds(&sender, &recipient, &amount);
        client.release_funds(&recipient, &escrow_id);

        // Second release on the same escrow — must panic
        client.release_funds(&sender, &escrow_id);
    }
}
