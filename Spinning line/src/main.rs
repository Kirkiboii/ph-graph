use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{CrosstermBackend, Stylize, Terminal};
use ratatui::widgets::{Block, Borders, Paragraph};

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
    let mut angle = 0.0;

    loop {
        terminal.draw(|f| ui(f, angle))?;

        let elapsed = last_frame_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        last_frame_time = Instant::now();
        angle += 0.1;

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, angle: f64) {
    let size = f.size();
    let block = Block::default().borders(Borders::ALL).title("Spinning Line");
    f.render_widget(block, size);

    let line = create_line(angle, size.width as f64, size.height as f64);
    let paragraph = Paragraph::new(line).white();
    f.render_widget(paragraph, size);
}

fn create_line(angle: f64, width: f64, height: f64) -> String {
    let mut buffer = vec![vec![' '; width as usize]; height as usize];
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let length = (width / 6.0) * 0.8;

    for i in 0..=(length as i32) {
        let x = center_x + 2.0 * (i as f64) * angle.cos();
        let y = center_y + (i as f64) * angle.sin();
        if x >= 0.0 && x < width && y >= 0.0 && y < height {
            buffer[y as usize][x as usize] = '#';
        }

        let x = center_x - 2.0 * (i as f64) * angle.cos();
        let y = center_y - (i as f64) * angle.sin();
        if x >= 0.0 && x < width && y >= 0.0 && y < height {
            buffer[y as usize][x as usize] = '#';
        }
    }

    buffer.into_iter().map(|row| row.into_iter().collect()).collect::<Vec<String>>().join("\n")
}