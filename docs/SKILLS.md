# Agent skills for NNK Stalker

Найдены через SERP (mcpwork/serpapi) и установлены локально в `.agents/skills/`.

## Installed (project)

| Skill | Source | Why |
|-------|--------|-----|
| `verifying-turn-based-games` | [agentic-gamedev-skills](https://github.com/abagames/agentic-gamedev-skills) | Pure-function engine contract (`init`/`step`/`legal`) — adapted for multiplayer+GM |
| `implementing-gameplay-invariants` | same | Invariants for rules tests |
| `designing-minimal-game-rules` | same | Keep MVP rules thin |
| `probing-web-game-mechanics` | same | Web playtest probes |
| `smoke-testing-web-games` | same | WASM smoke checks |
| `bevy-ecs` | [awesome-gamedev-agent-skills](https://github.com/gamedev-skills/awesome-gamedev-agent-skills) | Bevy client ECS structure |
| `save-systems` | same | Persist room/campaign snapshots |
| `game-ui-ux` | same | Hex HUD / turn UX |

## Catalogs (vendor mirrors)

- `vendor/skills/awesome-gamedev` — 66 gamedev skills + router
- `vendor/skills/agentic-gamedev` — turn-based verification toolkit

## Also noted (not vendored)

- Bevy ECS Expert: https://github.com/sickn33/antigravity-awesome-skills/blob/main/skills/bevy-ecs-expert/SKILL.md
- Multiplayer skill template: https://github.com/davila7/claude-code-templates/.../multiplayer/SKILL.md
- Web game foundations (Phaser-oriented): openai/plugins game-studio

Refresh mirrors: `./scripts/fetch-refs.sh` and re-clone skill repos under `vendor/skills/`.
