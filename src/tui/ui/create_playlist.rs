use crate::core::app::{App, CreatePlaylistFocus, CreatePlaylistStage};
use crate::tui::theme::{EmphasisExt, ThemeExt};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Modifier, Style},
  text::Span,
  widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
  Frame,
};

fn centered_rect(bounds: Rect, width_pct: u16, height_pct: u16) -> Rect {
  let width = (bounds.width * width_pct / 100).max(1);
  let height = (bounds.height * height_pct / 100).max(1);
  let x = bounds.x + bounds.width.saturating_sub(width) / 2;
  let y = bounds.y + bounds.height.saturating_sub(height) / 3;
  Rect::new(x, y, width, height)
}

pub fn draw_create_playlist_form(f: &mut Frame<'_>, app: &App) {
  let area = centered_rect(f.area(), 80, 80);
  f.render_widget(Clear, area);

  match app.view.create_playlist_stage {
    CreatePlaylistStage::Name => draw_name_stage(f, app, area),
    CreatePlaylistStage::AddTracks => draw_add_tracks_stage(f, app, area),
  }
}

fn draw_name_stage(f: &mut Frame<'_>, app: &App, area: Rect) {
  let theme = &app.user_config.theme;

  let block = Block::default()
    .title(Span::styled(
      "Create Playlist (Esc to cancel)",
      Style::default()
        .fg(theme.header.into())
        .add_modifier(app.user_config.behavior.emphasis(Modifier::BOLD)),
    ))
    .borders(Borders::ALL)
    .style(theme.base_style())
    .border_style(Style::default().fg(theme.active.into()));
  f.render_widget(block, area);

  let inner = Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints([
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Length(1),
    ])
    .split(area);

  let label = Paragraph::new("Playlist name:").style(theme.base_style());
  f.render_widget(label, inner[0]);

  let name_text: String = app.view.create_playlist_name.iter().collect();
  let name_inner_width = inner[1].width.saturating_sub(2);
  let name_scroll_offset =
    if name_inner_width > 0 && app.view.create_playlist_name_cursor >= name_inner_width {
      app.view.create_playlist_name_cursor - name_inner_width + 1
    } else {
      0
    };
  let input = Paragraph::new(name_text)
    .style(theme.base_style())
    .scroll((0, name_scroll_offset))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.active.into())),
    );
  f.render_widget(input, inner[1]);
  f.set_cursor_position((
    inner[1].x + 1 + app.view.create_playlist_name_cursor - name_scroll_offset,
    inner[1].y + 1,
  ));

  let hint = Paragraph::new("Press Enter to continue, Esc to cancel")
    .style(Style::default().fg(theme.inactive.into()));
  f.render_widget(hint, inner[2]);
}

