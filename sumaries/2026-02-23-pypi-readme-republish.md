# PyPI README update + republish attempt

## What changed

- Rewrote `crates/syspulse-py/README.md` to be Python-package specific.
- Removed Rust CLI/core-focused content and replaced with:
  - `pip install` instructions
  - supported Python versions
  - quick Python import example
  - package-specific notes and project links

## Publish attempt

- Ran publish from `.venv312`:
  - `python -m maturin publish --manifest-path crates/syspulse-py/Cargo.toml`
- Build succeeded.
- Upload failed because PyPI already has `syspulse==0.1.0` artifacts.

## Blocking reason

PyPI does not allow re-uploading the same filename/version.
Need a new version (for example `0.1.1`) before publish will succeed.
