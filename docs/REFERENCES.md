# References (SERP + mcpwork)

Собраны 2026-07-18 через mcpwork (`serpapi`, `searchcode`) как архитектурные референсы для NNK Stalker web port.

## Primary (использовать паттерны)

1. **eryndor-mmo** — https://github.com/jbuehler23/eryndor-mmo  
   Bevy 0.17 workspace, server-authoritative, SQLite+sqlx, auth, WASM client (`bevy_replicon` / renet2).

2. **lightyear** — https://github.com/cBournhonesque/lightyear  
   Bevy networking, WebTransport/WASM, ConnectToken after auth. Book: https://cbournhonesque.github.io/lightyear/book/

3. **creative-mode / boardgame template** — https://github.com/CoreyCole/creative-mode  
   Turn-based Bevy WASM, pure `rules.rs`, Trunk. (У них нет must-auth game server — берём только layout клиента.)

4. **hexx** — https://github.com/ManevilleF/hexx  
   Hex coordinates, pathfinding, Bevy integration.

5. **Turn-based multiplayer in Rust (Bevy client)** — https://herluf-ba.github.io/making-a-turn-based-multiplayer-game-in-rust-01-whats-a-turn-based-game-anyway.html

## Secondary

- Extreme Bevy + Matchbox (P2P WASM): https://johanhelsing.studio/posts/extreme-bevy  
  Полезно для web net, но P2P не подходит под ГМ + честные кубики.
- Bevy WASM cheatbook: https://bevy-cheatbook.github.io/platforms/wasm.html
- bevygap / Edgegap deploy pattern: https://www.metabrew.com/article/bevygap-bevy-multiplayer-with-edgegap-and-lightyear
- Axum + SQLite session auth: https://medium.com/@mikecode/axum-session-authentication-login-register-logout-sqlite-rust-0de3287aa767

## Domain docs (этот репозиторий)

**Canonical rules:** [`docs/pilgrim/`](pilgrim/README.md) — НКИ «S.T.A.L.K.E.R: Пилигрим» v3.0  
(общие правила, карты событий, сюжетная книга). Всегда решать дизайн в сторону этих файлов.

- `form.md.md` / `term.md.md` — краткие выжимки; при конфликте побеждает `docs/pilgrim/`

## Optional local mirrors

```bash
./scripts/fetch-refs.sh   # shallow clones into vendor/refs/ (gitignored)
```
