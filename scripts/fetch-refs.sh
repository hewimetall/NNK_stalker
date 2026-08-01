#!/usr/bin/env bash
# Shallow-clone architecture reference repos for offline reading.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${ROOT}/vendor/refs"
mkdir -p "${DEST}"

clone() {
  local url="$1" name="$2"
  if [[ -d "${DEST}/${name}/.git" ]]; then
    echo "skip ${name} (exists)"
    return
  fi
  echo "fetch ${name}"
  git clone --depth 1 "${url}" "${DEST}/${name}"
}

clone https://github.com/jbuehler23/eryndor-mmo.git eryndor-mmo
clone https://github.com/cBournhonesque/lightyear.git lightyear
clone https://github.com/ManevilleF/hexx.git hexx
clone https://github.com/CoreyCole/creative-mode.git creative-mode

SKILLS="${ROOT}/vendor/skills"
mkdir -p "${SKILLS}"
DEST="${SKILLS}"
clone https://github.com/gamedev-skills/awesome-gamedev-agent-skills.git awesome-gamedev
clone https://github.com/abagames/agentic-gamedev-skills.git agentic-gamedev

echo "done → vendor/refs + vendor/skills"
