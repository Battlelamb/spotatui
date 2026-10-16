# Fork notes

This is a development fork of LargeModGames/spotatui (origin `Battlelamb/spotatui`).

- **Upstream contributions** go through the board only; agents never open PRs or comments on `LargeModGames/*`.
- **UI/UX work scope:** presentation layer only (`src/tui/**`, `gui/**`, docs). `src/core`, `src/infra`, `src/runtime`, `src/cli`, dependencies and gate baselines are out of scope for UI/UX PRs.
- **Feature candidates:** see [fork-feature-shortlist.md](fork-feature-shortlist.md).
- **Local gates:** `cargo fmt --all`, `cargo clippy --no-default-features --features telemetry,tui -- -D warnings`, `cargo test --no-default-features --features telemetry,tui`, `cargo check --locked`, `tools/check_gates_ratchet.sh origin/main`.
