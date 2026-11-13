# Container packaging (#63): smoke verification

Development verification of the Docker/Compose packaging on 2026-11-12.
The [machine-readable receipt](2026-11-12-container-verification.json) retains
exact commands, outputs, image/source identities and failed evidence.
Base: `b4226bb35d796222c30f93235ac22118dd26a344`. No Rust source, crate or
lockfile changes; conversion and workspace behavior are unmodified.

## Environment

macOS arm64 host, Docker Engine 29.5.2 via colima (Virtualization.Framework,
virtiofs), Compose 5.7.1. The VM was resized from 2 CPUs / 2 GiB to
6 CPUs / 8 GiB after the first build attempt OOMed. Installed Google Chrome
was driven headless through Playwright 1.62.1 for the browser unlock check.

## Images

- `forge:local` (linux/arm64, native build): multi-stage
  `rust:1.93.0-slim-bookworm` → `debian:bookworm-slim`, both digest-pinned;
  `cargo build --release --locked --bin forge`; runtime uid/gid 10001,
  185 MB, revision label bound to the base commit.
- `forge:local-amd64` (linux/amd64, QEMU-emulated build on this host):
  same Dockerfile and lockfile; `--version`, non-root identity and POL-01
  conversion all pass under emulation.

## Results

| Check | Result |
| --- | --- |
| `forge --version` in container | `forge 1.1.0`, exit 0 |
| Runtime identity | `uid=10001(forge) gid=10001(forge)` |
| Convert POL-01 → OSCAL Catalog JSON (non-root, read-only input mount) | exit 0; 1 group, 7 controls, OSCAL 1.2.3 |
| Embedded-schema `validate` of the artifact | exit 0, all validation passes |
| Workspace launch under PTY, host networking | passphrase prompts answered on `/dev/tty`; `http://127.0.0.1:<port>` printed |
| Root route + content-addressed JS asset | 200 / 200 |
| Wrong `Host`, `X-Forwarded-For` probes | 401 / 401 (guards intact) |
| Browser unlock: wrong then correct passphrase | 401 `unlock-failed`, then 200 with capability |
| Explicit registration preview + commit | 200 / 202 succeeded; index persisted |
| Upload preview + confirm write | 200 / 202 succeeded; exact bytes on host mount |
| Unsafe path (`../escape.md`) | 400 `invalid-request` |
| Symlink registration | 403 `resource-containment` |
| Read-only session mutation | 403 `read-only-session` |
| Restart persistence | project files + index persist; old capability 401; new capability differs |
| Machine session | headless descriptor; API works; **cannot unlock browser** (401) |
| No-TTY browser launch | exit 1 with the documented controlling-terminal message |
| API v2 machine session | descriptor `api_major: 2`, summary 200 |
| Real Chrome unlock | wrong passphrase refused; correct unlocks; overview renders; 0 non-loopback requests; 0 page errors |
| LAN isolation | loopback probe 200; same listener via host LAN IP unreachable |
| Compose config validation | `docker compose config --quiet` exit 0 |
| Compose no-TTY launch | documented controlling-terminal refusal |
| Compose PTY launch (`compose run workspace`) | passphrase prompts answered; root route 200 on printed loopback URL |

## Transport design

Host networking with no port publishing: the listener's loopback-peer
acceptance, exact `Host` pin, forwarded-header rejection and Origin checks
all remain intact (bridge + `-p` would fail the peer and Host guards).
Passphrase stays controlling-terminal-only; no env/argument/file injection
path exists or was added. See [docs/container.md](../container.md).

## Limitations

arm64 evidence is native; amd64 was built under QEMU emulation. Browser
verification is development evidence, not the maintained hosted-Chrome
campaign or release acceptance. No SBOM tool was available; identity is
bound via revision label, digest-pinned bases and the Cargo.lock hash.