# Secure Peer-to-Peer File Sharing Platform (AirDrop-like)

## 1) Product Goal
Build a **platform-independent**, **AirDrop-style** file sharing system with:
- Native-feeling UI inspired by Apple AirDrop
- Secure transfer over internet and offline LAN
- Unicast transfers with support for multiple receivers simultaneously
- End-to-end encryption
- Large file transfer reliability (resume/retry/chunking)
- No-internet local transfer mode for offline use

---

## 2) Core Functional Requirements

### Discovery & Connectivity
1. Discover nearby devices on local network without internet.
2. Optional internet relay mode for remote transfer when direct path is unavailable.
3. Cross-platform support: Windows, macOS, Linux (phase-1 desktop).

### Transfer Model
1. Unicast transfer to one or many selected receivers at once.
2. Support files/folders; preserve metadata (name, size, hash, timestamps).
3. Reliable transfer for large files using chunked streaming and resumable sessions.

### Security
1. Device identity using public/private key pairs.
2. Mutual authentication before transfer.
3. End-to-end encryption for payload and metadata.
4. Integrity verification with per-chunk and final file hash.

### UX (AirDrop-like)
1. Device cards showing avatar/name/status.
2. Drag-and-drop send flow.
3. Receiver consent prompt (Accept / Decline).
4. Real-time transfer progress and speed indicator.
5. Completed/failed transfer history.

---

## 3) High-Level Architecture

### A) Frontend (Desktop UI)
- **Recommended**: Tauri + React + TypeScript
- Why: Lightweight packaging, cross-platform, secure system API boundaries, fast UI iteration.

### B) Backend (Strong Core Service)
- **Recommended**: Rust service (embedded with Tauri or standalone daemon)
- Why: High performance for streaming/chunking, memory safety, strong crypto ecosystem.

### C) Networking Layer
- LAN discovery: mDNS/Bonjour + UDP broadcast fallback
- Session setup: Noise protocol or TLS 1.3 with pinned device keys
- Data transport:
  - Direct: QUIC (preferred) or TCP
  - Fallback: relay server when NAT traversal fails

### D) Security Layer
- Key management: Ed25519 (identity), X25519 (key exchange)
- Encryption: ChaCha20-Poly1305 or AES-256-GCM
- Hashing: BLAKE3/SHA-256 for chunk/file integrity

### E) Optional Cloud/Relay Services
- STUN/TURN/Relay for internet mode
- Presence bootstrap (minimal metadata)
- No file persistence on relay by default (pass-through streams only)

---

## 4) Proposed Technology Stack

### Frontend
- React + TypeScript
- TailwindCSS (for fast AirDrop-like visual system)
- Zustand/Redux Toolkit for state
- Tauri window + native notifications

### Backend/Core
- Rust
- Tokio async runtime
- Quinn (QUIC) / tokio-tungstenite fallback if needed
- serde + bincode/protobuf for protocol payloads

### Crypto
- libsodium / ring / rustls ecosystem
- age-like envelope encryption for files
- OS keychain integration:
  - macOS Keychain
  - Windows Credential Manager
  - Linux Secret Service

### Storage
- SQLite for local metadata, devices, transfer history
- File-based chunk cache for resumable transfers

### DevOps
- GitHub Actions CI
- Cross-platform build matrix
- Signing/notarization pipeline (later stage)

---

## 5) Independent Modules (Development Path)

Each module can be built and tested independently, then integrated.

1. **Device Identity Module**
   - Generate/store identity keys
   - Device fingerprint + trust model

2. **Local Discovery Module**
   - mDNS service advertisement and discovery
   - Device reachability + status updates

3. **Handshake & Session Security Module**
   - Mutual authentication
   - Session key derivation + rotation

4. **Transfer Protocol Module**
   - Chunked file stream
   - ACK/retry/resume logic
   - Multi-receiver transfer orchestration

5. **Large File Manager Module**
   - Chunk index, checkpointing
   - Pause/resume/cancel
   - Post-transfer assembly + validation

6. **LAN Offline Mode Module**
   - Zero-internet discovery/transfer path
   - Local-only policy enforcement

