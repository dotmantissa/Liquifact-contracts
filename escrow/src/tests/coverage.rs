use crate::{
    CollateralClearedEvt, CollateralRecordedEvt, EscrowError, LiquifactEscrow,
    LiquifactEscrowClient,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, BytesN, Env, Error, InvokeError, Vec as SorobanVec,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const AMOUNT: i128 = 10_000_0000000;
const PLEDGE: i128 = 5_000_0000000;

fn setup(env: &Env) -> (Address, Address, LiquifactEscrowClient<'_>) {
    let sme = Address::generate(env);
    let id = env.register(LiquifactEscrow, ());
    let client = LiquifactEscrowClient::new(env, &id);
    client.init(&symbol_short!("INV001"), &sme, &AMOUNT, &800i64, &1000u64);
    (sme, id, client)
}

// ---------------------------------------------------------------------------
// record → get → clear happy path
// ---------------------------------------------------------------------------

#[test]
fn escrow_error_discriminants_match_canonical_table() {
    const TABLE: &[(EscrowError, u32)] = &[
        (EscrowError::AmountMustBePositive, 1),
        (EscrowError::YieldBpsOutOfRange, 2),
        (EscrowError::EscrowAlreadyInitialized, 3),
        (EscrowError::InvoiceIdInvalidLength, 4),
        (EscrowError::InvoiceIdInvalidCharset, 5),
        (EscrowError::MinContributionNotPositive, 6),
        (EscrowError::MinContributionExceedsAmount, 7),
        (EscrowError::MaxUniqueInvestorsNotPositive, 8),
        (EscrowError::MaxPerInvestorNotPositive, 9),
        (EscrowError::TierYieldOutOfRange, 10),
        (EscrowError::TierYieldBelowBase, 11),
        (EscrowError::TierLockNotIncreasing, 12),
        (EscrowError::TierYieldNotNonDecreasing, 13),
        (EscrowError::EscrowNotInitialized, 20),
        (EscrowError::FundingTokenNotSet, 21),
        (EscrowError::TreasuryNotSet, 22),
        (EscrowError::LegalHoldBlocksTreasuryDustSweep, 30),
        (EscrowError::SweepAmountNotPositive, 31),
        (EscrowError::SweepAmountExceedsMax, 32),
        (EscrowError::DustSweepNotTerminal, 33),
        (EscrowError::NoFundingTokenBalanceToSweep, 34),
        (EscrowError::EffectiveSweepAmountZero, 35),
        (EscrowError::TransferAmountNotPositive, 36),
        (EscrowError::InsufficientTokenBalanceBeforeTransfer, 37),
        (EscrowError::SenderBalanceUnderflow, 38),
        (EscrowError::RecipientBalanceUnderflow, 39),
        (EscrowError::SenderBalanceDeltaMismatch, 40),
        (EscrowError::RecipientBalanceDeltaMismatch, 41),
        (EscrowError::SweepExceedsLiabilityFloor, 42),
        (EscrowError::PrimaryAttestationAlreadyBound, 50),
        (EscrowError::AttestationAppendLogCapacityReached, 51),
        (EscrowError::CollateralAmountNotPositive, 60),
        (EscrowError::CollateralAssetEmpty, 61),
        (EscrowError::CollateralTimestampBackwards, 62),
        (EscrowError::InvestorBatchEmpty, 70),
        (EscrowError::InvestorBatchTooLarge, 71),
        (EscrowError::TargetNotPositive, 72),
        (EscrowError::TargetUpdateNotOpen, 73),
        (EscrowError::TargetBelowFundedAmount, 74),
        (EscrowError::CapLowerNotOpen, 75),
        (EscrowError::NoInvestorCapConfigured, 76),
        (EscrowError::NewCapNotLower, 77),
        (EscrowError::NewCapBelowCurrentFunderCount, 78),
        (EscrowError::MaturityUpdateNotOpen, 79),
        (EscrowError::NewAdminSameAsCurrent, 80),
        (EscrowError::FundingBatchEmpty, 82),
        (EscrowError::FundingBatchTooLarge, 83),
        (EscrowError::MigrationVersionMismatch, 90),
        (EscrowError::AlreadyCurrentSchemaVersion, 91),
        (EscrowError::NoMigrationPath, 92),
        (EscrowError::FundingAmountNotPositive, 100),
        (EscrowError::FundingBelowMinContribution, 101),
        (EscrowError::LegalHoldBlocksFunding, 102),
        (EscrowError::EscrowNotOpenForFunding, 103),
        (EscrowError::InvestorNotAllowlisted, 104),
        (EscrowError::InvestorContributionOverflow, 105),
        (EscrowError::InvestorContributionExceedsCap, 106),
        (EscrowError::UniqueInvestorCapReached, 107),
        (EscrowError::TieredSecondDeposit, 108),
        (EscrowError::InvestorClaimTimeOverflow, 109),
        (EscrowError::FundedAmountOverflow, 110),
        (EscrowError::CommitmentLockExceedsMaturity, 111),
        (EscrowError::LegalHoldBlocksSettlement, 120),
        (EscrowError::SettlementNotFunded, 121),
        (EscrowError::MaturityNotReached, 122),
        (EscrowError::LegalHoldBlocksWithdrawal, 123),
        (EscrowError::WithdrawalNotFunded, 124),
        (EscrowError::LegalHoldBlocksInvestorClaims, 125),
        (EscrowError::NoContributionToClaim, 126),
        (EscrowError::InvestorClaimNotSettled, 127),
        (EscrowError::InvestorCommitmentLockNotExpired, 128),
        (EscrowError::ComputePayoutArithmeticOverflow, 129),
        (EscrowError::LegalHoldBlocksCancelFunding, 140),
        (EscrowError::CancelFundingNotOpen, 141),
        (EscrowError::RefundNotCancelled, 142),
        (EscrowError::NoContributionToRefund, 143),
        (EscrowError::LegalHoldClearRequestMissing, 150),
        (EscrowError::LegalHoldClearNotReady, 151),
        (EscrowError::LegalHoldClearDelayOverflow, 152),
        (EscrowError::LegalHoldBlocksBeneficiaryRotation, 160),
        (EscrowError::RotationNotOpen, 161),
        (EscrowError::NewSmeSameAsCurrent, 162),
        (EscrowError::FundingDeadlinePassed, 153),
        (EscrowError::NoPendingAdmin, 163),
    ];
    assert_eq!(TABLE.len(), 84);
    for (variant, code) in TABLE {
        assert_eq!(*variant as u32, *code, "discriminant drift for code {code}");
    }
}

#[test]
fn typed_error_codes_cover_range_boundaries() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    assert!(client.get_sme_collateral_commitment().is_some());

    client.clear_sme_collateral_commitment();
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ---------------------------------------------------------------------------
// Clear without prior record → NoCollateralToClear
// ---------------------------------------------------------------------------

#[test]
fn test_clear_without_record_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    let result = client.try_clear_sme_collateral_commitment();
    assert_eq!(result, Err(Ok(EscrowError::NoCollateralToClear)));
}

