#!/usr/bin/env bash
# Deploy nnk_server behind nginx on HTTP (no SSL).
# Run ON the VPS as root, or: ssh root@HOST 'bash -s' < scripts/deploy-vps-http.sh
set -euo pipefail

DOMAIN="${NNK_DOMAIN:-game.1339259-cn56088.tw1.ru}"
REPO_URL="${NNK_REPO_URL:-https://github.com/hewimetall/NNK_stalker.git}"
REPO_REF="${NNK_REPO_REF:-cursor/architecture-web-scaffold-67cf}"
APP_DIR="${NNK_APP_DIR:-/opt/nnk}"

export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y git curl build-essential pkg-config libssl-dev nginx ufw ca-certificates

if ! command -v rustc >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env"

mkdir -p "$APP_DIR"
if [[ ! -d "$APP_DIR/.git" ]]; then
  git clone "$REPO_URL" "$APP_DIR"
fi
cd "$APP_DIR"
git fetch --all --prune
git checkout "$REPO_REF"
git pull --ff-only origin "$REPO_REF" || true

rustup target add wasm32-unknown-unknown
if ! command -v trunk >/dev/null 2>&1; then
  cargo install trunk
fi
(cd web && NO_COLOR=true trunk build --release)
cargo build -p nnk_server --release

if [[ ! -f "$APP_DIR/.jwt_secret" ]]; then
  openssl rand -hex 32 >"$APP_DIR/.jwt_secret"
  chmod 600 "$APP_DIR/.jwt_secret"
fi

cat >"$APP_DIR/nnk.env" <<EOF
NNK_BIND=127.0.0.1:8080
NNK_DATABASE_URL=sqlite:${APP_DIR}/nnk.db?mode=rwc
NNK_STATIC_DIR=${APP_DIR}/web/dist
NNK_JWT_SECRET=$(cat "$APP_DIR/.jwt_secret")
EOF
chmod 600 "$APP_DIR/nnk.env"

cat >/etc/systemd/system/nnk.service <<EOF
[Unit]
Description=NNK Stalker server
After=network.target

[Service]
Type=simple
WorkingDirectory=${APP_DIR}
EnvironmentFile=${APP_DIR}/nnk.env
ExecStart=${APP_DIR}/target/release/nnk_server
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

cat >/etc/nginx/sites-available/nnk <<EOF
server {
    listen 80 default_server;
    server_name ${DOMAIN} _;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_connect_timeout 3s;
        proxy_read_timeout 3600s;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF

ln -sf /etc/nginx/sites-available/nnk /etc/nginx/sites-enabled/nnk
rm -f /etc/nginx/sites-enabled/default

ufw allow OpenSSH || true
ufw allow 80/tcp || true
ufw --force enable || true

systemctl daemon-reload
systemctl enable --now nnk nginx
systemctl restart nnk
nginx -t
systemctl reload nginx

sleep 1
curl -fsS http://127.0.0.1:8080/health
echo
curl -fsS -H "Host: ${DOMAIN}" http://127.0.0.1/health
echo
echo "OK: http://${DOMAIN}/"
