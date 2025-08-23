# PMAT Docker Guide

This guide covers running PMAT v2.12.0 in containerized environments using Docker.

## 🐳 Quick Start

### Pull and Run
```bash
# Pull the latest image
docker pull paiml/pmat:2.12.0

# Run version check
docker run --rm paiml/pmat:2.12.0 pmat --version

# Analyze current directory
docker run --rm -v $(pwd):/workspace paiml/pmat:2.12.0 pmat context
```

### Claude Code Agent Mode
```bash
# Start MCP server for Claude Code integration
docker run -it --rm paiml/pmat:2.12.0 pmat agent mcp-server

# Background agent monitoring
docker run -d --name pmat-agent \
  -v $(pwd):/workspace:ro \
  -v pmat-data:/var/lib/pmat-agent \
  paiml/pmat:2.12.0 pmat agent start --project-path /workspace
```

## 🏗️ Docker Compose

Use the provided `docker-compose.yml` for multi-service setup:

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f pmat

# Stop all services
docker-compose down
```

### Services Included:
- **pmat**: Web demo on http://localhost:8080
- **pmat-agent**: Background quality monitoring
- **pmat-mcp**: MCP server for Claude Code integration

## 🔧 Configuration

### Environment Variables
```bash
# Logging level
-e RUST_LOG=info,pmat=debug

# Configuration file
-e PMAT_CONFIG=/etc/pmat/agent-production.toml

# Custom working directory
-e PMAT_WORKDIR=/workspace
```

### Volume Mounts
```bash
# Mount source code (read-only)
-v $(pwd):/workspace:ro

# Persistent state
-v pmat-state:/var/lib/pmat-agent

# Logs
-v pmat-logs:/var/log/pmat-agent

# Custom config
-v ./my-config.toml:/etc/pmat/custom.toml
```

## 🚀 Usage Examples

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Run PMAT Quality Gates
  run: |
    docker run --rm -v ${{ github.workspace }}:/workspace \
      paiml/pmat:2.12.0 pmat quality-gate --strict --format json
```

### Development Workflow
```bash
# Interactive development shell
docker run -it --rm -v $(pwd):/workspace paiml/pmat:2.12.0 bash

# Watch mode for continuous analysis
docker run --rm -v $(pwd):/workspace \
  paiml/pmat:2.12.0 pmat agent start --project-path /workspace
```

### Production Deployment
```bash
# Production agent with systemd-style restart
docker run -d --name pmat-production \
  --restart unless-stopped \
  -v /opt/projects:/workspace:ro \
  -v pmat-prod-state:/var/lib/pmat-agent \
  -v pmat-prod-logs:/var/log/pmat-agent \
  -p 9090:9090 \
  paiml/pmat:2.12.0 pmat agent start --project-path /workspace
```

## 🛡️ Security Features

### Non-Root User
- Container runs as user `pmat` (non-root)
- UID/GID: 1001:1001
- No privileged access required

### Security Hardening
- Minimal base image (debian:bookworm-slim)
- Only essential packages installed
- No unnecessary services running
- Read-only root filesystem compatible

### Network Security
```bash
# Run with limited network access
docker run --rm --network none \
  -v $(pwd):/workspace:ro \
  paiml/pmat:2.12.0 pmat context
```

## 📊 Multi-Architecture Support

PMAT Docker images support multiple architectures:

- **linux/amd64**: x86_64 Intel/AMD processors
- **linux/arm64**: ARM64 processors (Apple M1/M2, ARM servers)

Docker automatically selects the correct architecture for your platform.

## 🔍 Monitoring & Health Checks

### Health Check
```bash
# Built-in health check
docker run --rm --health-cmd="pmat --version" \
  --health-interval=30s \
  --health-timeout=10s \
  --health-retries=3 \
  paiml/pmat:2.12.0
```

### Metrics Endpoint
```bash
# Access Prometheus metrics
docker run -p 9090:9090 paiml/pmat:2.12.0 pmat agent start --enable-metrics
curl http://localhost:9090/metrics
```

## 🐛 Troubleshooting

### Debug Mode
```bash
# Enable debug logging
docker run --rm -e RUST_LOG=debug \
  -v $(pwd):/workspace \
  paiml/pmat:2.12.0 pmat context --verbose
```

### Container Shell Access
```bash
# Access container shell for debugging
docker run -it --entrypoint=/bin/bash paiml/pmat:2.12.0

# Inspect running container
docker exec -it pmat-agent bash
```

### Common Issues

1. **Permission Errors**: Ensure mounted volumes have correct permissions
   ```bash
   chmod -R 755 $(pwd)
   ```

2. **Memory Limits**: For large codebases, increase container memory
   ```bash
   docker run --memory=2g paiml/pmat:2.12.0 pmat context
   ```

3. **Network Issues**: Check port conflicts and firewall settings
   ```bash
   docker run -p 8081:8080 paiml/pmat:2.12.0 pmat demo --serve
   ```

## 📦 Image Variants

### Available Tags
- `paiml/pmat:latest` - Latest stable release
- `paiml/pmat:2.12.0` - Specific version
- `paiml/pmat:2.12` - Minor version latest
- `paiml/pmat:2` - Major version latest

### Image Size
- **Compressed**: ~150MB
- **Uncompressed**: ~400MB
- **Multi-stage build**: Optimized for production

## 🔗 Related Resources

- **Docker Hub**: https://hub.docker.com/r/paiml/pmat
- **GitHub Repository**: https://github.com/paiml/paiml-mcp-agent-toolkit
- **Claude Code Integration**: [CLAUDE_CODE_AGENT.md](CLAUDE_CODE_AGENT.md)

Built with ❤️ following Toyota Way principles by [Pragmatic AI Labs](https://paiml.com)