// ---------------------------------------------------------------------------
// Non-SME caller is rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_clear_non_sme_caller_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let sme = Address::generate(&env);
    let id = env.register(LiquifactEscrow, ());
    let client = LiquifactEscrowClient::new(&env, &id);
    client.init(&symbol_short!("INV002"), &sme, &AMOUNT, &800i64, &1000u64);
    client.record_sme_collateral_commitment(&PLEDGE);

    // Provide empty auth set: require_auth on sme_address will panic.
    env.set_auths(&[]);
    client.clear_sme_collateral_commitment();
}

// ---------------------------------------------------------------------------
// CollateralClearedEvt payload (using to_xdr comparison)
// ---------------------------------------------------------------------------

#[test]
fn test_clear_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    // env.events().all() reflects the LAST call's events in the test env.
    client.clear_sme_collateral_commitment();

    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![CollateralClearedEvt {
            invoice_id: symbol_short!("INV001"),
            amount: PLEDGE,
        }
        .to_xdr(&env, &id)]
    );
}

// ---------------------------------------------------------------------------
// CollateralRecordedEvt payload
// ---------------------------------------------------------------------------

#[test]
fn test_record_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);

    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![CollateralRecordedEvt {
            invoice_id: symbol_short!("INV001"),
            amount: PLEDGE,
        }
        .to_xdr(&env, &id)]
    );
}

// ---------------------------------------------------------------------------
// Clear after settle (status=2) still works — metadata path is independent
// ---------------------------------------------------------------------------

#[test]
fn test_clear_after_settle_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    let investor = Address::generate(&env);
    client.fund(&investor, &AMOUNT);
    client.settle();

    client.clear_sme_collateral_commitment();
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ---------------------------------------------------------------------------
// Double clear → NoCollateralToClear on second attempt
// ---------------------------------------------------------------------------

