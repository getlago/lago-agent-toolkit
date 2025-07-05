use anyhow::Result;
use arboard::Clipboard;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use textwrap::fill;
use tokio::sync::{mpsc, Mutex};

use crate::agent::LagoAgent;

pub struct ChatApp {
    agent: Arc<Mutex<LagoAgent>>,
    messages: Vec<Message>,
    input: String,
    input_mode: InputMode,
    messages_state: ListState,
    scroll_offset: usize,
    is_streaming: bool,
    current_response: String,
    stream_receiver: Option<mpsc::UnboundedReceiver<StreamUpdate>>,
    clipboard: Option<Clipboard>,
    show_debug: bool,
    debug_logs: Vec<DebugLog>,
    debug_state: ListState,
}

#[derive(Debug, Clone)]
pub struct DebugLog {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub enum StreamUpdate {
    Chunk(String),
    Error(String),
    Complete,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

impl ChatApp {
    pub fn new(agent: LagoAgent) -> Self {
        let mut messages_state = ListState::default();
        messages_state.select(Some(0));
        
        let mut debug_state = ListState::default();
        debug_state.select(Some(0));
        
        // Initialize clipboard (may fail on some systems)
        let clipboard = Clipboard::new().ok();
        
        let mut app = Self {
            agent: Arc::new(Mutex::new(agent)),
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "🚀 Welcome to Lago AI Agent! I'm your intelligent assistant powered by advanced AI technology. I can help you manage and analyze your Lago invoices with natural language commands. Ready to get started?".to_string(),
                    timestamp: chrono::Local::now(),
                }
            ],
            input: String::new(),
            input_mode: InputMode::Editing,
            messages_state,
            scroll_offset: 0,
            is_streaming: false,
            current_response: String::new(),
            stream_receiver: None,
            clipboard,
            show_debug: false,
            debug_logs: Vec::new(),
            debug_state,
        };
        
        // Add initial debug log
        app.add_debug_log(LogLevel::Info, "UI", "Lago AI Agent initialized");
        
        app
    }
    
    pub fn add_debug_log(&mut self, level: LogLevel, source: &str, message: &str) {
        self.debug_logs.push(DebugLog {
            timestamp: chrono::Local::now(),
            level,
            message: message.to_string(),
            source: source.to_string(),
        });
        
        // Keep only last 1000 logs to prevent memory issues
        if self.debug_logs.len() > 1000 {
            self.debug_logs.remove(0);
        }
        
        // Auto-scroll to bottom in debug panel
        if !self.debug_logs.is_empty() {
            self.debug_state.select(Some(self.debug_logs.len() - 1));
        }
    }
    
