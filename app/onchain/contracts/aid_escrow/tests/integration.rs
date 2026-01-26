#![cfg(test)]

use aid_escrow::{AidEscrow, AidEscrowClient, Error, PackageStatus};
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, token};

// Helper to setup a token for testing solvency checks
fn setup_token(env: &Env, admin: &Address) -> token::Client<'static> {
    let token_contract = env.register_stellar_asset_contract(admin.clone());
    token::Client::new(env, &token_contract)
}

#[test]
fn test_integration_flow() {
    let env = Env::default();
    // Mock all auths to focus on logic flow rather than signing in integration test
    env.mock_all_auths();

    // Setup
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    // 1. Setup Token and Fund Admin
    let token_client = setup_token(&env, &token_admin);
    token_client.mint(&admin, &10_000);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    // 2. Initialize contract
    client.init(&admin);
    assert_eq!(client.get_admin(), admin);

    // 3. Fund the contract (Required for solvency check)
    // Transfer 5000 from Admin -> Contract Pool
    client.fund(&token_client.address, &admin, &5000);

    // 4. Create package
    // Note: ID is now manual, expires_at is absolute timestamp
    let pkg_id = 0;
    let expires_at = env.ledger().timestamp() + 86400; // 1 day from now
    
    let returned_id = client.create_package(
        &pkg_id, 
        &recipient, 
        &1000, 
        &token_client.address, 
        &expires_at
    );
    assert_eq!(returned_id, pkg_id);

    // 5. Verify package details
    // get_package now returns the Package directly (panics on error in default client config)
    let package = client.get_package(&pkg_id);
    assert_eq!(package.recipient, recipient);
    assert_eq!(package.amount, 1000);
    assert_eq!(package.token, token_client.address);
    assert_eq!(package.status, PackageStatus::Created);

    // 6. Claim package (recipient auth required)
    client.claim(&pkg_id);

    // 7. Verify claimed state
    let package = client.get_package(&pkg_id);
    assert_eq!(package.status, PackageStatus::Claimed);

    // 8. Verify funds moved
    // Recipient should have 1000
    assert_eq!(token_client.balance(&recipient), 1000);
    // Contract should have 4000 remaining
    assert_eq!(token_client.balance(&contract_id), 4000);
}

#[test]
fn test_multiple_packages() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_client = setup_token(&env, &token_admin);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    client.init(&admin);
    
    // Fund contract with enough for both packages
    token_client.mint(&admin, &10_000);
    client.fund(&token_client.address, &admin, &5000);

    // Create multiple packages with manual IDs
    let id1 = 100;
    let id2 = 101;
    let expiry = env.ledger().timestamp() + 86400;

    client.create_package(&id1, &recipient1, &500, &token_client.address, &expiry);
    client.create_package(&id2, &recipient2, &1000, &token_client.address, &expiry);

    // Verify each package is independent
    let p1 = client.get_package(&id1);
    let p2 = client.get_package(&id2);

    assert_eq!(p1.recipient, recipient1);
    assert_eq!(p2.recipient, recipient2);
    assert_eq!(p1.amount, 500);
    assert_eq!(p2.amount, 1000);
}

#[test]
fn test_error_cases() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_client = setup_token(&env, &token_admin);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    client.init(&admin);
    
    // Fund context
    token_client.mint(&admin, &10000);
    client.fund(&token_client.address, &admin, &5000);

    // Test invalid amount (0)
    let result = client.try_create_package(&0, &recipient, &0, &token_client.address, &86400);
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));

    // Create valid package first to establish state
    let pkg_id = 1;
    client.create_package(&pkg_id, &recipient, &1000, &token_client.address, &86400);

    // Try to claim non-existent package
    let result = client.try_claim(&999);
    assert_eq!(result, Err(Ok(Error::PackageNotFound)));

    // Get non-existent package
    // try_get_package returns Result<Result<Package, Error>, ...>
    let result = client.try_get_package(&999);
    assert_eq!(result, Err(Ok(Error::PackageNotFound)));
}