7. **Internet/NAT Traversal Module**
   - STUN/TURN integration
   - Relay fallback decision engine

8. **Desktop UI Module (AirDrop-style)**
   - Device grid/cards
   - Drag-drop sender panel
   - Incoming request modal
   - Transfer monitor dashboard

9. **Audit/Telemetry Module (Privacy-preserving)**
   - Local logs with user-controlled export
   - Metrics without content leakage

10. **Installer & Update Module**
   - Auto-update channels
   - Platform-specific packaging/signing

---

## 6) Roadmap (Milestones)

### Milestone 0 — Architecture & UX Blueprint (Week 1-2)
- Finalize protocol choices (QUIC + Noise/TLS)
- Finalize UI wireframes mirroring AirDrop interaction model
- Threat model and security acceptance criteria

**Deliverables**
- ADR documents
- Clickable UI prototype
- Protocol sequence diagrams

### Milestone 1 — Offline LAN MVP (Week 3-6)
- Identity keys
- LAN discovery
- One-to-one secure transfer
- Basic sender/receiver UI

**Success Criteria**
- Send/receive files over same LAN with no internet
- Encrypted transfer with integrity check

### Milestone 2 — Multi-Receiver + Large Files (Week 7-10)
- Unicast to multiple receivers concurrently
- Chunked/resumable transfers
- Progress + retry UX

**Success Criteria**
- 10GB+ transfer stability in LAN tests
- Resume after interruption

### Milestone 3 — Internet Mode (Week 11-14)
- NAT traversal
- Relay fallback
- Connection quality adaptation

**Success Criteria**
- Transfer works across different networks
- E2E encryption preserved in relay mode

### Milestone 4 — Hardening & Beta (Week 15-18)
- Security audit
- Performance tuning
- Cross-platform packaging/signing

**Success Criteria**
- Beta-ready installers for 3 OS
- No critical security findings

---

## 7) Development Readiness Check (Current Status)

### Are we ready to start development?
**Yes, for Milestone 0 and Milestone 1 planning work.**

**Not fully ready for implementation** until the issues below are closed.

### Must-resolve issues before coding core protocol
1. **Protocol decision is not final**
   - Need final choice: Noise vs TLS-only handshake details.
2. **Trust model is not finalized**
   - Decide TOFU only or TOFU + verification code/QR.
3. **Exact relay privacy policy missing**
   - Define what metadata relay can see and retain.
4. **Large-file constraints undefined**
   - Set target max file size, chunk size defaults, memory limits.
5. **No acceptance test matrix yet**
   - Need deterministic pass/fail criteria per OS and network condition.
6. **No threat model document yet**
   - Required before internet mode implementation.

### Full-readiness definition (what is needed)
Project is considered **fully ready for implementation** only when all of the following are complete:

- [ ] ADR-001 approved: LAN discovery + local transport decision finalized.
- [ ] ADR-002 approved: authentication handshake, identity verification, and key rotation finalized.
- [ ] ADR-003 approved: transfer protocol, chunking, resume, retry, and multi-receiver behavior finalized.
- [ ] Threat model v1 approved with mitigation mapping to implementation tasks.
- [ ] Security baseline mapped to concrete test cases (MITM, replay, tamper, path traversal).
- [ ] Test matrix defined for OS/network/file-size scenarios with pass/fail thresholds.
- [ ] Performance budgets defined (throughput, memory ceiling, CPU envelope).
- [ ] Privacy policy finalized for relay metadata retention and log retention.
- [ ] CI baseline ready (lint + unit tests + protocol simulation tests).
- [ ] Milestone 1 backlog created with owners, estimates, and dependencies.

When all checkboxes are complete, implementation can begin with low project risk.

### Entry criteria to begin Milestone 1 coding
- ADR-001 discovery and local transport approved.
- ADR-002 session handshake and key lifecycle approved.
- ADR-003 transfer protocol (chunking/resume) approved.
- Threat model v1 published in `/docs/threat-model`.
- Local test plan created (LAN-only, no internet).

---