    pub fn toggle_debug_panel(&mut self) {
        self.show_debug = !self.show_debug;
        self.add_debug_log(LogLevel::Info, "UI", 
            if self.show_debug { "Debug panel opened" } else { "Debug panel closed" });
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            // Check for streaming updates
            if let Some(receiver) = &mut self.stream_receiver {
                match receiver.try_recv() {
                    Ok(update) => {
                        match update {
                            StreamUpdate::Chunk(content) => {
                                self.current_response.push_str(&content);
                                if let Some(last_message) = self.messages.last_mut() {
                                    if last_message.role == MessageRole::Assistant {
                                        last_message.content = self.current_response.clone();
                                    }
                                }
                                self.add_debug_log(LogLevel::Debug, "Stream", &format!("Received chunk: {} chars", content.len()));
                            }
                            StreamUpdate::Error(error) => {
                                if let Some(last_message) = self.messages.last_mut() {
                                    if last_message.role == MessageRole::Assistant {
                                        last_message.content = format!("❌ Error: {}\n\n💡 Tip: Check your MISTRAL_API_KEY environment variable and MCP server connection.", error);
                                    }
                                }
                                self.is_streaming = false;
                                self.stream_receiver = None;
                                self.add_debug_log(LogLevel::Error, "Stream", &format!("Stream error: {}", error));
                            }
                            StreamUpdate::Complete => {
                                // Update the agent's conversation history with the final streamed response
                                if let Some(last_message) = self.messages.last() {
                                    if last_message.role == MessageRole::Assistant {
                                        let final_content = last_message.content.clone();
                                        let agent = self.agent.clone();
                                        tokio::spawn(async move {
                                            let mut agent_guard = agent.lock().await;
                                            agent_guard.add_assistant_message(final_content);
                                        });
                                    }
                                }
                                self.is_streaming = false;
                                self.stream_receiver = None;
                                self.add_debug_log(LogLevel::Info, "Stream", "Stream completed successfully");
                            }
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // No updates available
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        // Stream ended
                        self.is_streaming = false;
                        self.stream_receiver = None;
                    }
                }
            }

            // Check for keyboard input with a timeout to allow UI updates during streaming
            let timeout = std::time::Duration::from_millis(50);
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match self.input_mode {
                            InputMode::Normal => match key.code {
                                KeyCode::Char('e') => {
                                    self.input_mode = InputMode::Editing;
                                    self.add_debug_log(LogLevel::Info, "UI", "Switched to editing mode");
                                }
                                KeyCode::Char('q') => {
                                    self.add_debug_log(LogLevel::Info, "UI", "User requested quit");
                                    return Ok(());
                                }
                                KeyCode::Char('d') => {
                                    self.toggle_debug_panel();
                                }
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Copy selected message
                                    self.copy_selected_message()?;
                                    self.add_debug_log(LogLevel::Info, "UI", "Copied selected message");
                                }
                                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Copy all messages
                                    self.copy_all_messages()?;
                                    self.add_debug_log(LogLevel::Info, "UI", "Copied all messages");
                                }
                                KeyCode::Up => {
                                    // Navigate up in messages or debug logs
                                    if self.show_debug {
                                        if let Some(selected) = self.debug_state.selected() {
                                            if selected > 0 {
                                                self.debug_state.select(Some(selected - 1));
                                            }
                                        }
                                    } else {
                                        if let Some(selected) = self.messages_state.selected() {
                                            if selected > 0 {
                                                self.messages_state.select(Some(selected - 1));
                                            }
                                        }
                                    }
                                }
                                KeyCode::Down => {
                                    // Navigate down in messages or debug logs
                                    if self.show_debug {
                                        if let Some(selected) = self.debug_state.selected() {
                                            if selected + 1 < self.debug_logs.len() {
                                                self.debug_state.select(Some(selected + 1));
                                            }
                                        }
                                    } else {
                                        if let Some(selected) = self.messages_state.selected() {
                                            if selected + 1 < self.messages.len() {
                                                self.messages_state.select(Some(selected + 1));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            },
                            InputMode::Editing => match key.code {
                                KeyCode::Enter => {
                                    if !self.input.trim().is_empty() && !self.is_streaming {
                                        let message = self.input.trim().to_string();
                                        self.input.clear();
                                        
                                        self.add_debug_log(LogLevel::Info, "UI", &format!("User sent message: {}", message));
                                        
                                        // Add user message immediately
                                        self.messages.push(Message {
                                            role: MessageRole::User,
                                            content: message.clone(),
                                            timestamp: chrono::Local::now(),
                                        });
                                        
                                        // Start streaming response
                                        self.start_streaming_response(message).await?;
                                        
                                        // Scroll to bottom
                                        self.scroll_to_bottom();
                                    }
                                }
                                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Paste from clipboard
                                    self.paste_from_clipboard()?;
                                }
                                KeyCode::Char(c) => {
                                    self.input.push(c);
                                }
                                KeyCode::Backspace => {
                                    self.input.pop();
                                }
                                KeyCode::Esc => {
                                    self.input_mode = InputMode::Normal;
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }
    }

    async fn start_streaming_response(&mut self, message: String) -> Result<()> {
        self.is_streaming = true;
        self.current_response = String::new();
        
        self.add_debug_log(LogLevel::Info, "Stream", "Starting streaming response");

        // Add empty assistant message that will be filled by streaming
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: String::new(),
            timestamp: chrono::Local::now(),
        });

        // Create channel for streaming updates
        let (sender, receiver) = mpsc::unbounded_channel::<StreamUpdate>();
        self.stream_receiver = Some(receiver);

        // Clone agent for async task
        let agent = self.agent.clone();
        
        self.add_debug_log(LogLevel::Debug, "Stream", "Spawning async streaming task");
        
        // Spawn streaming task
        tokio::spawn(async move {
            let mut agent_guard = agent.lock().await;

            let stream_result = agent_guard.process_message_stream(&message).await;
            
            match stream_result {
                Ok(mut stream) => {
                    // Process streaming chunks
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                // Send all chunks, even empty ones, to preserve order
                                if let Err(_) = sender.send(StreamUpdate::Chunk(chunk)) {
                                    break; // Receiver dropped
                                }
                            }
                            Err(e) => {
                                let _ = sender.send(StreamUpdate::Error(e.to_string()));
                                break;
                            }
                        }
                    }
                    let _ = sender.send(StreamUpdate::Complete);
                }
                Err(e) => {
                    let _ = sender.send(StreamUpdate::Error(format!("Streaming error: {}", e)));
                }
            }
        });

