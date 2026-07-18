# NNK Stalker — Web Architecture

Настольная кампания «Сталкер» → web-мультиплеер (друзья + ГМ).  
Стек: **Rust · Bevy (client) · Axum · SQLite · JWT must-auth**.

Дизайн: `form.md.md`, `term.md.md`. Skills: `docs/SKILLS.md`. Refs: `docs/REFERENCES.md`.

---

## SOLID / DRY workspace (no monolith)

| Crate | Layer | Responsibility |
|-------|-------|----------------|
| `nnk_domain` | Domain | Entities/VOs (`HexCoord`, `LocationId`, `CccDraw`, `RoomState`) |
| `nnk_rules` | Domain services | Pure engine `init` / `step` / `legal_actions` (TDD core) |
| `nnk_protocol` | Shared kernel | Wire DTOs + `ClientMsg`→`Action` mapping |
| `nnk_auth` | Infra capability | `PasswordService` + `TokenService` traits (Argon2/JWT) |
| `nnk_ports` | Ports | `UserRepository` / `RoomRepository` |
| `nnk_persistence` | Adapters | SQLite impl of ports |
| `nnk_app` | Application | Use-cases (`AuthService`, `RoomService`) |
| `nnk_server` | Delivery | Thin Axum HTTP/WS composition root |
| `nnk_client` | Delivery | Bevy presentation (+ Recraft assets) |

Dependency rule: **delivery → app → ports ← persistence**; **rules/domain have zero IO**.

```
Browser Bevy WASM          Axum (must-auth)
   │                          │
   │ WSS + REST               ▼
   └──────────────► nnk_app (use-cases)
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           nnk_rules  nnk_auth   nnk_ports
              │                     ▲
              ▼                     │
          nnk_domain         nnk_persistence (SQLite)
```

---

## Must-auth

- Public: `GET /health`, `POST /api/auth/register`, `POST /api/auth/login`
- Everything else: `Authorization: Bearer <jwt>`
- WS: `/ws/rooms/{code}?token=<jwt>` + prior REST join

---

## Engine contract (from skill `verifying-turn-based-games`)

Adapted for multiplayer+GM (skill’s 2p-only metrics not applied):

- `init(seed)` → `RoomState` with `rng_seed` + `history`
- `step(state, action)` → new state; **illegal → `Err`, no mutate**
- `legal_actions(state, user, role)`
- Deterministic `ChaCha8` from seed; dice/CCC only server-side via rules

---

## Assets (Recraft)

`assets/generated/{hex_sector_tile,stalker_token,zone_atmosphere}.png`

---

## Coverage gate

```bash
./scripts/coverage.sh   # ≥93% lines on lib crates (excl. client/server shells)
```

---

## Phases

| Phase | Scope |
|-------|--------|
| P0 | Modular crates + auth + rooms + rules TDD + Recraft assets |
| P1 | Client net (login/WS) |
| P2 | Full hex interaction + legal move highlights |
| P3 | Missions / events content pack |
| P4 | Optional lightyear/WebTransport if needed |
