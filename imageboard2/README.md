



# first audit of version 2 
ChatGpt 4.1 (advanced version) audited imageboard2 and says "Your Rust+Actix-web code for a minimalist imageboard is extremely well-structured, safe, and nearly production-ready. Your implementation is exemplary for a clean, safe, and modern Actix/Askama SQLite board.
It’s more robust than many open-source boards out there!".   

This version is still in early dev. I will not make any changes to version2. I'm not saying it is perfect, but the app is a solid and reliable "Starter app"- in other words you can start from the code and edit it how you want. Have Ai audit / change the code however you want. Bam. 

Php is just an outdated mind virus- it is just silly in comparison to Rust. Golang web apps are superior to php- but Rust is objectively best of all of them. 



# Chessboard 🏁 – A tiny Actix Web imageboard

Chessboard is a minimal message‑board written in Rust/Actix‑web and SQLite.
Out of the box it supports:

* Thread creation with an optional image / MP4 attachment (50 MiB max)
* Replies that automatically *bump* the thread
* Pagination (10 / 25 / 50 posts per page)
* Inline image and video previews
* Template rendering via **Askama**
* Safe file‑type detection with **infer**
* Database connection‑pool (r2d2‑sqlite) in WAL mode
* Basic security headers, MIME‑sniffing, and upload size guard
* Stateless configuration via environment variables (`CHESSBOARD__…`)
* Dev‑friendly “wipe & rebuild” mode (`RESET_ON_START=1`)

---

## Quick start (development)

```bash
# 1. Install stable Rust
curl https://sh.rustup.rs -sSf | sh -s -- -y

# 2. Clone and build
git clone https://github.com/your‑org/chessboard.git
cd chessboard
cargo build            # or `cargo run`

# 3. Browse to http://localhost:8080
```

During development the application **deletes the database and uploads folder on each start**.
To keep data between restarts, run:

```bash
export CHESSBOARD__RESET_ON_START=0
cargo run
```

---

## Configuration

| ENV key                      | Default                   | Description                          |
| ---------------------------- | ------------------------- | ------------------------------------ |
| `CHESSBOARD__BIND`           | `0.0.0.0:8080`            | Listen address                       |
| `CHESSBOARD__DB_PATH`        | `db.sqlite`               | SQLite file                          |
| `CHESSBOARD__UPLOADS_DIR`    | `uploads`                 | Where attachments are stored         |
| `CHESSBOARD__TITLE`          | `Chessboard Messageboard` | Site header                          |
| `CHESSBOARD__RESET_ON_START` | `1` (true)                | Drop DB & uploads at boot (dev only) |

You can also drop a `Config.toml` next to the binary; any matching keys override the defaults.

Example:

```toml
# Config.toml
bind           = "127.0.0.1:9000"
reset_on_start = false
title          = "Prod Board"
```

---

## Production deployment

1. **Build** a release binary

   ```bash
   cargo build --release
   ```

2. **Create** a dedicated Unix user (e.g. `aileda`) and copy
   `target/release/chessboard` plus the `templates/` directory.

3. **Systemd unit** (sample):

   ```ini
   [Unit]
   Description=Chessboard BBS
   After=network.target

   [Service]
   User=aileda
   WorkingDirectory=/home/aileda/chessboard
   ExecStart=/home/aileda/chessboard/chessboard
   Environment=RUST_LOG=info
   Restart=on-failure
   AmbientCapabilities=CAP_NET_BIND_SERVICE

   [Install]
   WantedBy=multi-user.target
   ```

4. **Reverse‑proxy** behind Nginx:

   ```nginx
   server {
       listen 80;
       server_name example.com;
       location / { proxy_pass http://127.0.0.1:8080; }
       client_max_body_size 50M;
   }
   ```

   Add TLS with Certbot:

   ```bash
   sudo certbot --nginx -d example.com -d www.example.com
   ```

---

## Folder layout

```
chessboard
├── src/
│   ├── main.rs           ← application entry
│   └── templates.rs      ← Askama context structs
├── templates/
│   ├── index.html
│   ├── post_form.html
│   └── thread.html
├── Cargo.toml
└── README.md
```

SQLite file (`db.sqlite`) and uploaded files live in the path given by
`CHESSBOARD__DB_PATH` and `CHESSBOARD__UPLOADS_DIR`, respectively.

---

## Security notes

* Attachments are accepted **only** if the extension *and* MIME probe match an
  allow‑list (`jpg jpeg png gif webp mp4`).
* HTTP responses include `X‑Content‑Type‑Options: nosniff`, `X‑Frame‑Options:
  DENY`, and `Referrer‑Policy: same‑origin`.
* Files are served via Actix’s static handler; `Uploads` can be remounted on a
  separate volume or behind a CDN.
* SQLite runs in **WAL mode** for non‑blocking reads/writes; foreign‑keys are
  enforced.

---

## Contributing

Pull requests are welcome! Run the full check suite locally with:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

---

## License

MIT © 2025 Your Name / Your Org 




