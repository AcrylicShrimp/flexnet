# AGENTS.md

## Repository Layout

- The repository root is controlled by `Taskfile.yml`.
- Rust crates must live under `crates/*`.

## Rust Workflow

- Use `cargo clippy` instead of `cargo check` for Rust validation.
- Before adding a Rust crate, run `cargo search <crate>` to verify the latest available version.
- When specifying crate versions:
  - If the crate is `>= 1.0.0`, specify only the major version.
  - If the crate is `< 1.0.0`, specify only the major and minor version.
- Use the modern Rust module layout:
  - prefer `foo.rs` with sibling files under `foo/`
  - do not add `mod.rs`

## Implementation Philosophy

This codebase prefers hard cutovers and structural simplicity over incremental migration.

When implementing changes, follow these rules:

1. Prefer replacing old paths entirely over preserving backward compatibility.
2. Do not keep legacy code, compatibility shims, adapters, fallbacks, aliases, or dual paths unless explicitly required.
3. If a new design is chosen, remove the old design completely rather than wiring both together.
4. Simplicity means fewer moving parts:
   - fewer layers
   - fewer abstractions
   - fewer branches
   - fewer configuration modes
   - fewer indirections
5. Do not introduce abstractions for hypothetical future use.
6. Local duplication is preferable to premature abstraction when it keeps the design more obvious.
7. During refactors, temporary breakage is acceptable. Do not preserve bad structure just to keep tests or compilation passing at every intermediate step.
8. The final result should be a clean end state, not a transitional architecture.
9. If forced to choose between:
   - keeping old behavior through extra indirection, or
   - performing a hard cutover with a smaller and clearer design,
     choose the hard cutover.
10. When in doubt, delete more and keep less.
11. Do not introduce heuristics unless they are truly unavoidable for the problem being solved.
12. If a heuristic is unavoidable, it must be explicitly justified in the implementation and reported clearly when the task is finished.

## Explicit Anti-Patterns

Avoid these unless explicitly requested:

- compatibility layers
- wrapper-on-wrapper structures
- preserving old and new flows at the same time
- migration scaffolding left in production code
- feature flags used only to avoid removing old code
- generic abstractions that are only used once
- interfaces/traits created without multiple real implementations
- configuration added to support legacy behavior
- heuristic behavior where deterministic rules or explicit contracts would work

## Refactor Rule

If the existing structure fights the new design, do not bend the new design around the old one.
Delete the obsolete structure and reshape the code around the new design directly.

## Reporting Expectations

When finishing a task, explicitly report:

- what old code paths were deleted
- what compatibility mechanisms were intentionally not preserved
- what simplifications were made

A change is considered simpler if it reduces one or more of:

- number of concepts
- number of code paths
- number of indirections
- number of public entry points
- number of configuration options
- number of types involved in the feature

Do not introduce interfaces/traits unless there are multiple concrete implementations that already exist or are required now.
Never leave both the old path and the new path alive at the same time unless explicitly required.
When reporting work, include:

1. what was deleted
2. what was intentionally not preserved
3. what branching or indirection was removed
4. why the final structure is simpler
