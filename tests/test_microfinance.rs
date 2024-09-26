use scrypto::prelude::*;
use scrypto_unit::*;
use microfinance_platform::*;

#[test]
fn test_microfinance_platform() {
    // Set up environment
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _private_key, account) = test_runner.new_allocated_account();

    // Deploy components
    let package_address = test_runner.compile_and_publish(package_dir!());

    let ai_oracle = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "MockAIOracle", "new", args![])
            .build(),
        vec![],
    ).expect("Error creating AI Oracle").new_component_addresses()[0];

    let economic_oracle = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "MockEconomicOracle", "new", args![])
            .build(),
        vec![],
    ).expect("Error creating Economic Oracle").new_component_addresses()[0];

    let ai_credit_scorer = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "AICreditScorer", "new", args![ai_oracle])
            .build(),
        vec![],
    ).expect("Error creating AI Credit Scorer").new_component_addresses()[0];

    let interest_calculator = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "DynamicInterestCalculator", "new", args![Decimal::from(0.05), economic_oracle])
            .build(),
        vec![],
    ).expect("Error creating Interest Calculator").new_component_addresses()[0];

    let liquidity_pool = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "CrossChainLiquidityPool", "new", args![RADIX_TOKEN])
            .build(),
        vec![],
    ).expect("Error creating Liquidity Pool").new_component_addresses()[0];

    let insurance_pool = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "InsurancePool", "new", args![RADIX_TOKEN, Decimal::from(0.01)])
            .build(),
        vec![],
    ).expect("Error creating Insurance Pool").new_component_addresses()[0];

    let governance = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "ReputationGovernance", "new", args![])
            .build(),
        vec![],
    ).expect("Error creating Governance").new_component_addresses()[0];

    let impact_tracker = test_runner.execute_manifest(
        ManifestBuilder::new()
            .call_function(package_address, "ImpactTracker", "new", args![])
            .build(),
        vec![],
    ).expect("Error creating Impact Tracker").new_component_addresses()[0];

    // Deploy MicrofinancePool
    let (microfinance_pool, admin_badge) = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "MicrofinancePool", "new", args![
                RADIX_TOKEN,
                ai_credit_scorer,
                interest_calculator,
                liquidity_pool,
                insurance_pool,
                governance,
                impact_tracker
            ])
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        (receipt.expect("Error creating Microfinance Pool").new_component_addresses()[0],
         receipt.expect("Error creating Microfinance Pool").new_resource_addresses()[0])
    };

    // Test deposit
    let deposit_amount = dec!("1000");
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, RADIX_TOKEN, deposit_amount)
        .take_from_worktop(RADIX_TOKEN, deposit_amount, "deposit")
        .call_method(microfinance_pool, "deposit", args![Bucket("deposit"), RADIX_TOKEN])
        .call_method(account, "deposit_batch", args![Expression::entire_worktop()])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    assert!(receipt.is_ok());

    // Test request loan
    let loan_amount = dec!("500");
    let manifest = ManifestBuilder::new()
        .call_method(microfinance_pool, "request_loan", args![account, loan_amount, RADIX_TOKEN])
        .call_method(account, "deposit_batch", args![Expression::entire_worktop()])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    assert!(receipt.is_ok());

    // Test repay loan
    let repayment_amount = dec!("550"); // Assuming 10% interest
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, RADIX_TOKEN, repayment_amount)
        .take_from_worktop(RADIX_TOKEN, repayment_amount, "repayment")
        .call_method(microfinance_pool, "repay_loan", args![account, Bucket("repayment")])
        .call_method(account, "deposit_batch", args![Expression::entire_worktop()])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    assert!(receipt.is_ok());

    // Test liquidate overdue loans (admin function)
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account, admin_badge, 1)
        .call_method(microfinance_pool, "liquidate_overdue_loans", args![Proof("admin_proof")])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    assert!(receipt.is_ok());

    // Test impact metrics
    let manifest = ManifestBuilder::new()
        .call_method(impact_tracker, "get_impact_metrics", args![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    assert!(receipt.is_ok());
    let (total_loans, total_repayments, repayment_rate): (Decimal, Decimal, Decimal) = receipt.expect("Error getting impact metrics").output(1);
    assert!(total_loans > Decimal::zero());
    assert!(total_repayments > Decimal::zero());
    assert!(repayment_rate > Decimal::zero());
}