        Ok(())
    }

    fn scroll_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.messages_state.select(Some(self.messages.len() - 1));
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        // Modern AI-style color scheme
        let primary_color = Color::Rgb(0, 255, 255);    // Cyan
        let secondary_color = Color::Rgb(138, 43, 226); // Blue Violet
        let accent_color = Color::Rgb(255, 20, 147);    // Deep Pink
        let text_color = Color::Rgb(230, 230, 230);     // Light Gray
        let background_color = Color::Rgb(20, 20, 30);  // Dark Blue
        let assistant_color = Color::Rgb(0, 255, 127);  // Spring Green
        let user_color = Color::Rgb(135, 206, 235);     // Sky Blue
        let system_color = Color::Rgb(255, 165, 0);     // Orange
        let debug_color = Color::Rgb(255, 255, 0);      // Yellow
        
        // Create main layout with modern spacing
        let main_chunks = if self.show_debug {
            // Split horizontally for debug panel
            Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                .split(f.area())
        } else {
            // Full width for normal UI
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.area())
        };
        
        let chat_area = main_chunks[0];
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),      // Header
                    Constraint::Min(5),         // Chat area
                    Constraint::Length(4),      // Input area
                    Constraint::Length(3),      // Help area
                ]
                .as_ref(),
            )
            .split(chat_area);

        // AI Header with gradient-like effect
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("🤖 ", Style::default().fg(primary_color)),
                Span::styled("LAGO", Style::default().fg(primary_color).add_modifier(Modifier::BOLD)),
                Span::styled(" AI ", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                Span::styled("AGENT", Style::default().fg(secondary_color).add_modifier(Modifier::BOLD)),
                Span::styled(" ⚡", Style::default().fg(accent_color)),
                if self.show_debug {
                    Span::styled(" [DEBUG]", Style::default().fg(debug_color).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("", Style::default())
                },
            ]),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", 
                            Style::default().fg(primary_color)),
            ]),
        ])
        .style(Style::default().fg(text_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(primary_color))
                .style(Style::default().bg(background_color))
        )
        .alignment(Alignment::Center);

        f.render_widget(header, chunks[0]);

        // Format messages with modern styling
        let width = chunks[1].width - 4;
        let formatted_messages: Vec<(Text, MessageRole)> = self
            .messages
            .iter()
            .map(|m| (Self::format_message_modern(m, width), m.role.clone()))
            .collect();

        // Chat messages with AI-style colors
        let messages: Vec<ListItem> = formatted_messages
            .into_iter()
            .map(|(content, role)| {
                let style = match role {
                    MessageRole::User => Style::default().fg(user_color),
                    MessageRole::Assistant => Style::default().fg(assistant_color),
                    MessageRole::System => Style::default().fg(system_color),
                    MessageRole::Tool => Style::default().fg(accent_color),
                };
                ListItem::new(content).style(style)
            })
            .collect();

        let messages_list = List::new(messages)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(vec![
                        Span::styled("💬 ", Style::default().fg(primary_color)),
                        Span::styled("CONVERSATION", Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
                    ])
                    .title_alignment(Alignment::Left)
                    .border_style(Style::default().fg(primary_color))
                    .style(Style::default().bg(background_color)),
            )
            .style(Style::default().fg(text_color))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 40, 60))
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(messages_list, chunks[1], &mut self.messages_state);

        // Modern input field
        let input_style = match self.input_mode {
            InputMode::Normal => Style::default().fg(text_color),
            InputMode::Editing => Style::default().fg(primary_color),
        };

        let input_title = if self.is_streaming {
            vec![
                Span::styled("🔄 ", Style::default().fg(accent_color)),
                Span::styled("PROCESSING", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
            ]
        } else {
            vec![
                Span::styled("✨ ", Style::default().fg(primary_color)),
                Span::styled("MESSAGE INPUT", Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
            ]
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(input_title)
            .title_alignment(Alignment::Left)
            .border_style(Style::default().fg(if self.input_mode == InputMode::Editing { primary_color } else { Color::Rgb(100, 100, 100) }))
            .style(Style::default().bg(background_color));

        let input_text = if self.is_streaming {
            "⚡ AI is thinking... Please wait for the response to complete"
        } else {
            &self.input
        };

        let input_paragraph = Paragraph::new(input_text)
            .style(input_style)
            .block(input_block)
            .wrap(Wrap { trim: true });

        f.render_widget(input_paragraph, chunks[2]);

        // Set cursor position with modern styling
        if self.input_mode == InputMode::Editing && !self.is_streaming {
            f.set_cursor_position((
                chunks[2].x + (self.input.len() as u16 % (chunks[2].width - 2)) + 1,
                chunks[2].y + 1 + (self.input.len() as u16 / (chunks[2].width - 2)),
            ));
        }

        // Modern help panel
        let help_text = match self.input_mode {
            InputMode::Normal => vec![
                Line::from(vec![
                    Span::styled("🎯 ", Style::default().fg(primary_color)),
                    Span::styled("NAVIGATION: ", Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
                    Span::styled("e", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" edit | ", Style::default().fg(text_color)),
                    Span::styled("q", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" quit | ", Style::default().fg(text_color)),
                    Span::styled("d", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" debug | ", Style::default().fg(text_color)),
                    Span::styled("↑↓", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" navigate | ", Style::default().fg(text_color)),
                    Span::styled("Ctrl+C", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" copy | ", Style::default().fg(text_color)),
                    Span::styled("Ctrl+A", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" copy all", Style::default().fg(text_color)),
                ]),
            ],
            InputMode::Editing => vec![
                Line::from(vec![
                    Span::styled("⌨️ ", Style::default().fg(primary_color)),
                    Span::styled("EDITING: ", Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
                    Span::styled("Esc", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" stop | ", Style::default().fg(text_color)),
                    Span::styled("Enter", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" send | ", Style::default().fg(text_color)),
                    Span::styled("Ctrl+V", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::styled(" paste", Style::default().fg(text_color)),
                ]),
            ],
        };

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(text_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(vec![
                        Span::styled("🔧 ", Style::default().fg(primary_color)),
                        Span::styled("CONTROLS", Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
                    ])
                    .title_alignment(Alignment::Left)
                    .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
                    .style(Style::default().bg(background_color))
            );

        f.render_widget(help, chunks[3]);

        // Debug panel (if enabled)
        if self.show_debug {
            let debug_area = main_chunks[1];
            self.render_debug_panel(f, debug_area, primary_color, text_color, background_color, debug_color);
        }
    }
    
    fn render_debug_panel(&mut self, f: &mut Frame, area: ratatui::layout::Rect, primary_color: Color, text_color: Color, background_color: Color, debug_color: Color) {
        let debug_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        // Format debug logs
        let debug_items: Vec<ListItem> = self
            .debug_logs
            .iter()
            .map(|log| {
                let timestamp = log.timestamp.format("%H:%M:%S%.3f").to_string();
                let level_color = match log.level {
                    LogLevel::Debug => Color::Rgb(150, 150, 150),
                    LogLevel::Info => Color::Rgb(100, 200, 255),
                    LogLevel::Warning => Color::Rgb(255, 200, 0),
                    LogLevel::Error => Color::Rgb(255, 100, 100),
                };
                
                let level_text = match log.level {
                    LogLevel::Debug => "DBG",
                    LogLevel::Info => "INF",
                    LogLevel::Warning => "WRN",
                    LogLevel::Error => "ERR",
                };

                // Calculate available width for content (account for borders and padding)
                let content_width = area.width.saturating_sub(4) as usize;
                
                // Create the prefix
                let prefix = format!("[{}] {} {}: ", timestamp, level_text, log.source);
                
                // Wrap the message content if it's too long
                let full_line = format!("{}{}", prefix, log.message);
                let wrapped_lines: Vec<String> = if full_line.len() > content_width {
                    let wrapped = fill(&full_line, content_width);
                    wrapped.lines().map(|s| s.to_string()).collect()
                } else {
                    vec![full_line]
                };

                // Create lines for the wrapped content
                let mut lines = Vec::new();
                for (i, line) in wrapped_lines.iter().enumerate() {
                    if i == 0 {
                        // First line - try to parse and format with colors
                        if line.starts_with(&prefix) {
                            let message_part = line.strip_prefix(&prefix).unwrap_or(line).to_string();
                            lines.push(Line::from(vec![
                                Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::Rgb(100, 100, 100))),
                                Span::styled(format!("{} ", level_text), Style::default().fg(level_color).add_modifier(Modifier::BOLD)),
                                Span::styled(format!("{}: ", log.source), Style::default().fg(primary_color)),
                                Span::styled(message_part, Style::default().fg(text_color)),
                            ]));
                        } else {
                            // Fallback for wrapped first line
                            lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(text_color)),
                            ]));
                        }
                    } else {
                        // Continuation lines with indentation
                        lines.push(Line::from(vec![
                            Span::styled(format!("    {}", line), Style::default().fg(text_color)),
                        ]));
                    }
                }

                let content = Text::from(lines);
                ListItem::new(content)
            })
            .collect();

        let debug_list = List::new(debug_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(vec![
                        Span::styled("🐛 ", Style::default().fg(debug_color)),
                        Span::styled("DEBUG LOGS", Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" ({} logs)", self.debug_logs.len()), Style::default().fg(Color::Rgb(100, 100, 100))),
                    ])
                    .title_alignment(Alignment::Left)
                    .border_style(Style::default().fg(debug_color))
                    .style(Style::default().bg(background_color)),
            )
            .style(Style::default().fg(text_color))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 40, 60))
                    .fg(debug_color)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(debug_list, debug_chunks[0], &mut self.debug_state);
    }

    fn format_message_modern(message: &Message, width: u16) -> Text<'_> {
        let timestamp = message.timestamp.format("%H:%M:%S").to_string();
        let (role_icon, role_name, role_color) = match message.role {
            MessageRole::User => ("👤", "YOU", Color::Rgb(135, 206, 235)),
            MessageRole::Assistant => ("🤖", "AI", Color::Rgb(0, 255, 127)),
            MessageRole::System => ("ℹ️", "SYS", Color::Rgb(255, 165, 0)),
            MessageRole::Tool => ("🔧", "TOOL", Color::Rgb(255, 20, 147)),
        };

        let wrapped_content = fill(&message.content, width as usize - 6);

        let mut lines = vec![
            Line::from(vec![
                Span::styled(format!("{} ", role_icon), Style::default().fg(role_color)),
                Span::styled(role_name, Style::default().fg(role_color).add_modifier(Modifier::BOLD)),
                Span::styled(" • ", Style::default().fg(Color::Rgb(100, 100, 100))),
                Span::styled(timestamp, Style::default().fg(Color::Rgb(150, 150, 150))),
            ]),
            Line::from(vec![
                Span::styled("╭─", Style::default().fg(Color::Rgb(60, 60, 60))),
                Span::styled("─".repeat(width as usize - 4), Style::default().fg(Color::Rgb(60, 60, 60))),
                Span::styled("─╮", Style::default().fg(Color::Rgb(60, 60, 60))),
            ]),
        ];

        for line in wrapped_content.lines() {
            lines.push(Line::from(vec![
                Span::styled("│ ", Style::default().fg(Color::Rgb(60, 60, 60))),
                Span::styled(line.to_string(), Style::default().fg(Color::Rgb(230, 230, 230))),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("╰─", Style::default().fg(Color::Rgb(60, 60, 60))),
            Span::styled("─".repeat(width as usize - 4), Style::default().fg(Color::Rgb(60, 60, 60))),
            Span::styled("─╯", Style::default().fg(Color::Rgb(60, 60, 60))),
        ]));
        
        lines.push(Line::from("")); // Empty line for spacing

        Text::from(lines)
    }

    fn format_message_static(message: &Message, width: u16) -> Text<'_> {
        let timestamp = message.timestamp.format("%H:%M:%S").to_string();
        let role_prefix = match message.role {
            MessageRole::User => "👤 You",
            MessageRole::Assistant => "🤖 Agent",
            MessageRole::System => "ℹ️ System",
            MessageRole::Tool => "🔧 Tool",
        };

        let header = format!("{} ({})", role_prefix, timestamp);
        let wrapped_content = fill(&message.content, width as usize - 4);

        let mut lines = vec![
            Line::from(vec![
                Span::styled(header, Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
        ];

        for line in wrapped_content.lines() {
            lines.push(Line::from(line.to_string()));
        }

        lines.push(Line::from(""));
        Text::from(lines)
    }

    fn format_message<'a>(&self, message: &'a Message, width: u16) -> Text<'a> {
        Self::format_message_static(message, width)
    }

    fn copy_selected_message(&mut self) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            if let Some(selected) = self.messages_state.selected() {
                if let Some(message) = self.messages.get(selected) {
                    let content_to_copy = format!("{}: {}", 
                        match message.role {
                            MessageRole::User => "You",
                            MessageRole::Assistant => "Agent",
                            MessageRole::System => "System",
                            MessageRole::Tool => "Tool",
                        },
                        message.content
                    );
                    
                    if let Err(e) = clipboard.set_text(content_to_copy) {
                        eprintln!("Failed to copy to clipboard: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    fn copy_all_messages(&mut self) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            let mut all_content = String::new();
            
            for message in &self.messages {
                let role_name = match message.role {
                    MessageRole::User => "You",
                    MessageRole::Assistant => "Agent", 
                    MessageRole::System => "System",
                    MessageRole::Tool => "Tool",
                };
                
                all_content.push_str(&format!("{}: {}\n\n", role_name, message.content));
            }
            
            if let Err(e) = clipboard.set_text(all_content) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        }
        Ok(())
    }

    fn paste_from_clipboard(&mut self) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            if let Ok(content) = clipboard.get_text() {
                // Only paste if we're in editing mode
                if self.input_mode == InputMode::Editing {
                    self.input.push_str(&content);
                }
            }
        }
        Ok(())
    }
}
