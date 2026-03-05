use lib_core::entities::graph::Graph;
use lib_core::use_cases::load_graph::{GraphReader, GraphReaderError};
use lib_plantuml::adapters::graph_parser_plantuml_impl::GraphParserPlantumlImpl;
use ratatui::Terminal;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io;
use std::rc::Rc;

mod ascii_renderer;

fn main() {
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut stdout: io::Stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();

    let backend: ratatui::prelude::CrosstermBackend<io::Stdout> =
        ratatui::prelude::CrosstermBackend::new(stdout);
    let mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<io::Stdout>> =
        ratatui::Terminal::new(backend).unwrap();

    run_application(terminal, app);

    crossterm::terminal::disable_raw_mode().unwrap();
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .unwrap();
    terminal.show_cursor().unwrap();
}

struct App {
    source: String,
    graph: Option<Graph>,
    error: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            source: DEFAULT_SOURCE.to_string(),
            graph: None,
            error: None,
        }
    }

    fn parse(&mut self) {
        let parser = GraphParserPlantumlImpl::new();
        match smol::block_on(async { parser.read(&self.source).await }) {
            Ok(graph) => {
                self.graph = Some(graph);
                self.error = None;
            }
            Err(e) => {
                self.graph = None;
                self.error = Some(format_error(&e));
            }
        }
    }
}

fn format_error(err: &GraphReaderError) -> String {
    match err {
        GraphReaderError::Parse {
            source,
            message,
            line,
            column,
        } => format!("[{}:{}:{}] Parse Error: {}", source, line, column, message),
        GraphReaderError::Semantic { source, message } => {
            format!("[{}] Semantic Error: {}", source, message)
        }
    }
}

const DEFAULT_SOURCE: &str = r#"@startuml
class "Customer" as C
database "OrdersDB" as DB

C --> DB : "places order"
@enduml"#;

fn run_application(
    mut terminal: Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    app.parse();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => return Ok(()),
                    crossterm::event::KeyCode::Char('r') => app.parse(),
                    crossterm::event::KeyCode::Char('e') => {
                        app.source = edit_mode(&app.source);
                        app.parse();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(frame: &mut ratatui::Frame, app: &App) {
    let chunks: Rc<[ratatui::prelude::Rect]> = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(50),
        ])
        .split(frame.area());

    // Editor (edtui renders itself)
    frame.render_widget(app.editor.widget(), chunks[0]);

    let preview: Paragraph = Paragraph::new(app.output.as_str())
        .block(Block::default().title("Preview").borders(Borders::ALL));

    frame.render_widget(preview, chunks[1]);
}

fn edit_mode(current: &str) -> String {
    use std::process::Command;

    let temp_file = "/tmp/diagrama_edit.txt";
    std::fs::write(temp_file, current).ok();

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    Command::new(editor)
        .arg(temp_file)
        .output()
        .expect("Failed to open editor");

    std::fs::read_to_string(temp_file).unwrap_or_else(|_| current.to_string())
}
