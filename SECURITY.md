# Security Policy

## Scope

`rust-stl` gives users a safe public API, but the implementation of `Vector<T>`
is built on raw pointers, manual allocation and `unsafe` blocks. The threat
model is therefore narrow but real:

* **Undefined behaviour reachable through the safe public API** — for example a
  double drop, a use-after-free, an out-of-bounds read or write, or a
  mis-matched allocation/deallocation layout.
* **Violation of an aliasing or initialization invariant** that a user could
  trigger without writing `unsafe` themselves.
* **Memory exhaustion or a denial of service** caused by an internal
  bookkeeping bug rather than by the caller explicitly requesting a huge
  allocation.

The following are *not* treated as security issues:

* Panics for documented reasons (`index out of bounds`, capacity overflow,
  `mid > len`, …). These are part of the documented contract.
* Memory leaks caused by a panicking user-supplied closure or destructor. This
  is safe — ownership is never duplicated — and is a consequence of running
  user code in the middle of an operation.
* `unsafe` code in *your* program that misuses the raw pointers returned by
  `begin`/`end`.
* Quadratic worst-case performance in the removal-by-shifting operations. It is
  documented as a known limitation, not a vulnerability.

## Supported versions

The crate has **not been published**. There are no released versions and no
maintenance branches; only the latest commit on `main` is supported.

## Reporting a vulnerability

Please report suspected undefined behaviour **privately**, using GitHub's
private vulnerability reporting for this repository:

<https://github.com/rohan3155/rust-stl/security/advisories/new>

Do not open a public issue for a memory-safety bug. If GitHub reporting is
unavailable to you, open a minimal public issue that says only that you have a
private security report and would like a contact channel — do not include the
reproduction in it.

A useful report includes:

* the crate revision (commit hash) you tested;
* the smallest program that triggers the problem, or the exact public API calls
  and sequence of operations;
* what you expected and what happened instead;
* the toolchain version (`rustc --version`);
* whether you observed it under Miri, a sanitizer, or a normal build, and the
  exact commands you ran.

## What to expect

This is a personal, unpaid project, so there is no service-level agreement and
no bug-bounty programme. Reports will be acknowledged on a best-effort basis,
investigated, and — if confirmed — fixed with a regression test and a
`CHANGELOG.md` entry that credits you unless you ask otherwise.

## Verification status

Be aware of the following before relying on this crate:

* The test suite includes exact destructor-count and panic-safety tests, but it
  is **not** exhaustive, and there is no fuzzing harness yet.
* **Miri is not run in CI.** The `unsafe` code is written to be Miri-clean and
  Miri is the recommended tool when changing it (see
  [CONTRIBUTING.md](CONTRIBUTING.md)), but the maintainers have not yet
  established a nightly CI job for it. Do not assume a Miri-clean history.
* The crate is not `no_std`, is not `#[forbid(unsafe_code)]`-compatible, and
  makes no performance or constant-factor guarantees relative to
  `std::vec::Vec`.
