# GitHub Pages Configuration

GitHub Pages has been successfully enabled for this repository.

## Current Configuration:

- **Source**: GitHub Actions (workflow)
- **Branch**: master
- **Status**: ✅ Active
- **URL**: https://paiml.github.io/paiml-mcp-agent-toolkit/

## Documentation Deployment:

The documentation is automatically deployed via the `.github/workflows/docs.yml` workflow which:
1. Builds Rust documentation with `cargo doc`
2. Includes the `rust-docs` directory
3. Deploys to GitHub Pages

The documentation is updated automatically on every push to the master branch.

## Manual Deployment:

To manually trigger a documentation deployment:
```bash
gh workflow run docs.yml
```

## Viewing the Documentation:

The documentation is available at:
https://paiml.github.io/paiml-mcp-agent-toolkit/