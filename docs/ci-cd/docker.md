# Docker Integration

## Overview

Run PMAT in Docker containers for CI/CD pipelines.

## Example Dockerfile

```dockerfile
FROM rust:latest
RUN cargo install pmat
WORKDIR /app
CMD ["pmat", "quality-gate"]
```

## Related Documentation

- [GitHub Actions](./github-actions.md)
- [CI/CD Integration](./pre-commit.md)
