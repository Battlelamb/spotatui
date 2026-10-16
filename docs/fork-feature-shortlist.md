# Fork feature shortlist: 5 candidates, ranked

**This is a research deliverable, not an implementation.** Nothing here has been coded. The
board picks which (if any) of these become implementation issues, in whatever order it wants.

## Method

- Read every **open** issue (50), **open** PR (4), and **open** Discussion (24) on
  `LargeModGames/spotatui` via the GitHub API (`gh`, read-only — no comments or reactions were
  left), sorted by reaction count and by comment count, plus the upstream wiki
  (`spotatui.wiki`, cloned read-only to `/tmp`, removed after reading).
- Cross-checked every candidate against `~/Projects/spotatui-contributions/docs/ROADMAP.md`
  (our own evidence-led plan) so this list doesn't repeat what that document already covers,
  dismisses, or flags as unready.
- Verified against the **current** fork source (`main`, same commit as upstream `main`,
  `50e48e9`) whether each request is already implemented, already superseded by a module
  rename, or already has an open PR — issue text is a snapshot from whenever it was filed, not
  proof of current repo state.
- All reaction counts on this repo are small (max 3); comment count and discussion upvotes were
  the more useful signal. Where I write "N comments", that's as of 2026-10-16.

## Ranked shortlist

| # | Feature | Value | Effort | Risk |
|---|---|---|---|---|
| 1 | Listening Party: make Shared Control actually relay guest commands | High | Low–Med | Low |
| 2 | Playlist sidebar: pin + custom sort order | Med–High | Low | Low |
| 3 | Theme presets: accessibility-focused defaults + remaining community ports | Medium | Low | Low |
| 4 | Private session toggle (TUI + GUI) | High | Medium–High | Medium |
| 5 | Podcast progress: resume points + "mark as played" | Med–High | Medium | Med–High |

---

## 1. Listening Party: make Shared Control actually relay guest commands

**Source signal:** upstream/#442 (bug, 3 comments) + Discussion #331 "Is it possible to create a
jam?" (General, open, no reply pointing the user at Listening Party). The wiki
(`Listening-Party.md`) documents **Shared Control** as a shipped mode: *"Guests can send playback
commands (play/pause/next/previous/seek) to the host, who executes them and re-broadcasts the new
state."* It isn't true today.

**What's actually there:** this is not a new feature, it's finishing one. The relay protocol,
host-side command handling, and the `IoEvent::PartyPlaybackCommand(sync::PlaybackAction)` variant
all exist and are wired into the Spotify-auth bypass gate (`src/infra/network/mod.rs:195-202`,
`:1019-1020`). What's missing is purely client-side: nothing in `src/tui/handlers/party.rs` or the
global transport keys in `src/tui/handlers/mod.rs` (`:240-257`, e.g. `toggle_playback`,
`next_track`, `seek_backwards`) ever dispatches that `IoEvent` when the local app is a guest
(`PartyStatus::Joined`) under `ControlMode::Shared`. A guest pressing Space today just calls
`Action::TogglePlayback` against their own (silent) local state — nothing goes over the relay.

**Design sketch:**
- Add a guard, checked once, at the top of the global transport-key match in
  `tui/handlers/mod.rs` (or inside `App::apply` for the handful of `Action` variants involved):
  when `app.party_status() == Joined` **and** the session's `control_mode == Shared`, translate
  the key to a `sync::PlaybackAction` and dispatch `IoEvent::PartyPlaybackCommand` instead of
  applying the local `Action`.
  - Follow the existing `playback_owner()` precedent in intent: this is an *additive* guard in
    front of the normal path, not a new owner in the predicate chain the roadmap's R1 already
    flags as sensitive — a guest who leaves the party or switches to Host-only falls straight
    back through the unmodified existing dispatch.
- Host side needs no change (it already executes and re-broadcasts, per the wiki). Scope this PR
  to the guest-side key translation only.
- Which keys: play/pause, next, previous, seek forward/back — the same list the wiki already
  promises. Volume and local-only things (sidebar navigation) stay local.

