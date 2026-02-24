# Encryption Rollout Plan (Non-Breaking)

## Goal
Add end-to-end payload encryption in phases **without breaking current working modules**.

## Current Reality
- Identity signing + verification exists.
- Handshake derives directional session keys.
- Transfer payload path is currently plaintext.

---

## Status Update
- ✅ E1 implemented (scaffold): `crates/crypto_envelope` added with chunk envelope APIs, nonce derivation, and unit tests; crypto backend can be swapped to AEAD in hardening pass.
- ✅ E2 implemented: `transfer` now supports protocol-versioned chunk frame decoding for v1 plaintext and v2 metadata-aware frames.
- ✅ E3 implemented: handshake now carries encryption capability fields and negotiates session encryption mode with fail-closed required behavior.
- ✅ E4 implemented: transfer now has additive encrypt/decrypt adapters for chunk frames using session keys; ACK/retry/resume semantics are unchanged.
- ✅ E5 implemented: integration tests now cover plaintext + encrypted compatibility, required-mode plaintext rejection, and security telemetry lifecycle signals.

---

## Design Principles (to avoid breakage)
1. **Backward compatible protocol envelope**: support plaintext (`v1`) and encrypted (`v2`) frames during migration.
2. **Feature flag gate**: runtime toggle `encryption_mode = off | optional | required`.
3. **Dual-read / single-write transition**:
   - Readers accept both plaintext + encrypted.
   - Writers start plaintext, then optional encrypted, finally required encrypted.
4. **No silent fallback in required mode**: fail clearly if peer cannot do encrypted mode.
5. **Preserve existing public APIs first**: add wrappers/adapters before refactoring internals.

---

## Module Plan

### E1) Crypto Envelope Module (new crate)
**Purpose**: chunk payload encrypt/decrypt + integrity tag.

Deliverables:
- `encrypt_chunk(session_tx_key, nonce, plaintext) -> ciphertext`
- `decrypt_chunk(session_rx_key, nonce, ciphertext) -> plaintext`
- AEAD algorithm selection (ChaCha20-Poly1305 preferred)
- nonce strategy (transfer_id + chunk_index + sender/receiver direction)

Non-breaking strategy:
- No transfer crate replacement yet.
- Add as standalone crate and unit tests only.

---

### E2) Protocol Versioning Module (transfer integration)
**Purpose**: add wire format to distinguish plaintext vs encrypted frames.

Deliverables:
- Frame header extension:
  - `protocol_version`
  - `encryption_flag`
  - `nonce`/`aad` metadata as needed
- decoder supports both v1 and v2

Non-breaking strategy:
- Keep existing `TransferChunk` decode path for v1.
- Add parallel v2 parse path; default writer still v1.

---

### E3) Handshake Capability Negotiation
**Purpose**: decide encryption mode per session safely.

Deliverables:
- capability fields in hello messages:
  - `supports_encryption`
  - `preferred_encryption_mode`
- negotiated result stored in session state

Non-breaking strategy:
- if `optional` and peer lacks support -> stay v1.
- if `required` and peer lacks support -> explicit handshake error.

---

### E4) Transfer Encryption Adapter
**Purpose**: use negotiated session keys to encrypt/decrypt chunks in data path.

Deliverables:
- sender adapter: `TransferChunk -> EncryptedChunkFrame`
- receiver adapter: `EncryptedChunkFrame -> plaintext chunk`
- keep ACK/retry/resume semantics unchanged

Non-breaking strategy:
- ACK/checkpoint logic remains index-based (unchanged behavior).
- only payload transformation layer changes.

---

### E5) Integration + Regression Safety
**Purpose**: ensure no break in LAN, relay, UI, telemetry flows.

Deliverables:
- integration tests:
  1) plaintext v1 path still works
  2) mixed-mode optional encryption works
  3) required encryption rejects non-supporting peer
  4) encrypted transfer resume after interruption
- telemetry events:
  - `encryption.negotiated`
  - `encryption.required_rejected_peer`

Non-breaking strategy:
- keep existing integration suite tests green while adding new cases.

---

## Rollout Phases
1. **Phase A (safe prep)**: E1 + E2 with writer default plaintext.
2. **Phase B (negotiation)**: E3 and optional encryption runtime mode.
3. **Phase C (active encryption)**: E4 enabled in optional mode for compatible peers.
4. **Phase D (hardening)**: E5, then switch default mode from `off` to `optional`.
5. **Phase E (strict security)**: allow `required` mode for production profile.

---

## Acceptance Criteria
- No regression in existing plaintext integration tests.
- Encrypted transfers pass chunk integrity + resume tests.
- Mixed-version peers interoperate in optional mode.
- Required mode fails closed (no plaintext leak).
- UI shows negotiated security state.

---

## Immediate Next Steps
1. Create crate `crates/crypto_envelope` (E1).
2. Add protocol header version fields to transfer frame definitions (E2).
3. Add handshake capability fields and negotiation result object (E3).
4. Add integration test matrix entries for plaintext/encrypted compatibility (E5).
