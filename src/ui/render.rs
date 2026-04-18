use crate::app::App;
use crate::commands::APP_NAME;
use crossterm::{
    cursor, execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color as RColor, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use std::io::{self, Write};

pub fn intro_art() -> [&'static str; 22] {
    [
        "             -@@@@@@@#      #@@@@@@@-             ",
        "           %@@@@@@@@@@@@#*@@@@@@@@@@@@%           ",
        "         .@@@@@@@%%@@@@@@@@@@@@@%@@@@@@@.         ",
        "         @@@@@%     .#@@@@@@%.     %@@@@@         ",
        "        -@@@@@        -@@@@+        @@@@@-        ",
        "   .=@@@@@@@@@         @@@@         @@@@@@@@@+.   ",
        " .@@@@@@@@@@@@.        @@@@        .@@@@@@@@@@@@. ",
        ".@@@@@@@%@@@@@@=      .@@@@:      -@@@@@@%@@@@@@@.",
        "%@@@@#      *@@@@%..-@@@@@@@@=..%@@@@#      #@@@@%",
        "@@@@@-       -@@@@@@@@@@@@@@@@@@@@@@=       -@@@@@",
        "%@@@@%        @@@@@@#.      .*@@@@@@        #@@@@%",
        ".@@@@@@       @@@@.            .@@@@       @@@@@@.",
        " .@@@@@@@-..=@@%.                .%@@+..-@@@@@@@. ",
        "   #@@@@@@@@@@:                    .@@@@@@@@@@#   ",
        "     .@@@@@@@-                      :@@@@@@@:     ",
        "       @@@@@*                        *@@@@@       ",
        "       @@@@@                          @@@@@       ",
        "       @@@@@                          @@@@@.      ",
        "       @@@@@:       .=*%@@%*=.       .@@@@@       ",
        "       +@@@@@@%%@@@@@@@@@@@@@@@@@@@%@@@@@@*       ",
        "        *@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*        ",
        "          -@@@@@@@@+          +@@@@@@@@-          ",
    ]
}

fn read_line_masked(stdout: &mut impl Write) -> io::Result<String> {
    use crossterm::event::{read, Event, KeyCode, KeyEventKind};
    terminal::enable_raw_mode()?;
    let mut buf = String::new();
    loop {
        if let Event::Key(key) = read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        execute!(stdout, Print("\r\n"))?;
                        break;
                    }
                    KeyCode::Char(c) => {
                        buf.push(c);
                        execute!(stdout, Print("*"))?;
                        stdout.flush()?;
                    }
                    KeyCode::Backspace => {
                        if buf.pop().is_some() {
                            execute!(stdout, cursor::MoveLeft(1), Print(" "), cursor::MoveLeft(1))?;
                            stdout.flush()?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    terminal::disable_raw_mode()?;
    Ok(buf)
}

pub fn prompt_connection_info() -> io::Result<(String, String, String)> {
    let mut stdout = io::stdout();
    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    
    execute!(
        stdout,
        cursor::Hide,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All)
    )?;

    let art = intro_art();
    let art_len = art.len() as f32;
    let start_y = term_h.saturating_sub(art.len() as u16 + 8) / 2;

    for (i, line) in art.iter().enumerate() {
        let x = term_w.saturating_sub(line.len() as u16) / 2;
        
        let r = 255;
        let g = (100.0 * (1.0 - i as f32 / art_len)) as u8;
        let b = (50.0 * (i as f32 / art_len)) as u8;

        execute!(
            stdout,
            cursor::MoveTo(x, start_y + i as u16),
            SetForegroundColor(Color::Rgb { r, g, b }),
            Print(line),
        )?;
        
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

    let subtitle = "C O N N E C T I O N   W I Z A R D";
    let sub_x = term_w.saturating_sub(subtitle.len() as u16) / 2;
    std::thread::sleep(std::time::Duration::from_millis(200));
    execute!(
        stdout,
        cursor::MoveTo(sub_x, start_y + art.len() as u16 + 2),
        SetForegroundColor(Color::DarkGrey),
        Print(subtitle),
    )?;

    let prompt_x = term_w.saturating_sub(20) / 2;
    std::thread::sleep(std::time::Duration::from_millis(150));
    
    // NICK
    execute!(
        stdout,
        cursor::MoveTo(prompt_x, start_y + art.len() as u16 + 4),
        SetForegroundColor(Color::Red),
        Print("Nickname: "),
        cursor::Show
    )?;
    stdout.flush()?;
    let mut nick = String::new();
    io::stdin().read_line(&mut nick)?;

    // ROOM
    execute!(
        stdout,
        cursor::MoveTo(prompt_x, start_y + art.len() as u16 + 5),
        SetForegroundColor(Color::Red),
        Print("Room ID:  "),
    )?;
    stdout.flush()?;
    let mut room = String::new();
    io::stdin().read_line(&mut room)?;

    // PASSWORD
    execute!(
        stdout,
        cursor::MoveTo(prompt_x, start_y + art.len() as u16 + 6),
        SetForegroundColor(Color::Magenta),
        Print("Password: "),
    )?;
    stdout.flush()?;
    let password = read_line_masked(&mut stdout)?;
    
    let nick = nick.trim();
    let nick = if nick.is_empty() { "You" } else { nick };
    let room = room.trim();
    let room = if room.is_empty() { "global" } else { room };

    Ok((nick.to_string(), room.to_string(), password.clone()))
}

pub fn help_panel_progress(app: &App) -> f32 {
    let elapsed = app.help_toggled_at.elapsed().as_millis() as f32;
    (elapsed / 160.0).clamp(0.0, 1.0)
}

pub fn help_panel_lines() -> [&'static str; 9] {
    [
        "Commands",
        "/help        toggle this panel",
        "/clear       clear messages",
        "/nick NAME   change nickname",
        "/save        save chat to file",
        "/me TEXT     action message",
        "/demo        add demo messages",
        "/top         jump to top",
        "/bottom      jump to bottom",
    ]
}

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Layout: Header (3), Main Chat, Footer/Input (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(size);

    // 1. HEADER
    let status_text = if std::time::Instant::now() <= app.status_until {
        app.status.clone()
    } else {
        "Commands: /help /clear /nick /save".to_string()
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(RColor::Red));
        
    let header_content = ratatui::text::Line::from(vec![
        Span::styled(format!(" {} ", APP_NAME), Style::default().fg(RColor::LightRed).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" | Room: {} | ", app.room), Style::default().fg(RColor::DarkGray)),
        Span::styled(format!("{} ", status_text), Style::default().fg(RColor::Yellow)),
    ]);
    
    let header = Paragraph::new(header_content)
        .block(title_block)
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // 2. CHAT HISTORY
    let inner_chat_area = chunks[1];
    
    // We render messages manually line by line to support slide animation and advanced alignment
    render_messages(f, app, inner_chat_area);

    // 3. INPUT/FOOTER
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(RColor::LightRed));

    let prefix = Span::styled(" > ", Style::default().fg(RColor::LightRed).add_modifier(Modifier::BOLD));
    let placeholder = if app.input.is_empty() {
        Span::styled("Type a message...", Style::default().fg(RColor::DarkGray))
    } else {
        Span::raw("")
    };
    let input_text = Span::styled(&app.input, Style::default().fg(RColor::White));

    let mut line_spans = vec![prefix];
    if app.input.is_empty() {
        line_spans.push(placeholder);
    } else {
        line_spans.push(input_text);
    }

    let input_paragraph = Paragraph::new(Line::from(line_spans))
        .block(input_block);
        
    f.render_widget(input_paragraph, chunks[2]);

    // Update cursor position inside the input block
    let cursor_x_offset = app.cursor as u16; 
    let cursor_x = chunks[2].x + 4 + cursor_x_offset.min(chunks[2].width.saturating_sub(6));
    let cursor_y = chunks[2].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    // 4. HELP PANEL MODAL
    render_help_modal(f, app, inner_chat_area);
}

fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    let chat_block = Block::default().borders(Borders::LEFT | Borders::RIGHT).border_style(Style::default().fg(RColor::DarkGray));
    f.render_widget(chat_block, area);

    let inner_area = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: area.height,
    };

    if inner_area.height == 0 || inner_area.width == 0 {
        return;
    }

    // Build the rendered bubbles
    let mut rendered_lines = Vec::new();

    for msg in &app.messages {
        let age_ms = msg.created.elapsed().as_millis() as f32;
        let progress = (age_ms / 180.0).clamp(0.0, 1.0);
        let slide_offset = if progress >= 1.0 {
            0
        } else {
            ((1.0 - progress) * 6.0) as u16
        };

        // Determine max bubble width
        let max_bubble_w = (inner_area.width * 70) / 100;
        
        let words = msg.text.split_whitespace();
        let mut text_lines = Vec::new();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.len() + word.len() + 1 > max_bubble_w as usize {
                if !current_line.is_empty() {
                    text_lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        if !current_line.is_empty() {
            text_lines.push(current_line);
        }
        if text_lines.is_empty() {
            text_lines.push(String::new());
        }

        let bubble_width = text_lines.iter().map(|l| l.len()).max().unwrap_or(0).max(1) as u16 + 4;
        
        let color = if msg.is_mine { RColor::Red } else { RColor::LightRed };
        let align = if msg.is_mine { Alignment::Right } else { Alignment::Left };

        // Add bubble padding/margin
        rendered_lines.push((String::new(), RColor::Reset, Alignment::Left, 0)); // Spacing between bubbles
        
        // Top border
        let t_border = format!("╭{}╮", "─".repeat(bubble_width.saturating_sub(2) as usize));
        rendered_lines.push((t_border, color, align, slide_offset));
        
        for line in text_lines {
            let space_padding = bubble_width.saturating_sub(line.len() as u16 + 2) as usize;
            let content = format!("│ {}{}│", line, " ".repeat(space_padding));
            rendered_lines.push((content, color, align, slide_offset));
        }

        // Bottom border
        let b_border = format!("╰{}╯", "─".repeat(bubble_width.saturating_sub(2) as usize));
        rendered_lines.push((b_border, color, align, slide_offset));
    }

    let total_lines = rendered_lines.len();
    let viewport_h = inner_area.height as usize;
    let max_scroll = total_lines.saturating_sub(viewport_h);
    let scroll = app.scroll.min(max_scroll);
    
    let view_start = max_scroll.saturating_sub(scroll);
    let view_end = view_start + viewport_h;

    // Draw lines
    for (i, (line, color, align, slide_offset)) in rendered_lines.into_iter().enumerate() {
        if i < view_start || i >= view_end {
            continue;
        }

        let y = inner_area.y + (i - view_start) as u16;
        
        let x_slide = match align {
            Alignment::Left => slide_offset,
            Alignment::Right => {
                let pad = inner_area.width.saturating_sub(line.chars().count() as u16);
                pad.saturating_add(slide_offset)
            },
            _ => 0,
        };

        let final_x = inner_area.x + x_slide;
        let line_area = Rect {
            x: final_x,
            y,
            width: inner_area.width.saturating_sub(x_slide),
            height: 1,
        };

        let span = Span::styled(line, Style::default().fg(color));
        let p = Paragraph::new(span);
        f.render_widget(p, line_area);
    }
}

fn render_help_modal(f: &mut Frame, app: &App, chat_area: Rect) {
    if !app.help_visible() && help_panel_progress(app) >= 1.0 {
        return;
    }

    let lines = help_panel_lines();
    let target_h = lines.len() as u16 + 2;
    let p = help_panel_progress(app);
    
    let visible_h = if app.help_visible() {
        (target_h as f32 * p).ceil() as u16
    } else {
        (target_h as f32 * (1.0 - p)).ceil() as u16
    }.min(target_h);

    if visible_h < 2 {
        return;
    }

    // Modal size
    let modal_width = 30.min(chat_area.width.saturating_sub(4));
    let modal_height = visible_h;
    
    // Bottom aligned right above the input
    let area = Rect {
        x: chat_area.x + (chat_area.width.saturating_sub(modal_width)) / 2,
        y: chat_area.y + chat_area.height.saturating_sub(modal_height),
        width: modal_width,
        height: modal_height,
    };

    f.render_widget(Clear, area); // Clear background to prevent text overlap

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(RColor::Magenta));

    let mut text = Vec::new();
    let max_lines = visible_h.saturating_sub(2) as usize;
    for (i, line) in lines.iter().take(max_lines).enumerate() {
        if i == 0 {
            text.push(Line::from(Span::styled(*line, Style::default().fg(RColor::LightRed).add_modifier(Modifier::BOLD))));
        } else {
            text.push(Line::from(Span::styled(*line, Style::default().fg(RColor::Yellow))));
        }
    }

    let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
    f.render_widget(p, area);
}