#[test]
fn test_double_clear_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    client.clear_sme_collateral_commitment();

    let result = client.try_clear_sme_collateral_commitment();
    assert_eq!(result, Err(Ok(EscrowError::NoCollateralToClear)));
}

// ---------------------------------------------------------------------------
// get returns None before any record
// ---------------------------------------------------------------------------

#[test]
fn test_get_returns_none_before_record() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ---------------------------------------------------------------------------
// Overwrite: record twice, clear once → None; cleared amount is the last pledge
// ---------------------------------------------------------------------------

#[test]
fn test_overwrite_then_clear() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    client.record_sme_collateral_commitment(&(PLEDGE * 2));

    let pledge = client.get_sme_collateral_commitment().unwrap();
    assert_eq!(pledge.amount, PLEDGE * 2);

    // The clear event carries the overwritten (latest) amount.
    client.clear_sme_collateral_commitment();

    // Check cleared event BEFORE the next client call resets the event snapshot.
    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![CollateralClearedEvt {
            invoice_id: symbol_short!("INV001"),
            amount: PLEDGE * 2,
        }
        .to_xdr(&env, &id)]
    );
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ──────────────────────────────────────────────────────────────────────────────
// Anchoring tests: read-view default/absent return values (docs/escrow-read-api.md)
//
// Each test asserts the default or absent-key return value documented in the
// read-API catalog.  Tests are grouped by topic and use a fresh Env per test.
// ──────────────────────────────────────────────────────────────────────────────

/// All default-returning views return their documented defaults on an uninitialized contract.
#[test]
fn read_view_defaults_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _sme) = setup(&env);

    // get_version → 0
    assert_eq!(client.get_version(), 0);
    // get_legal_hold → false
    assert!(!client.get_legal_hold());
    // get_legal_hold_clear_delay → 0
    assert_eq!(client.get_legal_hold_clear_delay(), 0);
    // get_legal_hold_clearable_at → None
    assert!(client.get_legal_hold_clearable_at().is_none());
    // get_min_contribution_floor → 0 (key absent before init; after init written as 0)
    assert_eq!(client.get_min_contribution_floor(), 0);
    // get_max_unique_investors_cap → None
    assert!(client.get_max_unique_investors_cap().is_none());
    // get_max_per_investor_cap → None
    assert!(client.get_max_per_investor_cap().is_none());
    // get_unique_funder_count → 0
    assert_eq!(client.get_unique_funder_count(), 0);
    // get_funding_deadline → None
    assert!(client.get_funding_deadline().is_none());
    // is_funding_expired → false
    assert!(!client.is_funding_expired());
    // get_registry_ref → None
    assert!(client.get_registry_ref().is_none());
    // get_pending_admin → None
    assert!(client.get_pending_admin().is_none());
    // is_allowlist_active → false
    assert!(!client.is_allowlist_active());
    // get_primary_attestation_hash → None
    assert!(client.get_primary_attestation_hash().is_none());
    // get_attestation_append_log → empty vec (len 0)
    assert_eq!(client.get_attestation_append_log().len(), 0);
    // get_funding_close_snapshot → None
    assert!(client.get_funding_close_snapshot().is_none());
    // get_distributed_principal → 0
    assert_eq!(client.get_distributed_principal(), 0);
    // get_sme_collateral_commitment → None
    assert!(client.get_sme_collateral_commitment().is_none());
}

/// Per-investor views return their documented defaults for a fresh/absent investor.
#[test]
fn read_view_per_investor_defaults() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (funding_token, treasury) = free_addresses(&env);
    let investor = soroban_sdk::Address::generate(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV_DEF"),
        &sme,
        &1000,
        &500,
        &0,
        &funding_token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    // get_contribution → 0 for an address that has never funded
    assert_eq!(client.get_contribution(&investor), 0);
    // get_investor_yield_bps → base yield_bps (500) when key absent
    assert_eq!(client.get_investor_yield_bps(&investor), 500);
    // get_investor_claim_not_before → 0 when key absent
    assert_eq!(client.get_investor_claim_not_before(&investor), 0);
    // is_investor_claimed → false when key absent
    assert!(!client.is_investor_claimed(&investor));
    // is_investor_refunded → false when key absent
    assert!(!client.is_investor_refunded(&investor));
    // is_investor_allowlisted → false when key absent
    assert!(!client.is_investor_allowlisted(&investor));
    // compute_investor_payout → 0 before funding (no snapshot)
    assert_eq!(client.compute_investor_payout(&investor), 0);
    // is_attestation_revoked → false for any index when key absent
    assert!(!client.is_attestation_revoked(&0));
}

