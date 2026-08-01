#!/usr/bin/env bash
# Smoke: register 5 users, fill lobby 5/5, ready-all, GM starts.
set -euo pipefail
BASE="${1:-http://127.0.0.1:8080}"
SUFFIX="${RANDOM}"

auth() {
  local user="$1" pass="$2"
  curl -fsS -X POST "$BASE/api/auth/register" \
    -H 'content-type: application/json' \
    -d "{\"username\":\"$user\",\"password\":\"$pass\"}"
}

login() {
  local user="$1" pass="$2"
  curl -fsS -X POST "$BASE/api/auth/login" \
    -H 'content-type: application/json' \
    -d "{\"username\":\"$user\",\"password\":\"$pass\"}"
}

echo "== health =="
curl -fsS "$BASE/health"; echo

GM_JSON=$(auth "gm_${SUFFIX}" "zonezone1")
GM_TOKEN=$(echo "$GM_JSON" | python3 -c 'import sys,json; print(json.load(sys.stdin)["access_token"])')

ROOM=$(curl -fsS -X POST "$BASE/api/rooms" -H "Authorization: Bearer $GM_TOKEN" -H 'content-type: application/json' -d '{}')
CODE=$(echo "$ROOM" | python3 -c 'import sys,json; print(json.load(sys.stdin)["code"])')
echo "room=$CODE"

# GM already member; join to get lobby view
curl -fsS -X POST "$BASE/api/rooms/$CODE/join" -H "Authorization: Bearer $GM_TOKEN" -d '{}' >/dev/null

for i in 1 2 3 4; do
  PJ=$(auth "p${i}_${SUFFIX}" "zonezone1")
  PT=$(echo "$PJ" | python3 -c 'import sys,json; print(json.load(sys.stdin)["access_token"])')
  curl -fsS -X POST "$BASE/api/rooms/$CODE/join" -H "Authorization: Bearer $PT" -d '{}' >/dev/null
  curl -fsS -X POST "$BASE/api/rooms/$CODE/ready" -H "Authorization: Bearer $PT" \
    -H 'content-type: application/json' -d '{"ready":true}' >/dev/null
  echo "joined p$i"
done

# 6th must fail
SIX=$(auth "overflow_${SUFFIX}" "zonezone1")
ST=$(echo "$SIX" | python3 -c 'import sys,json; print(json.load(sys.stdin)["access_token"])')
CODE_HTTP=$(curl -s -o /tmp/nnk_overflow.json -w '%{http_code}' -X POST "$BASE/api/rooms/$CODE/join" \
  -H "Authorization: Bearer $ST" -d '{}')
echo "overflow http=$CODE_HTTP"
test "$CODE_HTTP" = "409"

curl -fsS -X POST "$BASE/api/rooms/$CODE/ready" -H "Authorization: Bearer $GM_TOKEN" \
  -H 'content-type: application/json' -d '{"ready":true}' >/dev/null

START=$(curl -fsS -X POST "$BASE/api/rooms/$CODE/start" -H "Authorization: Bearer $GM_TOKEN" -d '{}')
echo "$START" | python3 -c 'import sys,json; d=json.load(sys.stdin); assert d["phase"]=="playing" and d["player_count"]==5; print("OK playing 5/5")'
