# GitLab CI Integration

## Overview

Integrate PMAT into GitLab CI pipelines.

## Example

```yaml
quality-gates:
  stage: test
  script:
    - cargo install pmat
    - pmat quality-gate
```

## Related Documentation

- [GitHub Actions](./github-actions-integration.md)
- [Jenkins](./jenkins-integration.md)
