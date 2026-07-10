use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::{
    io::{self, stdout},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};
use unicode_width::UnicodeWidthStr; // To calculate cursor position

struct App {
    input: String,
    messages: Vec<String>,
    scroll_state: ListState,
}
#[tokio::main]
async fn main() -> io::Result<()> {
    // make the term
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // the channels
    let (ui_tx, mut ui_rx) = mpsc::channel::<String>(100);
    let (net_tx, mut net_rx) = mpsc::channel::<String>(100);

    //  Network Task
    tokio::spawn(async move {
        let addr = "127.0.0.1:5000";
        if let Ok(stream) = TcpStream::connect(addr).await {
            let (reader, mut writer) = stream.into_split();
            let mut server_reader = BufReader::new(reader).lines();

            loop {
                tokio::select! {
                    // Receive message from Server
                    res = server_reader.next_line() => {
                        match res {
                            Ok(Some(line)) => { let _ = ui_tx.send(line).await; }
                            _ => break, // Server disconnected
                        }
                    }
                    // Send message to Server
                    Some(msg) = net_rx.recv() => {
                        let _ = writer.write_all(format!("{}\n", msg).as_bytes()).await;
                    }
                }
            }
        }
    });

    // Main App State
    let mut app = App {
        input: String::new(),
        messages: Vec::new(),
        scroll_state: ListState::default(),
    };

    // had ai help with this
    // Main UI Loop
    loop {
        // Draw the UI
        terminal.draw(|f| ui(f, &mut app))?;

        // Check for Network updates (non-blocking)
        while let Ok(msg) = ui_rx.try_recv() {
            app.messages.push(msg);
            // Auto-scroll to bottom
            let last_idx = app.messages.len().saturating_sub(1);
            app.scroll_state.select(Some(last_idx));
        }

        // Handle Keyboard Events
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Enter => {
                        let msg: String = app.input.drain(..).collect();
                        if !msg.is_empty() {
                            let _ = net_tx.send(msg).await;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => break, // Exit program
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    // Vertical Layout: Top is Chat (Min 3), Bottom is Input (3 lines)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(f.area());

    // 1. Render Chat Messages
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| {
            let style = if m.contains("Gemini-Bot") {
                Style::default().fg(Color::Cyan)
            } else if m.contains("Enter code") || m.contains("Enter your name") {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(m, style)))
        })
        .collect();

    let chat = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title(" Chat Room "))
        .highlight_symbol(">> ");

    f.render_stateful_widget(chat, chunks[0], &mut app.scroll_state);

    // Render Input Box
    let input = Paragraph::new(app.input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Input (Enter to Send / Esc to Quit) "),
    );
    f.render_widget(input, chunks[1]);

    // position it at the start of the input chunk + the width of the current text
    // The +1 is for the border.
    f.set_cursor_position((chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1));
}
