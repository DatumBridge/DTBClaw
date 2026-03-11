# macOS Update and Uninstall Guide

This page documents supported update and uninstall procedures for OctoClaw on macOS (OS X).

Last verified: **February 22, 2026**.

## 1) Check current install method

```bash
which octoclaw
octoclaw --version
```

Typical locations:

- Homebrew: `/opt/homebrew/bin/octoclaw` (Apple Silicon) or `/usr/local/bin/octoclaw` (Intel)
- Cargo/bootstrap/manual: `~/.cargo/bin/octoclaw`

If both exist, your shell `PATH` order decides which one runs.

## 2) Update on macOS

Quick way to get install-method-specific guidance:

```bash
octoclaw update --instructions
octoclaw update --check
```

### A) Homebrew install

```bash
brew update
brew upgrade octoclaw
octoclaw --version
```

### B) Clone + bootstrap install

From your local repository checkout:

```bash
git pull --ff-only
./bootstrap.sh --prefer-prebuilt
octoclaw --version
```

If you want source-only update:

```bash
git pull --ff-only
cargo install --path . --force --locked
octoclaw --version
```

### C) Manual prebuilt binary install

Re-run your download/install flow with the latest release asset, then verify:

```bash
octoclaw --version
```

You can also use the built-in updater for manual/local installs:

```bash
octoclaw update
octoclaw --version
```

## 3) Uninstall on macOS

### A) Stop and remove background service first

This prevents the daemon from continuing to run after binary removal.

```bash
octoclaw service stop || true
octoclaw service uninstall || true
```

Service artifacts removed by `service uninstall`:

- `~/Library/LaunchAgents/com.octoclaw.daemon.plist`

### B) Remove the binary by install method

Homebrew:

```bash
brew uninstall octoclaw
```

Cargo/bootstrap/manual (`~/.cargo/bin/octoclaw`):

```bash
cargo uninstall octoclaw || true
rm -f ~/.cargo/bin/octoclaw
```

### C) Optional: remove local runtime data

Only run this if you want a full cleanup of config, auth profiles, logs, and workspace state.

```bash
rm -rf ~/.octoclaw
```

## 4) Verify uninstall completed

```bash
command -v octoclaw || echo "octoclaw binary not found"
pgrep -fl octoclaw || echo "No running octoclaw process"
```

If `pgrep` still finds a process, stop it manually and re-check:

```bash
pkill -f octoclaw
```

## Related docs

- [One-Click Bootstrap](../one-click-bootstrap.md)
- [Commands Reference](../commands-reference.md)
- [Troubleshooting](../troubleshooting.md)
