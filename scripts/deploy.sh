#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export NNK_JWT_SECRET="${NNK_JWT_SECRET:-nnk-zone-secret-change-me}"
export NNK_DATABASE_URL="${NNK_DATABASE_URL:-sqlite:${ROOT}/nnk.db?mode=rwc}"
export NNK_BIND="${NNK_BIND:-0.0.0.0:8080}"
export NNK_STATIC_DIR="${NNK_STATIC_DIR:-${ROOT}/web/dist}"

rustup target add wasm32-unknown-unknown
(cd "$ROOT/web" && NO_COLOR=true trunk build --release)
cargo build -p nnk_server --release

SESSION="nnk-server"
tmux -f /exec-daemon/tmux.portal.conf has-session -t "=$SESSION" 2>/dev/null && tmux -f /exec-daemon/tmux.portal.conf kill-session -t "$SESSION" || true
tmux -f /exec-daemon/tmux.portal.conf new-session -d -s "$SESSION" -c "$ROOT" -- "${SHELL:-bash}" -l
tmux -f /exec-daemon/tmux.portal.conf send-keys -t "$SESSION:0.0" \
  "export NNK_JWT_SECRET='$NNK_JWT_SECRET' NNK_DATABASE_URL='$NNK_DATABASE_URL' NNK_BIND='$NNK_BIND' NNK_STATIC_DIR='$NNK_STATIC_DIR'; ./target/release/nnk_server" C-m

echo "server starting in tmux session '$SESSION' on $NNK_BIND"
for i in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:${NNK_BIND##*:}/health" >/dev/null 2>&1; then
    echo "healthy: http://127.0.0.1:${NNK_BIND##*:}/"
    exit 0
  fi
  sleep 0.5
done
echo "server did not become healthy" >&2
exit 1
