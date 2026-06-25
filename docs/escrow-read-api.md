# Escrow Read API

Soroban read-only entry points on `LiquifactEscrow`. All functions take `env: Env` and are
view-only (no state mutation, no auth required).

---

## `get_funding_token() → Address`

**Storage key:** `DataKey::FundingToken`

Returns the SEP-41 token contract address bound to this escrow instance at `init`.

- **Immutable** — set once at `init`; cannot change after deploy.
- Panics with `"Funding token not set"` if called before `init`.
- This is the only token that `sweep_terminal_dust` may transfer to the treasury.

---

## `get_treasury() → Address`

**Storage key:** `DataKey::Treasury`

Returns the protocol treasury address that receives terminal dust sweeps.

- **Immutable** — set once at `init`; cannot change after deploy.
- Panics with `"Treasury not set"` if called before `init`.
- The treasury must authorize `sweep_terminal_dust`; the admin cannot sweep unless it is also the treasury.

---

## `get_registry_ref() → Option<Address>`

**Storage key:** `DataKey::RegistryRef`

Returns the optional registry contract address supplied at `init`, or `None` when absent.

### Non-authority model

`RegistryRef` is a **read-only discoverability hint** for off-chain indexers only.

- No on-chain logic in this contract reads or calls this address.
- Its presence **does not** prove registry membership; call the registry contract directly to
  verify any on-chain claim.
- `None` is a valid, fully operational state — registry integration is optional.
- The key is omitted from instance storage entirely when `registry = None` at `init`, so
  `get_registry_ref()` on an uninitialized contract also returns `None`.

---

## `get_escrow() → InvoiceEscrow`

**Storage key:** `DataKey::Escrow`

Returns the full escrow snapshot. Panics with `"Escrow not initialized"` before `init`.

---

## `get_version() → u32`

**Storage key:** `DataKey::Version`

Returns the current schema version (`SCHEMA_VERSION`). Returns `0` before `init`.

---

## `get_legal_hold() → bool`

**Storage key:** `DataKey::LegalHold`

Returns `true` when a compliance hold is active. Defaults to `false` when the key is absent.

---

## `has_maturity_lock() → bool`

Derived from `DataKey::Escrow.maturity`.

Returns `true` when `maturity > 0` and `settle()` is gated by
`Env::ledger().timestamp() >= maturity`. Returns `false` when `maturity == 0`,
which means there is no maturity time lock and a funded escrow can settle
immediately if SME auth, status, and legal-hold checks pass.

---

## `get_min_contribution_floor() → i128`

**Storage key:** `DataKey::MinContributionFloor`

Returns the per-call funding floor in token base units. `0` means no extra floor.

---

## `get_max_unique_investors_cap() → Option<u32>`

**Storage key:** `DataKey::MaxUniqueInvestorsCap`

Returns the optional cap on distinct investor addresses. `None` means unlimited.

---

## `get_max_per_investor_cap() → Option<i128>`

**Storage key:** `DataKey::MaxPerInvestorCap`

Returns the optional immutable cap on cumulative principal for a single investor. `None` means unlimited.

---

## `get_unique_funder_count() → u32`

**Storage key:** `DataKey::UniqueFunderCount`

Returns the count of distinct addresses that have contributed principal. Initialized to `0` at `init`.

---

## `get_contribution(investor: Address) → i128`

**Storage key:** `DataKey::InvestorContribution(investor)`

Returns the cumulative principal contributed by `investor`. `0` when absent.

---

## `get_funding_close_snapshot() → Option<FundingCloseSnapshot>`

**Storage key:** `DataKey::FundingCloseSnapshot`

Returns the pro-rata denominator snapshot captured when the escrow first became **funded** (status 1).
`None` until that transition. Immutable once written.

---

## `get_investor_yield_bps(investor: Address) → i64`

**Storage key:** `DataKey::InvestorEffectiveYield(investor)`

Returns the effective annualized yield (bps) locked in at the investor's first deposit.
Falls back to `InvoiceEscrow::yield_bps` when the key is absent (legacy positions).

---

## `get_investor_claim_not_before(investor: Address) → u64`

**Storage key:** `DataKey::InvestorClaimNotBefore(investor)`

Returns the earliest ledger timestamp at which the investor may call `claim_investor_payout`.
`0` means no extra gate beyond settled status.

---

## `get_sme_collateral_commitment() → Option<SmeCollateralCommitment>`

**Storage key:** `DataKey::SmeCollateralPledge`

Returns the SME collateral pledge metadata, or `None` when never recorded.

