#![cfg(test)]
//! Wall-clock micro-benchmarks for the table draw path (SPO-6). Not asserted
//! on: timings vary with machine load, so these are `#[ignore]`d and meant to
//! be run manually with `--nocapture` when changing this code, to check a
//! change actually helps instead of trusting intuition.
//!
//! `cargo test --no-default-features --features telemetry,tui --release \
//!   perf_bench -- --ignored --nocapture`
use super::*;
use crate::core::app::{ActiveBlock, App, RouteId};
use crate::core::plugin_api::TrackInfo;
use crate::core::test_helpers::full_track;
use chrono::Utc;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use rspotify::model::{
  context::{Actions, CurrentPlaybackContext},
  device::Device,
  enums::{CurrentlyPlayingType, RepeatState},
  DeviceType, PlayableItem,
};
use std::time::{Duration, Instant};

fn track(i: u32) -> TrackInfo {
  TrackInfo {
    uri: Some(format!("spotify:track:{i:022}")),
    name: format!("Track {i} - a reasonably long title to mimic real data"),
    artists: vec![format!("Artist {i}"), format!("Feature Artist {i}")],
    album: format!("Album {i}"),
    duration_ms: 180_000,
    id: Some(format!("{i:022}")),
    album_id: None,
    artist_refs: Vec::new(),
    is_playable: true,
    is_local: false,
    track_number: i + 1,
    explicit: false,
    image_url: None,
    release_date: Some("1997-01-20".to_string()),
  }
}

/// A `TrackTable` route with 500 tracks and an active playback context, wide
/// enough to exercise the sidebar, playbar and a full-height song table at
/// once — the common case while actually listening to something.
#[allow(deprecated)]
fn populated_app() -> App {
  let mut app = App::default();
  app.track_table.tracks = (0..500).map(track).collect();
  app.view.track_table_index = 250;
  app.current_playback_context = Some(CurrentPlaybackContext {
    device: Device {
      id: Some("device".to_string()),
      is_active: true,
      is_private_session: false,
      is_restricted: false,
      name: "spotatui".to_string(),
      _type: DeviceType::Computer,
      volume_percent: Some(50),
    },
    repeat_state: RepeatState::Off,
    shuffle_state: false,
    context: None,
    timestamp: Utc::now(),
    progress: Some(chrono::Duration::milliseconds(60_000)),
    is_playing: true,
    item: Some(PlayableItem::Track(full_track(
      "0123456789012345678901",
      "Now Playing Track With A Long Name",
    ))),
    currently_playing_type: CurrentlyPlayingType::Track,
    actions: Actions::default(),
  });
  app.push_navigation_stack(RouteId::TrackTable, ActiveBlock::TrackTable);
  app.view.size = crate::core::geometry::Viewport {
    width: 160,
    height: 50,
  };
  app
}

fn bench(label: &str, iters: usize, mut f: impl FnMut()) {
  for _ in 0..50 {
    f(); // warmup: let the allocator settle before timing
  }
  let start = Instant::now();
  for _ in 0..iters {
    f();
  }
  let elapsed = start.elapsed();
  println!(
    "{label}: {:?} total, {:?}/iter over {} iters",
    elapsed,
    elapsed / iters as u32,
    iters
  );
}

#[test]
#[ignore]
fn bench_full_frame_track_table() {
  let app = populated_app();
  let area = Rect::new(0, 0, 160, 50);
  let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
  bench("full_frame(track_table)", 5000, || {
    terminal.draw(|f| draw_main_layout(f, &app)).unwrap();
  });
}

#[test]
#[ignore]
fn bench_song_table_only() {
  let app = populated_app();
  let area = Rect::new(0, 0, 120, 47);
  let mut terminal = Terminal::new(TestBackend::new(120, 47)).unwrap();
  bench("song_table_only", 20000, || {
    terminal.draw(|f| draw_song_table(f, &app, area)).unwrap();
  });
}

/// Isolates exactly what `draw_table`'s row-building changed: cloning a row's
/// `Vec<String>` vs. borrowing it as `&str`s, which is all that happens for a
/// row that is neither the playing track nor liked (the common case). The A
/// and B samples interleave in the same loop so machine-load drift (this box
/// runs other agents' builds concurrently) affects both arms equally, instead
/// of skewing a comparison taken from two separate `cargo test` runs the way
/// the two benches above do.
#[test]
#[ignore]
fn bench_table_row_build_clone_vs_borrow() {
  let format: Vec<String> = (0..7)
    .map(|i| format!("cell value {i} of a realistic width"))
    .collect();
  let rows_per_frame = 47; // a typical full-height song table viewport

  let clone_variant = || {
    for _ in 0..rows_per_frame {
      let cloned = format.clone();
      std::hint::black_box(&cloned);
    }
  };
  let borrow_variant = || {
    for _ in 0..rows_per_frame {
      let borrowed: Vec<&str> = format.iter().map(String::as_str).collect();
      std::hint::black_box(&borrowed);
    }
  };

  let iters = 20000;
  let mut clone_total = Duration::ZERO;
  let mut borrow_total = Duration::ZERO;
  for _ in 0..iters {
    let s = Instant::now();
    clone_variant();
    clone_total += s.elapsed();
    let s = Instant::now();
    borrow_variant();
    borrow_total += s.elapsed();
  }
  println!(
    "row building, {rows_per_frame} rows/frame: clone {:?}/frame vs borrow {:?}/frame ({:.1}x)",
    clone_total / iters,
    borrow_total / iters,
    clone_total.as_secs_f64() / borrow_total.as_secs_f64()
  );
}