## 8) Security Baseline (Must-Have)

1. End-to-end encrypted payloads; relay cannot read contents.
2. Signed handshake messages to prevent MITM.
3. Trust-on-first-use + explicit device trust controls.
4. Strict file validation before write (path traversal prevention).
5. Rate limiting and anti-abuse controls on discovery and session requests.
6. Secure temporary storage cleanup after completed/failed transfer.

---

## 9) Suggested Repository Structure

```text
/apps
  /desktop-ui           # React/Tauri frontend
/services
  /core-transfer        # Rust transfer engine
  /relay                # Optional relay service
/crates
  /crypto
  /discovery
  /protocol
  /storage
/docs
  /adr
  /threat-model
  /api
  /test-matrix
```

---

## 10) Module Execution Order + Continuous Testing Rule

To satisfy incremental delivery, we implement **one module at a time** and require a module-level test pass before moving forward.

### Continuous testing rule (applies to every module)
1. Define module API + threat assumptions.
2. Implement module code.
3. Run module-scoped tests (`cargo test -p <module>` or equivalent).
4. Add/adjust negative tests for failure paths.
5. Only then mark module as complete and begin the next module.

### Current module status
- ✅ Module 1 complete: **Device Identity Module** (key generation, save/load, fingerprint).
- ✅ Module 2 implemented: **Local Discovery Module** (announcement encode/decode, UDP announce/receive, peer expiry registry).
- ✅ Module 3 implemented: **Handshake & Session Security Module** (signed client/server hello, replay guard, directional session key derivation).
- ✅ Module 4 implemented: **Transfer Protocol Module** (chunk framing, ACK checkpointing, resume cursor, multi-receiver progress).
- ✅ Module 5 implemented: **Large File Manager Module** (chunk index, checkpoint persistence, pause/resume/cancel state machine, file assembly + integrity tag).
- ✅ Module 6 implemented: **LAN Offline Mode Module** (local-network policy guard with private/link-local allowlist and public-address deny rules).
- ✅ Module 7 implemented: **Internet/NAT Traversal Module** (candidate set model, direct-vs-relay decision engine, hole-punch gating rules).
- ✅ Module 8 implemented: **Desktop UI Module** (device-card grid state, incoming-request modal flow, transfer dashboard state transitions).
- ✅ Module 9 implemented: **Audit/Telemetry Module** (local structured logs, redaction helper, counters, retention policy, export).
- ✅ Module 10 implemented: **Installer & Update Module** (manifest validation, update policy decisions, rollback markers).
- ⚠️ Module tests are in place and should run with network-enabled Cargo dependency access.

### Next module to build now
- **Integration Milestone: Cross-module wiring and end-to-end scenarios**
  - Connect discovery + handshake + transfer + UI state flow.
  - Add integration tests for offline LAN and relay fallback paths.
  - Validate security and telemetry events across full transfer lifecycle.

### Integration milestone progress
- ✅ Cross-module wiring prototype: discovery -> LAN policy -> transfer -> desktop UI state.
- ✅ E2E scenario coverage added for LAN-direct and relay-fallback route decisions.
- ✅ Lifecycle validation added for security + telemetry redaction behavior.


### Definition of done for Module 2
- Discovery service can announce local device identity.
- Two peers on same LAN can discover each other without internet.
- Discovery entries expire correctly when peer goes offline.
- Module test suite includes:
  - unit tests for packet parsing/validation,
  - integration tests for local announce/discover cycle,
  - timeout/expiry behavior tests.

## 11) Next Immediate Actions (1-Week Execution Plan)
1. Approve stack: **Tauri + React + Rust + QUIC**.
2. Create ADR-001/002/003 (discovery, handshake, transfer protocol).
3. Write threat model v1 with attacker assumptions and mitigations.
4. Define test matrix (OS x network scenarios x file sizes).
5. Build Milestone-1 spike: local discovery + encrypted one-file transfer.
6. Implement AirDrop-like UI skeleton connected to mock backend events.

This document is the foundation roadmap. Once approved, each module should get a dedicated spec with API contracts and test scenarios.
