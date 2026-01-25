#![cfg(test)]

use aid_escrow::{AidEscrow, AidEscrowClient, PackageStatus};
use soroban_sdk::{Address, Env, testutils::Address as _};

#[test]
fn test_integration_flow() {
    let env = Env::default();

    // Setup
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    // 1. Initialize contract
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);

    // 2. Create package (admin auth required)
    env.mock_all_auths();
    let package_id = client.create_package(&recipient, &1000, &token, &86400);
    assert_eq!(package_id, 0);

    // 3. Verify package details
    let package = client.get_package(&package_id).unwrap();
    assert_eq!(package.0, recipient);
    assert_eq!(package.1, 1000);
    assert_eq!(package.2, token);
    assert_eq!(package.3, PackageStatus::Created as u32);

    // 4. Claim package (recipient auth required)
    env.mock_all_auths();
    client.claim_package(&package_id);

    // 5. Verify claimed
    let package = client.get_package(&package_id).unwrap();
    assert_eq!(package.3, PackageStatus::Claimed as u32);

    // 6. Verify count
    assert_eq!(client.get_package_count(), 1);
}

#[test]
fn test_multiple_packages() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    client.initialize(&admin);
    env.mock_all_auths();

    // Create multiple packages
    let id1 = client.create_package(&recipient1, &500, &token, &3600);
    let id2 = client.create_package(&recipient2, &1000, &token, &7200);

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(client.get_package_count(), 2);

    // Verify each package is independent
    let p1 = client.get_package(&id1).unwrap();
    let p2 = client.get_package(&id2).unwrap();

    assert_eq!(p1.0, recipient1);
    assert_eq!(p2.0, recipient2);
    assert_eq!(p1.1, 500);
    assert_eq!(p2.1, 1000);
}

#[test]
#[test]
#[should_panic]
fn test_invalid_amount_panics() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    client.initialize(&admin);
    env.mock_all_auths();

    // Test invalid amount
    client.create_package(&recipient, &0, &token, &86400);
}

#[test]
#[should_panic]
fn test_claim_nonexistent_panics() {
    let env = Env::default();

    let admin = Address::generate(&env);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    client.initialize(&admin);
    env.mock_all_auths();

    // Try to claim non-existent package
    client.claim_package(&999);
}

#[test]
fn test_get_nonexistent_package() {
    let env = Env::default();

    let admin = Address::generate(&env);

    let contract_id = env.register(AidEscrow, ());
    let client = AidEscrowClient::new(&env, &contract_id);

    client.initialize(&admin);

    // Get non-existent package
    let result = client.get_package(&999);
    assert_eq!(result, None);
}