# //////////////////////////////

# If you have a vps or server  here is an automated script that was made for a fresh install of ubuntu 24-04 




# Setup Script for anysite.org Chessboard

This `setup.sh` script automates the provisioning of a server for the Chessboard application on domain `anysite.org`.

```bash
#!/usr/bin/env bash
set -euo pipefail

# ─── Config ────────────────────────────────────────────────────────────────────
DOMAIN="ANYSITE.org"
EMAIL="ANYMAIL@gmail.com"
APP_USER="ANYUSER"
APP_HOME="/home/${APP_USER}"
PROJECT_DIR="${APP_HOME}/chessboard"

# ─── Pre‑flight checks ────────────────────────────────────────────────────────
if [[ $EUID -ne 0 ]]; then
  echo "This script must be run as root (e.g. with sudo)." >&2
  exit 1
fi

id "${APP_USER}" &>/dev/null || {
  echo "Creating application user '${APP_USER}'..."
  useradd --create-home --shell /bin/bash "${APP_USER}"
}

# ─── System update ────────────────────────────────────────────────────────────
echo "==> Updating system packages…"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get upgrade -yqq
apt-get autoremove -yqq

# ─── Dependencies ─────────────────────────────────────────────────────────────
echo "==> Installing dependencies…"
apt-get install -yqq --no-install-recommends \
  nginx certbot python3-certbot-nginx \
  curl build-essential pkg-config libssl-dev \
  sqlite3 libsqlite3-dev

# ─── Rust toolchain ───────────────────────────────────────────────────────────
if ! sudo -u "${APP_USER}" -H bash -c 'command -v rustup &>/dev/null'; then
  echo "==> Installing Rust toolchain for ${APP_USER}…"
  sudo -u "${APP_USER}" -H bash -c 'curl -sSf https://sh.rustup.rs | sh -s -- -y'
else
  echo "==> Rust already installed for ${APP_USER} – skipping."
fi

RUST_ENV_SNIPPET='source "${HOME}/.cargo/env"'
grep -qF "${RUST_ENV_SNIPPET}" "${APP_HOME}/.bashrc" \
  || echo "${RUST_ENV_SNIPPET}" >> "${APP_HOME}/.bashrc"

# ─── Nginx ────────────────────────────────────────────────────────────────────
echo "==> Ensuring Nginx is running and enabled…"
systemctl enable --now nginx

# ─── Project & uploads directory ──────────────────────────────────────────────
echo "==> Creating/owning project directories…"
mkdir -p "${PROJECT_DIR}/uploads"
chown -R "${APP_USER}:${APP_USER}" "${PROJECT_DIR}"
chmod 755 "${PROJECT_DIR}"

# ─── Nginx server block ───────────────────────────────────────────────────────
NGINX_SITE="/etc/nginx/sites-available/${DOMAIN}"
if [[ ! -f "${NGINX_SITE}" ]]; then
cat > "${NGINX_SITE}" <<EOF
server {
    listen 80;
    server_name ${DOMAIN} www.${DOMAIN};

    client_max_body_size 50M;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
    }
}
EOF
  ln -sf "${NGINX_SITE}" "/etc/nginx/sites-enabled/${DOMAIN}"
  rm -f /etc/nginx/sites-enabled/default
  nginx -t && systemctl reload nginx
else
  echo "==> Nginx server block already present – skipping."
fi

# ─── SSL / Let’s Encrypt ──────────────────────────────────────────────────────
echo "==> Checking/obtaining Let’s Encrypt certificate…"
if [[ ! -e "/etc/letsencrypt/live/${DOMAIN}/fullchain.pem" ]]; then
  certbot --non-interactive --agree-tos --email "${EMAIL}" \
          --nginx -d "${DOMAIN}" -d "www.${DOMAIN}"
else
  echo "    – Certificate already exists. Triggering quiet renew if necessary."
  certbot renew --quiet --deploy-hook "systemctl reload nginx"
fi

# ─── Rust workspace ───────────────────────────────────────────────────────────
echo "==> Creating Rust workspace (if missing)…"
sudo -u "${APP_USER}" -H bash -c "
  source ~/.cargo/env
  cd '${APP_HOME}'
  if [[ ! -d chessboard/Cargo.toml && ! -f chessboard/Cargo.toml ]]; then
    cargo new chessboard
  else
    echo '    – Workspace already exists – skipping cargo new.'
  fi
"

# ─── Finished ────────────────────────────────────────────────────────────────
cat <<ENDMSG

✅  Setup (or maintenance) complete!
    – System packages are up‑to‑date.
    – Rust toolchain ensured for ${APP_USER}.
    – SQLite3 (client & dev headers) installed.
    – Nginx configured and reloaded; HTTP→Rust reverse proxy ready.
    – HTTPS active at https://${DOMAIN}/ (renewed automatically).
    – Rust workspace: ${PROJECT_DIR}
    – Static uploads directory: ${PROJECT_DIR}/uploads
ENDMSG
```





















