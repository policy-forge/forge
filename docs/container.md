# Container packaging

FORGE ships as a multi-stage OCI image: a Rust 1.93.0 build stage and a
Debian bookworm-slim runtime containing only the `forge` binary, running as
the non-root user `forge` (uid/gid 10001). Schemas, workspace UI assets and
the OpenAPI contracts are embedded at compile time (`include_str!`), so the
runtime image needs no data files. Conversion and embedded validation remain
fully offline; the only network activity is the explicitly launched local
workspace listener.

This is local development/testing packaging. Image publication, remote
deployment and release acceptance are out of scope (PRD 049 W-2 deferral is
lifted only for local use by ticket #63).

## Build

From the repository root (base images are digest-pinned in the `Dockerfile`):

```bash
docker build --build-arg FORGE_REVISION="$(git rev-parse HEAD)" -t forge:local .
# second architecture (emulated on an arm64 host, native on amd64):
docker build --platform linux/amd64 --build-arg FORGE_REVISION="$(git rev-parse HEAD)" -t forge:local-amd64 .
```

`cargo build --release --locked` binds the binary to the committed
`Cargo.lock`. The `.dockerignore` is a whitelist: only `Cargo.toml`,
`Cargo.lock`, `.cargo/`, `src/`, `schemas/`, `benches/` (manifest target
discovery), `ui/workspace.{js,css}` and `docs/api/` enter the build context.
Record the image identity for any evidence:

```bash
docker image inspect forge:local --format '{{.Id}} {{index .RepoDigests 0}}'
```

## CLI usage

The entrypoint is `forge`; the working directory is `/project` (uid 10001).
Mount inputs read-only and collect output on stdout, or mount a writable
output directory owned by uid 10001:

```bash
docker run --rm forge:local --version

# Convert a sample policy to OSCAL Catalog JSON (stdout, no container writes):
docker run --rm -v "$PWD/example_data:/in:ro" forge:local \
  convert /in/POL-01_Acceptable_Use_Policy.md --strategy catalog --format json \
  > catalog.json

# Validate the artifact with the embedded schemas (offline):
docker run --rm -v "$PWD/catalog.json:/in/catalog.json:ro" forge:local \
  validate /in/catalog.json
```

On Docker Desktop/colima bind mounts, host directories may not be writable by
uid 10001; prefer stdout redirection or a named volume for outputs, or
`chown 10001:10001` the output directory on the host first.

## Workspace transport design (review)

The workspace contract (PRD 062, `src/workspace/http.rs`) is deliberately
strict, and naive port publishing breaks it:

- The listener binds `127.0.0.1` with an OS-assigned port and **accepts only
  loopback peers**. With bridge networking and `-p`, connections arrive from
  the bridge gateway address and are dropped before any HTTP handling.
- Every request must carry exactly `Host: 127.0.0.1:<internal-port>`. A
  published host port differs from the internal port, so the guard rejects
  the request; rewriting `Host` would defeat the DNS-rebinding protection.
- `Forwarded`, `Via` and `X-Forwarded-*` headers are rejected, and API
  requests from browsers must match the exact internal `Origin`. A reverse
  proxy in front of the listener cannot satisfy these guards by design.

**Decision: host networking, no port publishing.** `network_mode: host`
places the listener on the host's own loopback interface: the peer address
stays `127.0.0.1`, the printed `http://127.0.0.1:<port>` URL is directly
usable, and every guard is preserved unmodified. Nothing is published to the
LAN — the socket is bound to loopback only. Docker Desktop and colima forward
guest-loopback listeners to the host loopback (verified in the #63 smoke
receipts). On a native Linux host the listener is the host loopback directly.
No Forge behavior was changed for containerization.

The port is OS-assigned per launch (there is no fixed-port flag). Read the
`Local workspace: http://127.0.0.1:<port>` line printed at startup; that URL
is the stable entry point for the session's lifetime.

### Passphrase contract

The per-launch passphrase is read from the **controlling terminal**
(`/dev/tty`, no echo) and confirmed by a second prompt. There is no
command-line, environment-variable, project-file or image-layer passphrase
path, and none is added by this packaging. Therefore the workspace must be
launched **interactively** with a TTY:

- `docker run -it …` or `docker compose run --rm …` (Compose sets
  `stdin_open`/`tty`). `docker compose up` without a TTY, or any detached
  launch, fails with: *"A controlling terminal is required; automation must
  use --machine-session."*
- Automation uses `--machine-session`: a headless capability token printed
  once to stdout as a JSON descriptor. Machine credentials are scoped to the
  machine session and **cannot unlock the browser session** (the unlock
  route refuses non-browser mode). Never log or persist the descriptor
  capability beyond the session that needs it.

`--no-open` is required in containers: there is no `xdg-open` in the runtime
image. Open the printed URL in your host browser yourself.

## Docker Compose (local workspace)

`docker-compose.yml` defines two services sharing one project directory and
one private-state volume. Both use host networking; the host endpoint is
loopback-only and inaccessible from the LAN by default.

```bash
mkdir -p forge-project                     # isolated project mount
docker compose build                       # or: docker build -t forge:local .

# One-time: make the project mount owned by the runtime uid (bind mounts
# inherit host ownership; on Docker Desktop/colima they start root-owned).
docker run --rm --user root --entrypoint chown \
  -v "$PWD/forge-project:/project" forge:local -R 10001:10001 /project

docker compose run --rm workspace          # writable project
docker compose run --rm workspace-read-only  # read-only project
```

At the prompts, set a 15–128 character passphrase (typed twice, no echo),
then open the printed `http://127.0.0.1:<port>` URL and unlock in the browser
with the same passphrase. Stop with Ctrl-C.

### Volumes and persistence

| Mount | Mode | Contents |
|---|---|---|
| `./forge-project:/project` | rw (`workspace`) / ro (`workspace-read-only`) | The selected project root: registered resources, `forge.workspace.json` index, `.forge.toml`, confirmed writes. |
| `workspace-state:/home/forge/.local/state` | rw | Forge private restore state (`forge/source-restores`), uid-10001-owned with 0700 leaf directories. |

- Project files and confirmed outcomes persist across container restarts
  because they live in the bind mount. Session capabilities do **not**
  persist: every launch mints a new session and passphrase.
- The private-state volume must stay owned by uid 10001 with no group/other
  write bits on the ancestry; the workspace qualifies this at startup and
  refuses a foreign or world-writable state root. Do not point
  `XDG_STATE_HOME` at a shared directory.
- The named `workspace-state` volume inherits uid-10001 ownership from the
  image directory on first use; Docker populates empty named volumes from
  the image. If you recreate it manually, chown it to 10001:10001.
- Read-only mode refuses every mutation with `read-only-session`; the
  project mount is additionally `:ro` at the container boundary.
- Filesystem admission is unchanged in containers: explicit registration
  only, no discovery; unsafe paths, symlinks and hard links are denied by the
  same descriptor-anchored checks as native runs.

### Reset

```bash
docker compose down                        # stop services
docker compose down -v                     # also delete private state volume
rm -rf forge-project                       # delete project data (disposable)
```

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `A controlling terminal is required; automation must use --machine-session.` | Launched without a TTY. Use `docker compose run` / `docker run -it`, or `--machine-session` for automation. |
| `The browser could not be opened. Restart with --no-open.` | No browser launcher in the image. Always pass `--no-open` and open the printed URL manually. |
| Workspace exits with an unavailable/private-state error | State volume not owned by uid 10001 or ancestry group/other-writable. Recreate the volume (`docker compose down -v`) or `chown -R 10001:10001` it. |
| `permission denied` writing outputs to a bind mount | Host directory not writable by uid 10001. Use stdout, a named volume, or chown the directory. |
| Connection refused from another machine | Expected: the listener is loopback-only by design. There is no LAN mode. |

## Native vs container vs release acceptance

Native host execution remains the reference path (see the
[usage guide](usage-guide.md)). Container execution reuses the identical
binary and contracts; the smoke receipts for #63
(`docs/plans/2026-11-12-container-verification.json`) record build, CLI,
workspace and persistence evidence with exact source/image identities.
Container smoke evidence is development verification, not release
acceptance: human security/accessibility approval and release packaging
remain separate gates.