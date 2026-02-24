# Workflow fixes for CI and release

## Request

Investigate and fix failures in `.github/workflows/release.yml` and `.github/workflows/ci.yml` after package publishing changes.

## What was changed

1. `release.yml` version validation
   - Replaced `grep '^version' Cargo.toml` parsing with Python `tomllib`.
   - New check reads `workspace.package.version` from root `Cargo.toml`.
   - This matches the workspace-style manifest used in this repo.

2. `ci.yml` maturin invocation
   - Changed install command from `pip install maturin` to `python -m pip install maturin`.
   - Changed build command from `maturin build ...` to `python -m maturin build ...`.
   - This avoids PATH resolution issues on runners (especially Windows).

## Why this fixes the failures

- Root `Cargo.toml` does not have a top-level `version` key; the old release workflow could not validate version tags correctly.
- Calling `maturin` directly can fail when scripts path is not in PATH; `python -m maturin` is runner-agnostic.

## Next validation

- Re-run CI workflow on a branch/PR.
- Push a test tag like `py-v0.1.0` (or next release tag) to validate release workflow end-to-end.
