use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use ratatui::Terminal;
use ratatui::crossterm;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::terminal::EnterAlternateScreen;
use ratatui::crossterm::terminal::LeaveAlternateScreen;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use tui_textarea::TextArea;

use crate::Settings;
use crate::traits::Runnable;
use crate::transport::TransportClient;

pub struct Client {
    transport_client: TransportClient,
    settings: Settings,
}

impl Client {
    async fn spawn_server(settings: &Settings) -> anyhow::Result<TransportClient> {
        let exe = std::env::current_exe()?;

        Command::new(exe)
            .arg("--server")
            .spawn()?;

        let start = Instant::now();
        let timeout = Duration::from_millis(settings.client_timeout);

        loop {
            let transport_client = TransportClient::connect(&settings.socket_path).await;

            if transport_client.is_ok() {
                return transport_client
            }

            if start.elapsed() > timeout {
                anyhow::bail!("Server timed out")
            }

            std::thread::sleep(Duration::from_millis(settings.client_retry_ms));
        }
    }
}

impl Client {
    async fn create_input_area() -> TextArea<'static> {
        let text_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Input")
            .padding(Padding::symmetric(1, 0));

        let mut text_area = TextArea::default();

        text_area.set_cursor_line_style(Style::default());
        text_area.set_block(text_block);
        text_area.set_placeholder_text("Ask anything");

        text_area
    }
}

impl Runnable for Client {
    async fn run(settings: Settings) -> anyhow::Result<()> {
        let mut transport_client = match TransportClient::connect(&settings.socket_path).await {
            Ok(x) => x,
            Err(_) => Self::spawn_server(&settings).await?,
        };

        // Enter crossterm mode
        let mut stdout = std::io::stdout();

        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, EnterAlternateScreen)?;

        // Initialize UI components
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut text_area = Self::create_input_area().await;
        let chat_text = Paragraph::new("Messages will appear here");

        // Main event loop
        loop {
            // Handle server events
            if let Some(event) = transport_client.recv().await? {
                // TODO: Handle server events
            };

            // Handle crossterm events
            if crossterm::event::poll(Duration::from_millis(300))? {
                let event = crossterm::event::read()?;

                if let Event::Key(key) = event {
                    // TODO: More useful keybinds
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if key.code == KeyCode::Char('q') {
                            break
                        } else if key.code == KeyCode::Char('a') {
                            text_area.select_all();
                        }
                    } else if key.modifiers.contains(KeyModifiers::ALT) {
                        if key.code == KeyCode::Enter {
                            text_area = Self::create_input_area().await;
                        }
                    } else {
                        text_area.input(key);
                    }
                }
            }

            // UI code
            terminal.draw(|f| {
                let height = std::cmp::min(text_area.lines().len() + 2, 10) as u16;
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(height),
                    ])
                    .split(f.area());

                f.render_widget(&chat_text, chunks[0]);
                f.render_widget(&text_area, chunks[1]);
            })?;
        }

        // Exit crossterm mode
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}
