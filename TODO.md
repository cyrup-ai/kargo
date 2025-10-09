# TODO: Errors & Warnings Elimination Plan

- **Objective**: 0 errors and 0 warnings across the entire workspace.
- **Rules**: No stubs. No suppression without written approval. Production-quality fixes only.

## Current Status (from `cargo check` and `cargo clippy --workspace --all-targets`)

- **Errors**: 0
- **Warnings**: 6 (Clippy)

---

## W1: clippy::collapsible_if in `plugins/native/kargo-turd/src/project_scanner.rs:54`
- Message: this `if` statement can be collapsed
- Suggestion: `if let Ok(entry) = res && entry.file_type().is_dir() { ... }`

QA W1:
- Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1-10. Provide specific feedback.

## W2: clippy::collapsible_if in `plugins/native/kargo-turd/src/project_scanner.rs:111`
- Message: this `if` statement can be collapsed
- Suggestion: `if let Ok(entry) = res && entry.file_type().is_dir() { ... }`

QA W2:
- Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1-10. Provide specific feedback.

## W3: clippy::needless_borrow in `plugins/native/kargo-turd/src/analyzers/dependency_analyzer.rs:153`
- Message: this expression creates a reference which is immediately dereferenced
- Suggestion: change `self.add_path(&i.path());` to `self.add_path(i.path());`

QA W3:
- Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1-10. Provide specific feedback.

## W4: clippy::collapsible_if in `plugins/native/kargo-turd/src/analyzers/dependency_analyzer.rs:246`
- Message: this `if` statement can be collapsed
- Suggestion: combine nested `if let` using `&& let` guard

QA W4:
- Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1-10. Provide specific feedback.

## W5: clippy::collapsible_if in `kargo-cli/src/plugins/trait_scanner.rs:285`
- Message: this `if` statement can be collapsed
- Suggestion: `if let Some((_, path, _)) = &impl_item.trait_ && let Some(segment) = path.segments.last() { ... }`

QA W5:
- Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1-10. Provide specific feedback.

## W6: duplicate warnings in tests (clippy duplicates)
- Message: duplicates of the above in test targets
- Plan: apply the same code fixes; duplicates will disappear.

QA W6:
- Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1-10. Provide specific feedback.

---

## Next Steps

1) Apply minimal code changes to address W1–W5.
2) Re-run `cargo clippy --workspace --all-targets` and `cargo check --workspace`.
3) Iterate until warnings reach 0. Then run product-level usage tests.
