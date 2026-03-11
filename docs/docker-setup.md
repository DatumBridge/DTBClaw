# Docker Setup Guide

This guide explains how to run OctoClaw in Docker mode, including bootstrap, onboarding, and daily usage.

## Prerequisites

- [Docker](https://docs.docker.com/engine/install/) or [Podman](https://podman.io/getting-started/installation)
- Git

## Quick Start

### 1. Bootstrap in Docker Mode

```bash
# Clone the repository
git clone https://github.com/octoclaw-labs/octoclaw.git
cd octoclaw

# Run bootstrap with Docker mode
./bootstrap.sh --docker
```

This builds the Docker image and prepares the data directory. Onboarding is **not** run by default in Docker mode.

### 2. Run Onboarding

After bootstrap completes, run onboarding inside Docker:

```bash
# Interactive onboarding (recommended for first-time setup)
./octoclaw_install.sh --docker --interactive-onboard

# Or non-interactive with API key
./octoclaw_install.sh --docker --api-key "sk-..." --provider openrouter
```

### 3. Start OctoClaw

#### Daemon Mode (Background Service)

```bash
# Start as a background daemon
./octoclaw_install.sh --docker --docker-daemon

# Check logs
docker logs -f octoclaw-daemon

# Stop the daemon
docker rm -f octoclaw-daemon
```

#### Interactive Mode

```bash
# Run a one-off command inside the container
docker run --rm -it \
  -v ~/.octoclaw-docker/.octoclaw:/home/claw/.octoclaw \
  -v ~/.octoclaw-docker/workspace:/workspace \
  octoclaw-bootstrap:local \
  octoclaw agent -m "Hello, OctoClaw!"

# Start interactive CLI mode
docker run --rm -it \
  -v ~/.octoclaw-docker/.octoclaw:/home/claw/.octoclaw \
  -v ~/.octoclaw-docker/workspace:/workspace \
  octoclaw-bootstrap:local \
  octoclaw agent
```

## Configuration

### Data Directory

By default, Docker mode stores data in:
- `~/.octoclaw-docker/.octoclaw/` - Configuration files
- `~/.octoclaw-docker/workspace/` - Workspace files

Override with environment variable:
```bash
OCTOCLAW_DOCKER_DATA_DIR=/custom/path ./bootstrap.sh --docker
```

### Pre-seeding Configuration

If you have an existing `config.toml`, you can seed it during bootstrap:

```bash
./bootstrap.sh --docker --docker-config ./my-config.toml
```

### Using Podman

```bash
OCTOCLAW_CONTAINER_CLI=podman ./bootstrap.sh --docker
```

## Common Commands

| Task | Command |
|------|---------|
| Start daemon | `./octoclaw_install.sh --docker --docker-daemon` |
| View daemon logs | `docker logs -f octoclaw-daemon` |
| Stop daemon | `docker rm -f octoclaw-daemon` |
| Run one-off agent | `docker run --rm -it ... octoclaw agent -m "message"` |
| Interactive CLI | `docker run --rm -it ... octoclaw agent` |
| Check status | `docker run --rm -it ... octoclaw status` |
| Start channels | `docker run --rm -it ... octoclaw channel start` |

Replace `...` with the volume mounts shown in [Interactive Mode](#interactive-mode).

## Reset Docker Environment

To completely reset your Docker OctoClaw environment:

```bash
./bootstrap.sh --docker --docker-reset
```

This removes:
- Docker containers
- Docker networks
- Docker volumes
- Data directory (`~/.octoclaw-docker/`)

## Troubleshooting

### "octoclaw: command not found"

This error occurs when trying to run `octoclaw` directly on the host. In Docker mode, you must run commands inside the container:

```bash
# Wrong (on host)
octoclaw agent

# Correct (inside container)
docker run --rm -it \
  -v ~/.octoclaw-docker/.octoclaw:/home/claw/.octoclaw \
  -v ~/.octoclaw-docker/workspace:/workspace \
  octoclaw-bootstrap:local \
  octoclaw agent
```

### No Containers Running After Bootstrap

Running `./bootstrap.sh --docker` only builds the image and prepares the data directory. It does **not** start a container. To start OctoClaw:

1. Run onboarding: `./octoclaw_install.sh --docker --interactive-onboard`
2. Start daemon: `./octoclaw_install.sh --docker --docker-daemon`

### Container Fails to Start

Check Docker logs for errors:
```bash
docker logs octoclaw-daemon
```

Common issues:
- Missing API key: Run onboarding with `--api-key` or edit `config.toml`
- Permission issues: Ensure Docker has access to the data directory

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OCTOCLAW_DOCKER_DATA_DIR` | Data directory path | `~/.octoclaw-docker` |
| `OCTOCLAW_DOCKER_IMAGE` | Docker image name | `octoclaw-bootstrap:local` |
| `OCTOCLAW_CONTAINER_CLI` | Container CLI (docker/podman) | `docker` |
| `OCTOCLAW_DOCKER_DAEMON_NAME` | Daemon container name | `octoclaw-daemon` |
| `OCTOCLAW_DOCKER_CARGO_FEATURES` | Build features | (empty) |

## Related Documentation

- [Quick Start](../README.md#quick-start)
- [Configuration Reference](config-reference.md)
- [Operations Runbook](operations-runbook.md)
