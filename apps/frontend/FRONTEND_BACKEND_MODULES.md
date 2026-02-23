# Frontend ↔ Backend Integration Modules

This plan adds backend integration incrementally so existing UI behavior keeps working while real services are wired in.

## BI-1: Discovery API Wiring (implemented)
- Add a minimal backend HTTP service endpoint `GET /api/v1/discovery/devices`.
- Update frontend discovery scan to fetch devices from backend instead of hardcoded timeout path.
- Keep existing UI states (`loading/ready/empty/error`) unchanged.

## BI-2: Transfer Session Start API (implemented)
- Add endpoint to create/send transfer intent (`POST /api/v1/transfers`).
- Wire FE confirm-send to backend request and show request IDs in transfer queue.

## BI-3: Incoming Request + Consent API (implemented)
- Add endpoint/event feed for incoming transfer requests.
- Wire incoming modal to backend accept/decline APIs.

## BI-4: Live Transfer Progress Streaming (implemented)
- Add server-sent events/websocket for transfer progress updates.
- Replace local timer progress simulation with backend-driven updates.

## BI-5: Security + Settings Persistence (implemented)
- Wire trust actions, network mode, and accessibility/security settings to backend persistence APIs.

---

## Status
- ✅ BI-1 implemented
- ✅ BI-2 implemented
- ✅ BI-3 implemented
- ✅ BI-4 implemented
- ✅ BI-5 implemented
- 🎯 Frontend↔backend module plan complete
