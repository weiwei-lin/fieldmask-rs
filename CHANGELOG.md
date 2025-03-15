# Overview
This file is intended to capture unreleased changes.

# Released
For released changes, check the release notes [here](https://github.com/weiwei-lin/fieldmask-rs/releases).

# Unreleased
## Breaking changes
- `Maskable` now needs to implement `empty_mask()`.
  - Previously `Default::default()` was used to construct empty masks.
  - `Maskable::Mask` no longer needs to implement `Default`.
  - Using `empty_mask()` to construct empty masks reduces confusion around what a default mask mean.

## Features
- `Mask` now implements `empty()`.
- Added crate-level documentation.

## Bug fixes
