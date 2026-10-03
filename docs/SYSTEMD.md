# systemd Services

This project can be run headless via systemd user services.

## Services included

- `council-of-dicks-web-server.service`
  - Runs the headless Rust server on port `8080`.
  - Provides REST + WebSocket endpoints (e.g. `/api/*`, `/ws/*`).

- `council-of-dicks-ui-dev.service`
  - Runs the Vite dev server on port `5175`.
  - Binds to `0.0.0.0` so it is reachable from LAN clients.

## Prerequisites

- Rust toolchain + a built release binary for the web server:
  - `cd src-tauri && cargo build --release --bin council-web-server`
- Node.js + `pnpm`
  - `pnpm install`

## Install (user services)

```bash
mkdir -p ~/.config/systemd/user
cp scripts/systemd/council-of-dicks-*.service ~/.config/systemd/user/

systemctl --user daemon-reload
systemctl --user enable --now council-of-dicks-web-server.service
systemctl --user enable --now council-of-dicks-ui-dev.service
```

## Status and logs

```bash
systemctl --user status council-of-dicks-web-server.service
systemctl --user status council-of-dicks-ui-dev.service

journalctl --user -u council-of-dicks-web-server.service -f
journalctl --user -u council-of-dicks-ui-dev.service -f
```

## Verify

From the host:

```bash
curl -sS http://127.0.0.1:8080/health
curl -sS -I http://127.0.0.1:5175/
```

From another machine on the LAN:

- `http://<this-host-ip>:5175/`

## Common failure modes

- **Browser shows “Kan geen verbinding maken”**
  - The UI service is not running, or port `5175` is blocked by a firewall.
  - Confirm `ss -ltnp | grep :5175` on the host.

- **UI service fails with `pnpm: No such file or directory`**
  - systemd user services may run with a minimal PATH.
  - Update `ExecStart` in `~/.config/systemd/user/council-of-dicks-ui-dev.service` to your absolute pnpm path (example used in this repo: `/home/flip/.local/share/pnpm/pnpm`).

- **Wrong IP**
  - Use the IP of the machine running the UI service (not the Ollama host unless they are the same box).
