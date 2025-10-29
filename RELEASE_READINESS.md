# PMAT Release Readiness Checklist

## Pre-Release Checklist

- [ ] All tests passing (`make test`)
- [ ] Test coverage ≥ 85% (`make coverage`)
- [ ] No clippy warnings (`make lint`)
- [ ] Book validation passing (`make validate-book`)
- [ ] Documentation accuracy validated (`pmat validate-docs`)
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml

## Release Process

1. Ensure all quality gates pass
2. Update version number
3. Update CHANGELOG.md
4. Push changes to master
5. Tag release
6. Publish to crates.io (if applicable)

## Related Documentation

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [ROADMAP.md](./ROADMAP.md)
- [QUALITY_STATUS.md](./QUALITY_STATUS.md)