fn draw_add_tracks_stage(f: &mut Frame<'_>, app: &App, area: Rect) {
  let theme = &app.user_config.theme;
  let name: String = app.view.create_playlist_name.iter().collect();
  let title = format!(
    "Add Tracks to \"{}\" (Enter=create, Tab=switch panel, Esc=cancel)",
    name
  );

  let block = Block::default()
    .title(Span::styled(
      title,
      Style::default()
        .fg(theme.header.into())
        .add_modifier(app.user_config.behavior.emphasis(Modifier::BOLD)),
    ))
    .borders(Borders::ALL)
    .style(theme.base_style())
    .border_style(Style::default().fg(theme.active.into()));
  f.render_widget(block, area);

  let inner = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([Constraint::Length(3), Constraint::Min(5)])
    .split(area);

  // Search input
  let search_text: String = app.view.create_playlist_search_input.iter().collect();
  let search_border_style = if app.view.create_playlist_focus == CreatePlaylistFocus::SearchInput {
    Style::default().fg(theme.active.into())
  } else {
    Style::default().fg(theme.inactive.into())
  };
  let search_inner_width = inner[0].width.saturating_sub(2);
  let search_scroll_offset =
    if search_inner_width > 0 && app.view.create_playlist_search_cursor >= search_inner_width {
      app.view.create_playlist_search_cursor - search_inner_width + 1
    } else {
      0
    };
  let search_input = Paragraph::new(search_text)
    .style(theme.base_style())
    .scroll((0, search_scroll_offset))
    .block(
      Block::default()
        .title(Span::styled(
          "Search (Enter to search)",
          Style::default().fg(theme.header.into()),
        ))
        .borders(Borders::ALL)
        .border_style(search_border_style),
    );
  f.render_widget(search_input, inner[0]);
  if app.view.create_playlist_focus == CreatePlaylistFocus::SearchInput {
    f.set_cursor_position((
      inner[0].x + 1 + app.view.create_playlist_search_cursor - search_scroll_offset,
      inner[0].y + 1,
    ));
  }

  // Two-panel area: results + added tracks
  let panels = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    .split(inner[1]);

  // Left: search results
  let results_border_style = if app.view.create_playlist_focus == CreatePlaylistFocus::SearchResults
  {
    Style::default().fg(theme.active.into())
  } else {
    Style::default().fg(theme.inactive.into())
  };
  let result_items: Vec<ListItem> = app
    .create_playlist_search_results
    .iter()
    .map(|t| {
      let artist = t.artists.first().map(|a| a.as_str()).unwrap_or("Unknown");
      ListItem::new(format!("{} — {}", t.name, artist)).style(theme.base_style())
    })
    .collect();

  let mut results_state = ListState::default();
  if app.view.create_playlist_focus == CreatePlaylistFocus::SearchResults
    && !app.create_playlist_search_results.is_empty()
  {
    results_state.select(Some(app.view.create_playlist_selected_result));
  }

  let results_list = List::new(result_items)
    .block(
      Block::default()
        .title(Span::styled(
          "Results (Enter to add)",
          Style::default().fg(theme.header.into()),
        ))
        .borders(Borders::ALL)
        .border_style(results_border_style),
    )
    .highlight_style(
      Style::default()
        .fg(theme.selected.into())
        .add_modifier(app.user_config.behavior.emphasis(Modifier::BOLD)),
    )
    .style(theme.base_style());
  f.render_stateful_widget(results_list, panels[0], &mut results_state);

  // Right: added tracks
  let added_border_style = if app.view.create_playlist_focus == CreatePlaylistFocus::AddedTracks {
    Style::default().fg(theme.active.into())
  } else {
    Style::default().fg(theme.inactive.into())
  };

  let added_items: Vec<ListItem> = app
    .create_playlist_tracks
    .iter()
    .map(|t| {
      let artist = t.artists.first().map(|a| a.as_str()).unwrap_or("Unknown");
      ListItem::new(format!("{} — {}", t.name, artist)).style(theme.base_style())
    })
    .collect();

  let mut added_state = ListState::default();
  if app.view.create_playlist_focus == CreatePlaylistFocus::AddedTracks
    && !app.create_playlist_tracks.is_empty()
  {
    added_state.select(Some(app.view.create_playlist_selected_result));
  }

  let added_tracks_title = format!(
    "Added ({}) — d=remove, Enter=create",
    app.create_playlist_tracks.len()
  );
  let added_list = List::new(added_items)
    .block(
      Block::default()
        .title(Span::styled(
          added_tracks_title,
          Style::default().fg(theme.header.into()),
        ))
        .borders(Borders::ALL)
        .border_style(added_border_style),
    )
    .highlight_style(
      Style::default()
        .fg(theme.selected.into())
        .add_modifier(app.user_config.behavior.emphasis(Modifier::BOLD)),
    )
    .style(theme.base_style());
  f.render_stateful_widget(added_list, panels[1], &mut added_state);
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{backend::TestBackend, Terminal};

  fn rendered_cursor(app: &App, area: Rect) -> (u16, u16) {
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal
      .draw(|f| draw_create_playlist_form(f, app))
      .unwrap();
    let cursor = terminal.backend().cursor_position();
    (cursor.x, cursor.y)
  }

  /// The exact name-input `Rect` `draw_name_stage` renders into, so a test can
  /// compute the real right-border column instead of approximating it.
  fn name_input_rect(area: Rect) -> Rect {
    let form_area = centered_rect(area, 80, 80);
    Layout::default()
      .direction(Direction::Vertical)
      .margin(2)
      .constraints([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
      ])
      .split(form_area)[1]
  }

  /// The exact search-input `Rect` `draw_add_tracks_stage` renders into.
  fn search_input_rect(area: Rect) -> Rect {
    let form_area = centered_rect(area, 80, 80);
    Layout::default()
      .direction(Direction::Vertical)
      .margin(1)
      .constraints([Constraint::Length(3), Constraint::Min(5)])
      .split(form_area)[0]
  }

  /// Typing a playlist name past the input box width must scroll the text so
  /// the cursor never escapes the border, instead of clipping silently.
  #[test]
  fn name_input_scrolls_so_cursor_stays_inside_the_box() {
    let mut app = App::default();
    let area = Rect::new(0, 0, 20, 10);
    let input_rect = name_input_rect(area);
    let text_inner_width = input_rect.width.saturating_sub(2);

    let long_name: Vec<char> = "a".repeat(text_inner_width as usize + 10).chars().collect();
    app.view.create_playlist_name_cursor = long_name.len() as u16;
    app.view.create_playlist_name = long_name;

    let (cursor_x, _) = rendered_cursor(&app, area);
    let right_border_x = input_rect.x + input_rect.width.saturating_sub(1);
    assert!(
      cursor_x < right_border_x,
      "cursor at x={cursor_x} escaped the input box border at x={right_border_x}"
    );
  }

  /// Same guarantee for the Add Tracks stage's search box.
  #[test]
  fn search_input_scrolls_so_cursor_stays_inside_the_box() {
    let mut app = App::default();
    app.view.create_playlist_stage = CreatePlaylistStage::AddTracks;
    app.view.create_playlist_focus = CreatePlaylistFocus::SearchInput;
    let area = Rect::new(0, 0, 20, 10);
    let input_rect = search_input_rect(area);
    let text_inner_width = input_rect.width.saturating_sub(2);

    let long_query: Vec<char> = "b".repeat(text_inner_width as usize + 10).chars().collect();
    app.view.create_playlist_search_cursor = long_query.len() as u16;
    app.view.create_playlist_search_input = long_query;

    let (cursor_x, _) = rendered_cursor(&app, area);
    let right_border_x = input_rect.x + input_rect.width.saturating_sub(1);
    assert!(
      cursor_x < right_border_x,
      "cursor at x={cursor_x} escaped the search box border at x={right_border_x}"
    );
  }

  /// Below the scroll threshold the cursor position is untouched (no
  /// off-by-one from an unconditional scroll offset).
  #[test]
  fn short_name_input_does_not_scroll() {
    let mut app = App::default();
    let area = Rect::new(0, 0, 30, 10);
    let input_rect = name_input_rect(area);

    let short_name: Vec<char> = "hi".chars().collect();
    app.view.create_playlist_name_cursor = short_name.len() as u16;
    app.view.create_playlist_name = short_name;

    let (cursor_x, _) = rendered_cursor(&app, area);
    assert_eq!(
      cursor_x,
      input_rect.x + 1 + 2,
      "unscrolled cursor should sit right after the typed text"
    );
  }
}