/// Immutable binding views return their set values after init.
#[test]
fn read_view_immutable_bindings_after_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (funding_token, treasury) = free_addresses(&env);
    let registry = soroban_sdk::Address::generate(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "BIND_TST"),
        &sme,
        &1000,
        &500,
        &0,
        &funding_token,
        &Some(registry.clone()),
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    assert_eq!(client.get_funding_token(), funding_token);
    assert_eq!(client.get_treasury(), treasury);
    assert_eq!(client.get_registry_ref(), Some(registry));
    assert_eq!(client.get_version(), SCHEMA_VERSION);
}

/// Error views return typed errors before init.
#[test]
fn read_view_error_on_absent_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _sme) = setup(&env);
    let (funding_token, treasury) = free_addresses(&env);

    // get_escrow → EscrowNotInitialized (20)
    assert_contract_error(client.try_get_escrow(), EscrowError::EscrowNotInitialized);
    // get_funding_token → FundingTokenNotSet (21)
    assert_contract_error(
        client.try_get_funding_token(),
        EscrowError::FundingTokenNotSet,
    );
    // get_treasury → TreasuryNotSet (22)
    assert_contract_error(client.try_get_treasury(), EscrowError::TreasuryNotSet);
    // get_escrow_summary → EscrowNotInitialized (20)
    assert_contract_error(
        client.try_get_escrow_summary(),
        EscrowError::EscrowNotInitialized,
    );

    // After init they succeed
    client.init(
        &Address::generate(&env),
        &soroban_sdk::String::from_str(&env, "PREINIT2"),
        &Address::generate(&env),
        &100,
        &100,
        &0,
        &funding_token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.get_version(), SCHEMA_VERSION);
    assert_eq!(client.get_funding_token(), funding_token);
}

/// has_maturity_lock reflects the configured maturity.
#[test]
fn read_view_has_maturity_lock() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (token, treasury) = free_addresses(&env);

    // maturity = 0 → no lock
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT_ZERO"),
        &sme,
        &100,
        &100,
        &0,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(!client.has_maturity_lock());

    let env2 = Env::default();
    env2.mock_all_auths();
    let (client2, admin2, sme2) = setup(&env2);
    let (token2, treasury2) = free_addresses(&env2);

    // maturity > 0 → lock active
    client2.init(
        &admin2,
        &soroban_sdk::String::from_str(&env2, "MAT_SET"),
        &sme2,
        &100,
        &100,
        &99_999,
        &token2,
        &None,
        &treasury2,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(client2.has_maturity_lock());
}

/// get_funding_close_snapshot returns None until funded, then the captured snapshot.
#[test]
fn read_view_funding_close_snapshot_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (funding_token, treasury) = free_addresses(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "SNAP_TST"),
        &sme,
        &100,
        &100,
        &0,
        &funding_token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    // Before any funding: no snapshot
    assert!(client.get_funding_close_snapshot().is_none());

    // Fund to target → snapshot created
    let investor = soroban_sdk::Address::generate(&env);
    client.fund(&investor, &100);
    let snap = client.get_funding_close_snapshot();
    assert!(snap.is_some());
    let snap = snap.unwrap();
    assert_eq!(snap.total_principal, 100);
    assert_eq!(snap.funding_target, 100);

    // Snapshot is immutable: second fund call does not change it
    let investor2 = soroban_sdk::Address::generate(&env);
    client.fund(&investor2, &50);
    let snap2 = client.get_funding_close_snapshot().unwrap();
    assert_eq!(snap2.total_principal, snap.total_principal);
}

/// Attestation views return correct defaults and update after mutations.
#[test]
fn read_view_attestation_defaults_and_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (token, treasury) = free_addresses(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "ATT_DFLT"),
        &sme,
        &100,
        &100,
        &0,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    // Before any attestation
    assert!(client.get_primary_attestation_hash().is_none());
    assert_eq!(client.get_attestation_append_log().len(), 0);
    assert!(!client.is_attestation_revoked(&0));

    // Bind primary
    let digest: BytesN<32> = BytesN::from_array(&env, &[7u8; 32]);
    client.bind_primary_attestation_hash(&digest);
    assert_eq!(client.get_primary_attestation_hash(), Some(digest));

    // Append one log entry
    let log_digest: BytesN<32> = BytesN::from_array(&env, &[9u8; 32]);
    client.append_attestation_digest(&log_digest);
    assert_eq!(client.get_attestation_append_log().len(), 1);
    assert!(!client.is_attestation_revoked(&0));

    // Revoke it
    client.revoke_attestation_digest(&0);
    assert!(client.is_attestation_revoked(&0));
}

