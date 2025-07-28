use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{CrosstermBackend, Stylize, Terminal};
use ratatui::widgets::{Block, Borders, Paragraph};
use rand::Rng;

fn main() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut last_frame_time = Instant::now();
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut rng = rand::thread_rng();
    let mut angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
    let mut spinning_active = false; // New state variable

    loop {
        terminal.draw(|f| ui(f, angle))?;

        let elapsed = last_frame_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        last_frame_time = Instant::now();

        if spinning_active {
            angle += 0.05; // Adjust speed as needed
        }

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('s') | KeyCode::Char('S') => spinning_active = !spinning_active,
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, angle: f64) {
    let size = f.size();

    // Create a layout to split the screen into two parts: main content and keybinds
    let chunks = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints([
            ratatui::prelude::Constraint::Min(0),
            ratatui::prelude::Constraint::Length(1),
        ])
        .split(size);

    // Main content area
    let main_block = Block::default().borders(Borders::ALL).title("Pulsing Line");
    f.render_widget(main_block, chunks[0]);

    let line = create_line(angle, chunks[0].width as f64, chunks[0].height as f64);
    let line_paragraph = Paragraph::new(line).white();
    f.render_widget(line_paragraph, chunks[0]);

    // Keybinds message at the bottom
    let keybinds_message = Paragraph::new("Quit: q | Toggle Spin: s").white();
    f.render_widget(keybinds_message, chunks[1]);
}

fn create_line(angle: f64, width: f64, height: f64) -> String {
    let mut buffer = vec![vec![' '; width as usize]; height as usize];
    let center_x = width / 2.0;
    let center_y = height / 2.0;

    // Calculate radius: diameter is half of terminal width
    let radius = (width / 2.0) / 4.0;

    // Calculate endpoints of the line on the circle
    // Adjust for terminal character aspect ratio (typically 2:1 width:height)
    let aspect_ratio_correction = 2.0;

    let x1_f = center_x + radius * angle.cos() * aspect_ratio_correction;
    let y1_f = center_y + radius * angle.sin();
    let x2_f = center_x - radius * angle.cos() * aspect_ratio_correction;
    let y2_f = center_y - radius * angle.sin();

    // Round to nearest integer for drawing
    let x1 = x1_f.round() as isize;
    let y1 = y1_f.round() as isize;
    let x2 = x2_f.round() as isize;
    let y2 = y2_f.round() as isize;

    // Draw the line using Bresenham's line algorithm
    // This is a more standard implementation
    let mut x = x1;
    let mut y = y1;

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();

    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };

    let mut err = dx - dy;

    loop {
        if x >= 0 && x < width as isize && y >= 0 && y < height as isize {
            buffer[y as usize][x as usize] = '#';
        }

        if x == x2 && y == y2 { break; }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    buffer.into_iter().map(|row| row.into_iter().collect()).collect::<Vec<String>>().join("\n")
}