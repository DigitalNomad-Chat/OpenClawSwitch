# Updating Claw

## Check Current Version

```bash
claw --version
```

## Update Methods

### Go Install (recommended for Go users)

```bash
go install github.com/vaayne/claw@latest
```

### From Source

```bash
cd ~/path/to/claw
git pull origin main
go build -o claw .
# Move binary to your PATH
```

### GitHub Releases

Download the latest binary from https://github.com/vaayne/claw/releases

Binaries available for: linux/darwin/windows x amd64/arm64.

### Docker

```bash
docker pull ghcr.io/vaayne/claw:latest
```

Tags: `latest` (stable), `vX.Y.Z` (specific release).

## After Updating

- Config format is backward-compatible; no migration needed
- Run `claw models update` to refresh the model cache if new models are available
- Builtin skills update automatically with the binary
