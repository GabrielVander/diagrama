use lib_core::use_cases::load_graph::LoadGraph;
use lib_plantuml::infrastructure::adapters::plant_uml_graph_gateway::PlantUmlGraphGateway;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::adapters::tui_presenter::{TuiEvent, TuiPresenter, TuiPresenterImpl};
mod adapters;

mod ascii_renderer;

fn main() {
    let plantuml_graph_gateway: Arc<PlantUmlGraphGateway> = Arc::new(PlantUmlGraphGateway::new());
    let load_plantuml_graph: Arc<LoadGraph<PlantUmlGraphGateway>> =
        Arc::new(LoadGraph::new(plantuml_graph_gateway.clone()));
    let presenter: Arc<TuiPresenterImpl<LoadGraph<PlantUmlGraphGateway>>> =
        Arc::new(TuiPresenterImpl::new(load_plantuml_graph.clone()));

    crossterm::terminal::enable_raw_mode().unwrap();

    let mut stdout: io::Stdout = io::stdout();
    ratatui::crossterm::execute!(stdout, crossterm::event::EnableBracketedPaste).unwrap();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();

    let backend: ratatui::prelude::CrosstermBackend<io::Stdout> =
        ratatui::prelude::CrosstermBackend::new(stdout);
    let mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<io::Stdout>> =
        ratatui::Terminal::new(backend).unwrap();

    run_application(&mut terminal, presenter.clone()).unwrap();

    crossterm::terminal::disable_raw_mode().unwrap();
    ratatui::crossterm::execute!(std::io::stdout(), crossterm::event::DisableBracketedPaste)
        .unwrap();
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .unwrap();
    terminal.show_cursor().unwrap();
}

fn run_application(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    presenter: Arc<impl TuiPresenter>,
) -> io::Result<()> {
    let mut editor_state: edtui::EditorState = edtui::EditorState::default();
    let mut event_handler: edtui::EditorEventHandler = edtui::EditorEventHandler::default();
    let presenter_clone = presenter.clone();
    let mut preview_scroll: (u16, u16) = (0, 0);

    loop {
        let current_tui_event: TuiEvent = presenter_clone.state();

        let preview_value: String = {
            match current_tui_event {
                TuiEvent::PreviewReady(value) => value,
                TuiEvent::LoadingGraph => "Loading...".to_owned(),
                TuiEvent::Error(e) => format!("ERROR: {}", e),
                _ => "Press 'Ctrl+s' to load preview".to_owned(),
            }
        };

        terminal.draw(|f| {
            let chunks: Rc<[ratatui::prelude::Rect]> = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(50),
                    ratatui::layout::Constraint::Percentage(50),
                ])
                .split(f.area());

            f.render_widget(
                edtui::EditorView::new(&mut editor_state)
                    .theme(dracula_theme())
                    .wrap(true)
                    .syntax_highlighter(None)
                    .line_numbers(edtui::LineNumbers::Relative)
                    .tab_width(4),
                chunks[0],
            );

            let preview: ratatui::widgets::Paragraph =
                ratatui::widgets::Paragraph::new(preview_value)
                    .block(
                        ratatui::widgets::Block::default()
                            .title("Preview")
                            .borders(ratatui::widgets::Borders::ALL),
                    )
                    .scroll(preview_scroll);

            f.render_widget(preview, chunks[1]);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            let event: crossterm::event::Event = crossterm::event::read()?;

            let crossterm::event::Event::Key(key) = event else {
                continue;
            };

            if key.kind == crossterm::event::KeyEventKind::Press {
                let is_ctrl_pressed = key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL);

                match key.code {
                    crossterm::event::KeyCode::Char('q') if is_ctrl_pressed => {
                        return Ok(());
                    }
                    crossterm::event::KeyCode::Char('c') if is_ctrl_pressed => {
                        return Ok(());
                    }

                    crossterm::event::KeyCode::Char('s') if is_ctrl_pressed => {
                        // Reset scroll to top-left when rendering a new graph
                        preview_scroll = (0, 0);

                        presenter_clone
                            .clone()
                            .load_graph(editor_state.lines.to_string())
                            .detach();
                    }

                    // Handle Scrolling with Ctrl + Arrows
                    crossterm::event::KeyCode::Up if is_ctrl_pressed => {
                        preview_scroll.0 = preview_scroll.0.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Down if is_ctrl_pressed => {
                        preview_scroll.0 = preview_scroll.0.saturating_add(1);
                    }
                    crossterm::event::KeyCode::Left if is_ctrl_pressed => {
                        preview_scroll.1 = preview_scroll.1.saturating_sub(1);
                    }
                    crossterm::event::KeyCode::Right if is_ctrl_pressed => {
                        preview_scroll.1 = preview_scroll.1.saturating_add(1);
                    }
                    // Forward all other keys to the editor
                    _ => {
                        event_handler.on_key_event(key, &mut editor_state);
                    }
                }
            }
        }
    }
}

fn dracula_theme() -> edtui::EditorTheme<'static> {
    edtui::EditorTheme::default()
        .block(
            ratatui::widgets::Block::default()
                .title(" Code ")
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(
                    ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(98, 114, 164)),
                ), // Muted Purple
        )
        .base(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(40, 42, 54)) // Deep Dark Background
                .fg(ratatui::style::Color::Rgb(248, 248, 242)), // Off-white Foreground
        )
        .cursor_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(255, 121, 198)) // Bright Pink Cursor
                .fg(ratatui::style::Color::Rgb(40, 42, 54)),
        )
        .selection_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(68, 71, 90)) // Highlight Background
                .fg(ratatui::style::Color::Rgb(248, 248, 242)),
        )
        .line_numbers_style(
            ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(98, 114, 164)), // Muted Purple line numbers
        )
}

/// A cool, relaxed dark theme inspired by Nord.
pub fn nord_theme() -> edtui::EditorTheme<'static> {
    edtui::EditorTheme::default()
        .block(
            ratatui::widgets::Block::default()
                .title(" Code ")
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(
                    ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(76, 86, 106)),
                ), // Grayish Blue
        )
        .base(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(46, 52, 64)) // Soft Charcoal
                .fg(ratatui::style::Color::Rgb(216, 222, 233)), // Frost White
        )
        .cursor_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(136, 192, 208)) // Frost Blue Cursor
                .fg(ratatui::style::Color::Rgb(46, 52, 64)),
        )
        .selection_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::Rgb(67, 76, 94)) // Darker Grayish Blue Selection
                .fg(ratatui::style::Color::Rgb(216, 222, 233)),
        )
        .line_numbers_style(
            ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(76, 86, 106)), // Grayish Blue line numbers
        )
}