**Affected modules:**
- `src/tui/handlers/party.rs`, `src/tui/handlers/mod.rs` (guest-side key translation)
- `src/core/app/party.rs` (a read-only helper to answer "is this key allowed to pass through
  locally right now", if the guard needs to live there instead of in the handler)
- `src/infra/network/sync.rs`, `src/infra/network/mod.rs` — read-only reference; no change
  expected, since the relay/host path already works

**Test plan:**
- Unit tests with the house `app_with_x()` pattern asserting `IoEvent::PartyPlaybackCommand` is
  sent on the receiver when `PartyStatus::Joined` + `ControlMode::Shared`, and asserting it is
  **not** sent (local `Action` applies instead) under `ControlMode::HostOnly`, `Disconnected`, or
  `Hosting`.
- A ratatui `TestBackend` scenario: join a party (mock relay or a stub `PartyConnection`), toggle
  control mode, press the transport keys, assert the right `IoEvent` variant and payload.
- No real relay connection, no Spotify login, no audio — consistent with the "no telemetry/login"
  constraint on this work.

---

## 2. Playlist sidebar: pin + custom sort order

**Source signal:** upstream/#863 "Add ability to pin playlists" (2 comments, filed 2026-10-15) +
upstream/#657 "Sort playlists view" (3 comments, filed 2026-09-29, label `enhancement`). Both ask
for the same underlying pain — a long, unordered playlist sidebar — and neither has an open PR.

**Design sketch:** one user-facing improvement, two small PRs (per the "one concern each" rule):
- **Sort PR:** add a `behavior.playlist_sort` setting (e.g. `CreatedDate` (today's implicit
  order) / `Name` / `RecentlyPlayed`) following the existing `behavior.*` recipe in
  `core/user_config.rs` (`BehaviorConfigString`, `BehaviorConfig`, `load_behaviorconfig`,
  `save_config`) plus the matching `settings_schema.rs` / `settings_apply.rs` arms. Apply the sort
  when building the sidebar list, not by mutating `all_playlists: Vec<PlaylistInfo>`
  (`core/app/mod.rs:500`) in place — keep the fetched order as the source of truth and sort a view
  over it, the same separation `core/app/view.rs` already draws between data and presentation.
  "Recently played" needs a last-played timestamp per playlist, which doesn't exist yet — that's
  the one piece of new state (see below).
- **Pin PR:** a `Vec<String>` (or `HashSet`) of pinned playlist URIs, persisted in `state.yml`
  (per `AGENTS.md`: "a value that changes as the app runs goes in state, never config" — pin
  status is exactly that, not a hand-edited setting). A pinned-toggle key/action on the sidebar
  row, and pinned playlists always render first regardless of the active sort mode.
- Scope note: #863 also asks "or if it would grab the pinned playlists associated with the
  Spotify account" — the internal API's own pin/favorite concept is a separate, riskier
  investigation (undocumented endpoint); ship the local-only pin first, leave server-side sync as
  a follow-up if someone wants to investigate the API surface.

**Affected modules:**
- `src/core/user_config.rs` (new `behavior.playlist_sort` setting)
- `src/core/app/settings_schema.rs`, `src/core/app/settings_apply.rs` (setting wiring)
- `src/core/state.rs` (new persisted `pinned_playlists` field, sparse-patch friendly)
- `src/core/app/library.rs` (sidebar list construction — where `active_playlist_index` and the
  playlist rows are built)
- `src/tui/ui/` (sidebar draw function — pin indicator)
- `src/tui/handlers/` (pin-toggle key)

**Test plan:**
- Pure-function unit tests for the sort comparator (stable order for ties, each mode) and the pin
  partition (pinned-first, pin toggle idempotent) — following the house style of testing decisions
  as plain functions over scalars/structs, no `App`/network required.
- A `TestBackend` snapshot of the sidebar before/after pinning and before/after switching sort
  mode.
- A `state.yml` round-trip test: pin survives a save/reload, and an old `state.yml` without the
  field loads with an empty pin set (no migration needed, it's a new optional field).

---

## 3. Theme presets: accessibility-focused defaults + remaining community ports

**Source signal:** upstream/#3 "Add New Theme Presets" (3 comments, open since 2025-12-09,
labelled `good first issue` — i.e. upstream itself invites exactly this). Discussion traffic
elsewhere (`Discussions` General/Ideas) keeps referencing specific palettes.

**Reality check against current source:** this issue is partially stale. `src/core/theme.rs`
(moved from `user_config.rs`, where the 10-month-old issue still points) already ships
`Dracula, Nord, SolarizedDark, Monokai, Gruvbox, GruvboxLight, CatppuccinMocha, TokyoNight` plus
`Terminal`, `PookiePink`, `Vesper` and `Spotify` — most of the issue's own example list
(Catppuccin, Tokyo Night, Gruvbox) is already done. What's still genuinely missing, and still
explicitly asked for in the issue body, is the **accessibility angle**: *"accessibility-focused
themes (high contrast, colorblind-friendly)"* — there is no such preset today, and nothing in
`ThemeField`/`ThemePreset` currently encodes a contrast-ratio or colorblind-safe guarantee for
any preset, built-in or custom.

**Design sketch:**
- Add one or two new `ThemePreset` variants (e.g. `HighContrast`, and a colorblind-safe palette —
  Rosé Pine is still requested and unported too, lower-hanging fruit alongside it) following the
  exact four-place recipe the issue itself documents: enum variant, `all()`, `name()`,
  `to_theme()` match arms.
- For the accessibility ones specifically, pick colors against a documented contrast target
  (WCAG-AA-ish ratio for text/background pairs) rather than by eye, and say so in the PR
  description — the issue is asking for a guarantee, not just another palette.
- No settings-schema or persistence change: `ThemePreset` is already a selectable value, this is
  purely new match arms.

**Affected modules:**
- `src/core/theme.rs` only.

**Test plan:**
- Existing theme tests (if any) extended to assert every `ThemePreset::all()` entry has a
  `to_theme()` arm (exhaustiveness — likely already enforced by the match being non-wildcard;
  confirm and keep it that way).
- A small contrast-ratio unit test for the new accessibility preset(s): compute luminance contrast
  between `text`/background-ish field pairs and assert it clears a stated minimum, so "high
  contrast" is a checked claim, not a vibe.
- Manual TUI screenshot in the PR description showing the new preset against at least the track
  table and playbar, per the repo's "before/after screenshots" convention.

---

## 4. Private session toggle (TUI + GUI)

**Source signal:** upstream/#850 "Private session toggle in the TUI and GUI settings" (1 comment,
filed 2026-10-14, label `enhancement`). The issue is maintainer-authored and unusually
actionable: it already names the exact proto field, the files to touch on both the app and the
librespot-fork side, and a "Done when" acceptance list.