**Record-only:** this is not an enforced on-chain asset lock.

---

## `is_investor_claimed(investor: Address) → bool`

**Storage key:** `DataKey::InvestorClaimed(investor)`

Returns `true` when the investor has exercised `claim_investor_payout`. Defaults to `false`.

---

## `get_primary_attestation_hash() → Option<BytesN<32>>`

**Storage key:** `DataKey::PrimaryAttestationHash`

Returns the single-set 32-byte attestation digest, or `None` when unbound.

---

## `get_attestation_append_log() → Vec<BytesN<32>>`

**Storage key:** `DataKey::AttestationAppendLog`

Returns the append-only audit chain of digests. Returns an empty `Vec` when no entries exist.
Bounded by `MAX_ATTESTATION_APPEND_ENTRIES`.

---

## `get_escrow_summary() → EscrowSummary`

Bundles multiple read-only values in a single host invocation, optimizing read latency and gas efficiency for off-chain indexers and frontend rendering.

- **Pure Read** — view-only (no authorization required, no state writes).
- **Safe Fallback** — matches individual getters exactly, returning defaults when optional keys are absent, and does not panic unless the escrow itself is uninitialized.

### Return Type: `EscrowSummary`

A `#[contracttype]` struct containing:

- `escrow: InvoiceEscrow` — The full escrow snapshot.
- `has_maturity_lock: bool` — True when `escrow.maturity > 0`; false means `maturity == 0` and settlement has no maturity time lock.
- `legal_hold: bool` — True if a compliance hold is active.
- `funding_close_snapshot: EscrowCloseSnapshot` — Custom option-like representation of the captured pro-rata denominator snapshot (detailed below).
- `unique_funder_count: u32` — Distinct address count of contributors.
- `is_allowlist_active: bool` — True if the investor allowlist is active.
- `schema_version: u32` — The schema version of the contract state.
- `sme_collateral_commitment: Option<SmeCollateralCommitment>` — SME collateral pledge metadata, or `None` when never recorded.
- `has_primary_attestation: bool` — True when a primary attestation hash has been bound via `bind_primary_attestation_hash`.
- `attestation_log_length: u32` — Number of entries currently in the attestation append log.

### Sub-type: `EscrowCloseSnapshot`

A `#[contracttype]` enum representing the optional `FundingCloseSnapshot`:

- `None` — Escrow is not yet funded; no close snapshot exists.
- `Some(FundingCloseSnapshot)` — The pro-rata denominator snapshot captured when the escrow first transitioned to **funded**.

---

## `preview_fund(investor: Address, amount: i128) → u32`

**Pure read-only preview** of a deposit call. Runs the same precondition checks as
`fund()` in the exact same order, without requiring authorization or mutating state.

### Return values

| Code | Meaning |
|------|---------|
| `0`  | Deposit would be accepted by `fund()` |
| `>0` | The numeric [`EscrowError`](escrow-error-messages.md) code that `fund()` would raise first |

### Guard order (matches `fund_impl`)

| Order | Check | Error code |
|-------|-------|------------|
| 1 | Amount is positive | `FundingAmountNotPositive` (100) |
| 2 | Meets `min_contribution` floor (if configured) | `FundingBelowMinContribution` (101) |
| 3 | Escrow is initialized (reads `DataKey::Escrow`) | — (panics if uninitialized, matching `fund`) |
| 4 | No active legal hold | `LegalHoldBlocksFunding` (102) |
| 5 | Escrow status is open (0) | `EscrowNotOpenForFunding` (103) |
| 6 | Funding deadline not passed | `FundingDeadlinePassed` (164) |
| 7 | Allowlist gate (if active): investor is allowlisted | `InvestorNotAllowlisted` (104) |
| 8 | Investor contribution does not overflow | `InvestorContributionOverflow` (105) |
| 9 | Per-investor cap not exceeded (if configured) | `InvestorContributionExceedsCap` (106) |
| 10 | Unique-investor cap not reached (if configured, new investors only) | `UniqueInvestorCapReached` (107) |
| 11 | Total funded-amount does not overflow | `FundedAmountOverflow` (110) |

### Advisory

This is a **read-only preview**. The actual `fund()` call is the source of truth
and may still revert due to racing state changes (e.g. another transaction fills
the unique-investor cap or the admin closes funding between the preview and the
subsequent `fund()` call).

### Security

- **No `require_auth`** — the investor address is not required to sign.
- **No storage writes** — returns the first failing code without mutating state.
- **Advisory only** — callers must still handle `fund()` reverting on race conditions.
