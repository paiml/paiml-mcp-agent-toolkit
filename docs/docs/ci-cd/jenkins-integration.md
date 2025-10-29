# Jenkins Integration

## Overview

Integrate PMAT into Jenkins pipelines.

## Example

```groovy
stage('Quality Gates') {
    steps {
        sh 'cargo install pmat'
        sh 'pmat quality-gate'
    }
}
```

## Related Documentation

- [GitHub Actions](./github-actions-integration.md)
- [GitLab CI](./gitlab-ci-integration.md)
