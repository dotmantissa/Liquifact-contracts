# Escrow Data Model

## Invoice Identifiers

Every escrow is indexed by a unique `invoice_id`, which is a Soroban `Symbol`.

### Validation Rules

To ensure compatibility with indexers and stable URL routing in off-chain dashboard systems, `invoice_id` strings provided at `init` are strictly validated:

1.  **Length**: 1 to 32 bytes.
2.  **Charset**: `[A-Za-z0-9_]` (ASCII alphanumeric and underscores).
3.  **Soroban Symbol Compatibility**: The allowed charset is a strict subset of what the Soroban platform allows for Symbols, ensuring no encoding ambiguities.

### Security Invariants

- **Bounds Handling**: The contract uses a fixed-size stack buffer (32 bytes) for validation. Conversion to `Symbol` uses a slice matching the exact input length, preventing null-byte leakage or uninitialized memory read into the persistent state.
- **Fail-Fast**: Any violation of length or charset rules results in an immediate contract trap, preventing the creation of malformed or "un-indexable" escrows.
- **Immutability**: Once initialized, the `invoice_id` is part of the immutable `InvoiceEscrow` state stored in the contract instance's persistent storage.

## Related Components

- `LiquifactEscrow::init`: The primary entrypoint where validation occurs.
- `DataKey::Escrow`: Stores the validated metadata.
