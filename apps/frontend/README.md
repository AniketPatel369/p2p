# Frontend (Module FE-1)

This is a static front-end starter so you can test locally right now.

## Run locally (frontend + backend discovery integration)

From repository root, start backend service in terminal-1:

```bash
cargo run -p backend_service
```

Then start frontend static server in terminal-2:

```bash
cd apps/frontend
python3 -m http.server 4173
```

If Python is not installed, you can use Node instead:

```bash
cd apps/frontend
npx http-server -p 4173
```

Then open:

- http://localhost:4173

## Current status

- ✅ FE-1 App Shell + AirDrop-like layout implemented.
- ✅ FE-2 Device Discovery UI binding implemented (loading/ready/empty/error + dynamic cards).
- ✅ FE-3 Drag & Drop send flow implemented (file picker, drag-drop, multi-receiver selection, send readiness check).
- ✅ FE-4 Incoming request modal implemented (accept/decline).
- ✅ FE-5 Transfer dashboard implemented (progress + pause/resume/cancel + completed/failed states).
- ✅ FE-6 Security/Trust UI implemented (fingerprint + trust/revoke state).
- ✅ FE-7 Settings/Network controls implemented (LAN-only/relay/diagnostics + update channel).
- ✅ FE-8 Accessibility/polish implemented (skip link, focus-visible, reduced motion, high-contrast, large text, responsive polish).
- ✅ BI-1 Frontend-backend discovery API integration implemented (`backend_service` + `/api/v1/discovery/devices`).

- ✅ BI-2 Transfer session start integration implemented (`POST /api/v1/transfers` from FE send action).

- ✅ BI-3 Incoming request consent integration implemented (`GET /api/v1/incoming-request` + `POST /api/v1/incoming-request/decision`).

- ✅ BI-4 Transfer progress streaming integration implemented (`GET /api/v1/transfers/progress` polling from FE transfer dashboard).

- ✅ BI-5 Security/settings persistence integration implemented (`/api/v1/security/state`, `/api/v1/security/trust`, `/api/v1/settings`).