**What it is:** Spotify's own privacy feature — while a private session is on, plays don't appear
in "Recently played", don't feed the taste profile, and aren't visible to followers. spotatui has
no equivalent today; every native-streaming play counts against the account's profile with no way
to opt out short of pausing the app entirely.

**Verified against current source:** `is_private_session` is already read from the Connect device
list (`src/infra/spotify/player.rs:194`) and hardcoded `false` at one call site (`:376`) — i.e.
the app already understands the field exists on other devices' state, it just never sets its own.
That matches the issue's framing exactly: this is wiring a flag through, not inventing a new
protocol concept.

**Design sketch:**
- The flag lives on the Connect device state our librespot fork publishes
  (`connect.proto` field 11, per the issue), not in spotatui's own `App` state beyond a thin
  runtime toggle — so most of the work is in the **fork**, not this repo: a setter on
  `ConnectState` that flips `is_private_session` and re-sends `PutStateRequest`. That's a
  `spotatui-librespot-*` change, published and version-pinned before the app side can consume it
  (per `AGENTS.md`'s native-streaming section: "All seven crates are `=`-pinned in lockstep").
- App side, once the fork exposes the setter: a new `Action` (e.g. `TogglePrivateSession`), a
  settings row (`settings_schema.rs` / `settings_apply.rs`, gated on `Requirement::SpotifySession`
  like the other Spotify-only rows), a lock icon in the TUI playbar (`src/tui/ui/player.rs`) and
  the GUI player bar (`gui/src/PlayerBar.tsx`), and a GUI settings row (`gui/src/Settings.tsx`).
- **Runtime-only, by explicit design** (per the issue): never persisted to `config.yml` or
  `state.yml` — a restart always starts out of private session, matching Spotify's own behavior
  and avoiding a "user forgot it was on" footgun.
- Open design question the issue itself raises and leaves for the implementer: whether a private
  session should also suppress spotatui's **own** local listening history/stats/recap and Discord
  Rich Presence. Decide and state that explicitly in the PR description; don't let it default
  silently either way.

**Affected modules:**
- The `spotatui-librespot` fork (separate repo/crate — not in this tree): `connect/src/state.rs`,
  the `PutStateRequest` path. This PR must land and publish *before* the app-side PR, and the
  lockstep version bump covers all seven pinned crates together.
- `src/core/action/` (new `Action` variant)
- `src/core/app/settings_schema.rs`, `src/core/app/settings_apply.rs`
- `src/tui/ui/player.rs` (TUI playbar lock icon)
- `gui/src/PlayerBar.tsx`, `gui/src/Settings.tsx` (GUI lock icon + settings row)
- `src/infra/spotify/player.rs` (already reads the flag from other devices; no change expected)

**Test plan:**
- Fork side: a unit test on the new setter asserting the outgoing `PutStateRequest` carries the
  flipped `is_private_session` bit (and `incognito_mode` on `LegacyEndSong`/`RawCoreStream` if the
  spike confirms those need setting too).
- App side: `TestBackend` assertion that the settings row toggles the `Action` and the playbar
  draws the lock icon when the (test-doubled) session reports private-session-on.
- A GUI binding-regeneration check if the new `Action`/protocol type touches `gui/src/bindings/`.
- Explicitly **not** testable without live login whether "the official Spotify app shows the
  spotatui device as private" (one of the issue's own acceptance criteria) — call that out as a
  manual verification step in the PR, not something CI can assert.

---

## 5. Podcast progress: resume points + "mark as played"

**Source signal:** upstream/#794 "Episode resume points from the internal API" (1 comment) +
upstream/#788 "Support 'mark as played'" (2 comments), both filed 2026-10-13, both about the same
underlying gap: spotatui shows no played/resume state for podcast episodes at all.

**This is exactly the item our own ROADMAP.md already flagged** ("Podcast resume/played state... Low
implementation readiness, not a judgment that podcasts lack user value" — Jev priority 1.03, risk
1.91) — included here because the community signal is concrete and recent, but the ranking below
reflects that the roadmap's caution was correct: readiness, not value, is the blocker.

**Design sketch — two PRs, in order, with a hard investigation gate before either:**
- **Spike (not a PR):** #794 is precise about the cause: `EpisodeInfo.resume_point` exists as a
  domain field, but `episode_simplified` in `src/infra/spotify/json.rs:175` builds the Web-API-
  shaped JSON without ever setting a `resume_point` key, because the internal API's actual answer
  for "where did this user leave off" hasn't been located since the move off the real Web API
  (#752, referenced by the issue). Before any code: find what the internal API (the one the
  official client's own session already talks to) actually returns for episode playback
  position, by comparing the official app's traffic shape to what `infra/spotify/router.rs`
  currently sends for show/episode routes. If that answer doesn't exist or isn't practically
  discoverable, say so and stop — per the issue's own fallback: "hide the played column and
  progress bar until there is a source for them" is an acceptable, smaller PR on its own.
- **PR 1 (if the spike finds a source):** wire the real `resume_point` into `episode_simplified`,
  so the GUI's `PodcastPage.tsx` Played tag/progress bar and the TUI's played column — which
  already exist and already expect this field, per #794 — start rendering real data. This is a
  read-only data-plumbing fix, not a new feature surface.
- **PR 2 (#788, local-first "mark as played"):** a manual "mark as played" action, stored locally
  (not synced to Spotify — #788 explicitly says "right now it's ok to keep it local"), following
  the same `last_session.yml` / local-playback-state pattern already used for non-Spotify
  playback persistence (`core/persisted_playback.rs`). A played episode shows an icon in the list,
  independent of whatever PR 1 did or didn't manage to wire up.

**Affected modules:**
- `src/infra/spotify/json.rs` (`episode_simplified`), `src/infra/spotify/router.rs` (if a new
  internal-API route answer is needed) — PR 1, spike-dependent
- `gui/src/PodcastPage.tsx` — already consumes the field; verify behavior once real data flows,
  likely no GUI code change needed for PR 1
- `src/core/persisted_playback.rs` or a new sibling file (local played-state map, keyed by episode
  URI) — PR 2
- `src/tui/ui/` (played-column rendering), `gui/src/PodcastPage.tsx` (played tag) — PR 2 reads the
  new local state in addition to (or instead of, if the spike comes up empty) server data

**Test plan:**
- PR 1: a fixture-based test asserting `episode_simplified` emits the located `resume_point`
  shape; a regression test pinning today's `None` behavior removed.
- PR 2: pure-function tests for the local played-state map (mark/unmark, persists across a
  `state`-style save/reload, keyed correctly by episode URI not list index — per the repo's
  "address by identity, never by list ordinal" rule for `Action` payloads).
- Both: GUI binding regeneration (`gui/src/bindings/`) committed if any shared type changes, per
  the `gui` test leg's any-difference-fails rule.
- **Do not open PR 2 before the PR-1 spike concludes** — the issue itself frames the two as
  related but separable; shipping "mark as played" first and the real resume state later (or
  never) is fine, but conflating them into one PR risks blocking a safe local-only feature on an
  uncertain internal-API discovery.

---

## Considered and explicitly not shortlisted

| Candidate | Why not here |
|---|---|
| Cross-source "Similar Tracks" via Last.fm (upstream/#714) | **Overtaken by a merge, not just a judgment call.** #714 was filed 2026-10-07 on the premise that Spotify's `/recommendations` 403s for self-hosted (Development Mode) apps. PR #752, "drop the Web API and client id", merged 2026-10-11 — four days later. Current `main`'s `catalog::recommendations` (`src/infra/spotify/catalog.rs:226`) answers the route through the internal API (`get_apollo_station`) over the user's own session, with no client id and no Development Mode restriction at all. The exact failure the issue describes no longer exists on current main; the fallback it proposes would solve a problem the architecture migration already removed. This is the ROADMAP.md Stage-0 caution ("an open issue may describe an older dependency") playing out concretely, not hypothetically. Worth a maintainer reply pointing this out; not a PR. |
| Parametric EQ + AutoEQ (upstream/#782) | Our ROADMAP.md already scores this low-readiness/high-risk (1.99/1.90): touches both the decoded-source (`LocalPlayer`) and native-streaming (librespot fork) audio paths, and conflicts with bit-perfect mode (#783). Design-first, not a shortlist item. |
| Bit-perfect output (upstream/#783) | Same ROADMAP verdict (1.26/2.29) — hardware/DSP research question, not a bounded PR. |
| Persistent headless Connect daemon (upstream/#466) | ROADMAP flags lifecycle/recovery prerequisites first (1.79/2.33); also directly collides with the ghost-Connect-device failure mode (#297) the issue itself names. |
| Crossfader (Discussion #507) | Same category as EQ: needs two overlapping decode/stream sessions across both playback pipelines, with no existing hook. Not bounded enough for this list. |
| Localization / typed string table (upstream/#726) | Already has an **open PR** (#865, `feat(i18n): share typed English strings across terminal and GUI`) — would duplicate active work. |
| Flathub packaging (upstream/#805) | Already has an **open PR** (#804, `feat(desktop): add Flatpak files for Flathub`). |
| SMTC support (Discussion #248) | Already shipped: `windows-media` is in `default` and pulls `smtc-tokio` (`Cargo.toml:200`). The discussion reads like a bug on a specific machine ("media keys... aren't working on my machine"), not a missing feature — worth a maintainer reply, not a new PR. |
| RPM packaging (upstream/#416) | Real and well-specified, but it's a distribution/CI concern, not an in-app user feature, and the issue itself asks for `dnf install`-testing in a container before touching `cd.yml` — a packaging task, better tracked on its own rather than competing for one of five *feature* slots. |
| Sonos integration (Discussion #247) | A whole new output protocol/backend with no existing infra hook — large, open-ended, not a bounded first PR. |
| YouTube video playback via mpv (upstream/#727) | spotatui is a TUI-first audio player; video output is a different problem (an embedded or external video surface) with no clear design, and the issue itself has no concrete proposal, just a question. |
| Local Files sidebar path truncation (upstream/#621) | Well-specified and valuable, but it's a rendering correctness fix, not a new feature — good candidate for a normal bug-fix PR outside this exercise. |

## Sources

- `gh api search/issues` against `repo:LargeModGames/spotatui`, open issues and PRs, sorted by
  `reactions` and by `comments`, 2026-10-16.
- `gh api graphql` against the repo's `discussions` connection, sorted by `updatedAt`, 2026-10-16.
- `LargeModGames/spotatui.wiki` (`Listening-Party.md`, `Roadmap.md`, `Themes.md`), cloned
  read-only, not pushed anywhere, removed after reading.
- `~/Projects/spotatui-contributions/docs/ROADMAP.md`, read in full for cross-check.
- Current fork `main` source tree (commit `50e48e9`, == upstream `main`, v0.46.0) for every
  "is this already done / already in flight" claim above.
