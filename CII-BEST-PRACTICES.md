# OpenSSF Best Practices (CII) Adherence

This document tracks the project's adherence to the [OpenSSF Best Practices Badge](https://best-practices.coreinfrastructure.org/) criteria.

## Summary
The anvomidav project is committed to following open-source security and quality best practices.

## Change Control
- **Public Repository**: All source code is hosted on GitHub and is public.
- **Version Control**: We use Git for version control.
- **Unique Versioning**: All releases use unique version identifiers (SemVer).

## Reporting
- **Bug Reporting Process**: Documented in `CONTRIBUTING.md`.
- **Vulnerability Reporting**: A clear `SECURITY.md` file defines the private reporting process.

## Quality
- **Automated Builds**: We use GitHub Actions for automated builds and CI.
- **Testing**: Automated test suites are integrated into the CI pipeline.
- **New Features**: New functionality is required to have associated tests.

## Security
- **Secure Development**: We use automated security scanners (CodeQL, Trufflehog, ClusterFuzzLite).
- **Dependency Pinning**: GitHub Actions and critical dependencies (including Fuzzing Dockerfiles) are pinned to specific versions/SHAs.
- **No Hardcoded Secrets**: Scanned via `trufflehog` and `gitleaks`.

## Best Practices
- **SPDX Headers**: We use SPDX license identifiers in all source files.
- **Code Review**: All changes require a pull request and code review before merging to `main`.
