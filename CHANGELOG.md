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
- `Maskable::project` now takes `&mut self` instead of `self`.
    - This enables more efficient implementation of `SelfMaskable` on `Box<T: SelfMaskable>`.
    - This enables more efficient implementation of `SelfMaskable` on large types.
    - `Mask::project`'s signature is unchanged.
- `Maskable::project` now takes a `ProjectOptions` argument.
    - Which can be used to normalize a message (e.g. converts `Some(Default::default())` to `None`).
- Deriving `OptionMaskable` on a `struct` now requires the `struct` to implement `PartialEq`.
    - This is required to implement normalization.
- `Mask::update` no longer takes a `UpdateOptions` argument.
    - Use `Mask::update_with_options` instead if you need to use options.

## Features
- `Mask` now implements
    - `empty()`
    - `project_with_options()`
    - `update_with_options()`
- Added crate-level documentation.

## Bug fixes
