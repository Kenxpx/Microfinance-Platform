use scrypto_test::prelude::*;
use microfinance_platform::hello_test::*;

#[test]
fn test_hello_manifest_flow() {
    // Initialize simulator environment
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Create a new test account
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Compile and publish the package
    let package_address = ledger.compile_and_publish(this_package!());

    // === Test instantiate_hello function ===
    let instantiate_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Hello",
            "instantiate_hello",
            manifest_args!(),
        )
        .build();

    let instantiate_receipt = ledger.execute_manifest(
        instantiate_manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("Instantiate Receipt:\n{:?}\n", instantiate_receipt);
    let component = instantiate_receipt.expect_commit(true).new_component_addresses()[0];

    // === Test free_token method ===
    let token_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "free_token", manifest_args!())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let token_receipt = ledger.execute_manifest(
        token_manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("Free Token Receipt:\n{:?}\n", token_receipt);
    token_receipt.expect_commit_success();
}

#[test]
fn test_hello_with_test_environment() -> Result<(), RuntimeError> {
    // Initialize test environment
    let mut env = TestEnvironment::new();

    // Compile and publish the package with fast compile profile
    let package_address = PackageFactory::compile_and_publish(
        this_package!(),
        &mut env,
        CompileProfile::Fast,
    )?;

    // Instantiate Hello component via test abstraction
    let mut hello = Hello::instantiate_hello(package_address, &mut env)?;

    // Call the free_token method
    let bucket = hello.free_token(&mut env)?;

    // Check that the amount in the bucket is 1
    let amount = bucket.amount(&mut env)?;
    assert_eq!(amount, dec!("1"));

    Ok(())
}
