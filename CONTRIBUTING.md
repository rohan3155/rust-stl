# Contributing

Thanks for taking a look at `rust-stl`. This is a small, dependency-free,
educational crate, so the contribution process is deliberately lightweight.

## Prerequisites

* Rust **1.85** or newer (the crate uses edition 2024 and declares
  `rust-version = "1.85"`). Install via [rustup](https://rustup.rs/).
* The `rustfmt` and `clippy` components:

  ```sh
  rustup component add rustfmt clippy
  ```

## Development setup

```sh
git clone https://github.com/rohan3155/rust-stl.git
cd rust-stl
cargo build
```

There are no runtime or development dependencies. If you believe one is
necessary, open an issue first and explain why — adding a dependency to a crate
whose selling point is having none needs a strong justification.

## Checks to run before opening a pull request

All of these must pass. They are the same checks CI runs.

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc --all-features
cargo test --examples --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

Running `cargo fmt` (without `--check`) rewrites the files for you.

You can also run each example end to end, which catches runtime panics that
compilation alone will not:

```sh
for example in basic_usage iteration filtering_and_sorting capacity_management; do
    cargo run --quiet --example "$example"
done
```

## Working on the `unsafe` code

`Vector<T>` is built on raw pointers and manual allocation, so it contains a
fair amount of `unsafe`. Two rules apply:

1. **Every `unsafe` block needs a `// SAFETY:` comment that states the
   invariant** being relied on — for example "index < len, and slots below len
   are initialized", not "read the element at index". A comment that merely
   restates what the code does is not enough.
2. Keep `len`, `capacity` and the initialized prefix consistent *at every
   point a user callback can run*. A panicking predicate, `Clone`, `PartialEq`
   or destructor must leave the vector valid — leaking is acceptable, double
   dropping is not.

[Miri](https://github.com/rust-lang/miri) is the most useful tool here. It needs
a nightly toolchain and is not run in CI by default:

```sh
rustup +nightly component add miri
cargo +nightly miri test
```

If your change touches pointer arithmetic, allocation, or the owned iterator,
please run Miri before submitting and mention the result in the pull request.

## Tests

The integration suite lives in `tests/` and is organised by behaviour, not by
method:

| File | Covers |
|---|---|
| `tests/core_behavior.rs` | Construction, growth, indexing, mutation, filtering, sorting, searching, rotation, slicing |
| `tests/capacity.rs` | Reservation, shrinking, retained capacity, overflow and error paths |
| `tests/edge_cases.rs` | Empty/single-element vectors, zero-sized types, non-`Copy` values, nested vectors |
| `tests/iterators.rs` | The three iteration forms and the iterator trait contracts |
| `tests/memory_safety.rs` | Exact destructor counts and panic safety |
| `tests/traits.rs` | Standard trait implementations and generic usage |

Unit tests for private invariants live in `src/collections/vector.rs`.

Guidelines:

* **Add a regression test for every bug fix.** The test should fail before the
  fix and pass after it; say so in the pull request.
* Prefer asserting documented guarantees over implementation details. Do not
  assert an exact capacity produced by the growth policy — only that the
  capacity is sufficient.
* Do not weaken or delete an existing test to make a change pass. If a test
  encodes wrong behaviour, change it deliberately and explain why.
* Do not silence warnings or add `#[allow(...)]` to get a green build. Fix the
  cause; if a lint is genuinely wrong, discuss it in the issue first.

## Documentation

Every public item needs a doc comment. New public methods should cover purpose,
arguments, return value, panic/error conditions, and complexity where it is
meaningful. Reuse the existing style: `///` comments with `# Examples`, `#
Panics`, `# Errors` and `# Complexity` sections where they apply.

Examples in doc comments are compiled and run by `cargo test --doc`, so they
must be correct. Prefer a realistic example over a trivial one-liner.

## Commit and pull-request expectations

* Keep each commit to **one logical change** and make sure it builds. A commit
  that leaves the tree broken is not acceptable.
* Use a short conventional prefix in the subject line: `feat`, `fix`,
  `refactor`, `test`, `docs`, `perf`, `chore`. Existing history uses this style,
  e.g. `fix(vector): stop double-dropping elements in append`.
* Explain *why* in the body, especially for safety-related changes, and note the
  test that covers the change.
* Keep pull requests focused. Unrelated formatting or refactoring makes review
  harder; split it out.
* By opening a pull request you agree to license your contribution under the
  crate's dual MIT/Apache-2.0 terms (see below).

## Reporting bugs

Use the repository's issue tracker:
<https://github.com/rohan3155/rust-stl/issues>.

Anything that can cause undefined behaviour through the public API should be
reported privately instead — see [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contribution is dual licensed under the
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE) licenses, at the user's
option, without any additional terms or conditions.

## Git hooks and local validation

Install the repository-local hooks once per clone:

```sh
./scripts/install-hooks.sh
```

This sets `core.hooksPath` locally (it does not touch global Git configuration)
and makes the scripts executable. Verify with:

```sh
git config --local --get core.hooksPath
test -x .githooks/pre-push && echo "pre-push hook executable"
```

The pre-commit hook runs only the fast formatting check:

```sh
cargo fmt --all -- --check
```

The pre-push hook runs the full validation script:

```sh
./scripts/validate.sh
```

That script runs, in order:

1. `cargo fmt --all -- --check`
2. `cargo test --all-targets --all-features`
3. `cargo test --doc --all-features`
4. `cargo clippy --all-targets --all-features -- -D warnings`
5. `cargo doc --no-deps --all-features`
6. `cargo package --locked`

To validate manually at any time:

```sh
./scripts/validate.sh
```

## Publishing

Do not call `cargo publish` directly. Use the wrapper so validation and the
dry run always happen first:

```sh
./scripts/publish.sh          # validation + dry run only
./scripts/publish.sh --publish # validation + dry run + real publish
```

Pass registry or package-selection arguments through unchanged, for example:

```sh
./scripts/publish.sh --registry my-registry --publish
```

The wrapper never uses `--no-verify` or `--allow-dirty` and never publishes
automatically during a Git push. GitHub Actions runs the same essential checks
on pushes and pull requests but does not publish.

## Troubleshooting failed checks

- Formatting: run `cargo fmt --all` and review the diff.
- Tests: run the failing target directly, for example
  `cargo test --test vector` or `cargo test --doc`.
- Clippy: address the reported lint; do not add `#[allow(...)]`.
- Documentation: run `cargo doc --no-deps --all-features` and inspect warnings.
- Packaging: ensure the working tree is clean and `Cargo.lock` is up to date;
  `cargo package --locked` refuses dirty trees by design.