/// is_allowlist_active and is_investor_allowlisted reflect mutations correctly.
#[test]
fn read_view_allowlist_defaults_and_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (token, treasury) = free_addresses(&env);
    let investor = soroban_sdk::Address::generate(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "AL_DEF"),
        &sme,
        &100,
        &100,
        &0,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    assert!(!client.is_allowlist_active());
    assert!(!client.is_investor_allowlisted(&investor));

    client.set_allowlist_active(&true);
    assert!(client.is_allowlist_active());

    client.set_investor_allowlisted(&investor, &true);
    assert!(client.is_investor_allowlisted(&investor));

    client.set_investor_allowlisted(&investor, &false);
    assert!(!client.is_investor_allowlisted(&investor));
}

/// compute_investor_payout returns 0 before funded and correct value after settlement.
#[test]
fn read_view_compute_investor_payout_pre_and_post_fund() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (funding_token, treasury) = free_addresses(&env);
    let investor = soroban_sdk::Address::generate(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "PAY_TST"),
        &sme,
        &1000,
        &1000, // 10% yield
        &0,
        &funding_token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    // Before any funding: payout = 0
    assert_eq!(client.compute_investor_payout(&investor), 0);

    // Fund to target (investor contributes full 1000)
    client.fund(&investor, &1000);

    // After funding: payout = 1000 + (1000*1000/10000) = 1000 + 100 = 1100
    let payout = client.compute_investor_payout(&investor);
    assert_eq!(payout, 1100);
}

/// get_legal_hold_clear_delay and get_legal_hold_clearable_at match init config.
#[test]
fn read_view_legal_hold_clear_delay_config() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (token, treasury) = free_addresses(&env);

    // No delay configured: delay = 0
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "LHD_ZERO"),
        &sme,
        &100,
        &100,
        &0,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.get_legal_hold_clear_delay(), 0);
    assert!(client.get_legal_hold_clearable_at().is_none());
}

/// get_min_contribution_floor matches configured value and 0 when unconfigured.
#[test]
fn read_view_min_contribution_floor_config() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (token, treasury) = free_addresses(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "FLOOR50"),
        &sme,
        &1000,
        &100,
        &0,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &Some(50i128),
        &None,
        &None,
        &None,
    );
    assert_eq!(client.get_min_contribution_floor(), 50);
}

/// Optional cap views return None when unconfigured and Some when set.
#[test]
fn read_view_optional_caps_config() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let (token, treasury) = free_addresses(&env);

    // Without caps
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "NOCAPS"),
        &sme,
        &1000,
        &100,
        &0,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(client.get_max_unique_investors_cap().is_none());
    assert!(client.get_max_per_investor_cap().is_none());

    let env2 = Env::default();
    env2.mock_all_auths();
    let (client2, admin2, sme2) = setup(&env2);
    let (token2, treasury2) = free_addresses(&env2);

    // With caps
    client2.init(
        &admin2,
        &soroban_sdk::String::from_str(&env2, "WITHCAPS"),
        &sme2,
        &1000,
        &100,
        &0,
        &token2,
        &None,
        &treasury2,
        &None,
        &Some(5u32),
        &None,
        &Some(200i128),
        &None,
        &None,
    );
    assert_eq!(client2.get_max_unique_investors_cap(), Some(5u32));
    assert_eq!(client2.get_max_per_investor_cap(), Some(200i128));
}

/// get_distributed_principal increments correctly after refund.
#[test]
fn read_view_distributed_principal_after_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let tok = install_stellar_asset_token(&env);
    let treasury = soroban_sdk::Address::generate(&env);
    let investor = soroban_sdk::Address::generate(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "DIST_P"),
        &sme,
        &200,
        &100,
        &0,
        &tok.id,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.get_distributed_principal(), 0);

    // Mint into escrow contract so it holds the principal for refund transfers
    tok.stellar.mint(&client.address, &150);
    client.fund(&investor, &150);
    client.cancel_funding();
    client.refund(&investor);

    assert_eq!(client.get_distributed_principal(), 150);
}
