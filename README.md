# NNK Stalker

Настольная кампания в духе «Сталкер» → web-мультиплеер для игры с друзьями.

**Стек:** Rust · Bevy · Axum · SQLite · JWT (**must-auth**)  
**Архитектура:** SOLID/DRY multi-crate (не монолит) — см. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## Docs

- [Architecture](docs/ARCHITECTURE.md)
- [Skills (SERP)](docs/SKILLS.md)
- [References](docs/REFERENCES.md)
- Design: `form.md.md`, `term.md.md`

## Crates

| Crate | Role |
|-------|------|
| `nnk_domain` | Value objects / entities |
| `nnk_rules` | Pure `init`/`step`/`legal` engine |
| `nnk_protocol` | Wire messages |
| `nnk_auth` | Argon2 + JWT traits |
| `nnk_ports` | Repository ports |
| `nnk_persistence` | SQLite adapters |
| `nnk_app` | Use-cases |
| `nnk_server` | Thin Axum host |
| `nnk_client` | Bevy UI + Recraft assets |

## Quick start

```bash
export NNK_JWT_SECRET=change-me
export NNK_DATABASE_URL='sqlite:nnk.db?mode=rwc'

cargo test --workspace --exclude nnk_client
./scripts/coverage.sh          # TDD gate ≥93%

cargo run -p nnk_server
cargo run -p nnk_client        # hex shell + generated art
```

```bash
curl -s localhost:8080/health
curl -s -X POST localhost:8080/api/auth/register \
  -H 'content-type: application/json' \
  -d '{"username":"stalker1","password":"zonezone1"}'
```

## Assets

Recraft-generated under `assets/generated/` (hex tile, stalker token, zone atmosphere).

## Skills

Installed under `.agents/skills/` from:

- https://github.com/abagames/agentic-gamedev-skills
- https://github.com/gamedev-skills/awesome-gamedev-agent-skills
