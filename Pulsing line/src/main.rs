use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::f64::consts::PI;

struct App {
    last_pulse_time: Instant,
    pulse_position: f64,
    pulse_active: bool,
}

impl App {
    fn new() -> Self {
        Self {
            last_pulse_time: Instant::now(),
            pulse_position: 0.0,
            pulse_active: false,
        }
    }

    fn on_tick(&mut self) {
        if !self.pulse_active && self.last_pulse_time.elapsed() >= Duration::from_secs_f64(1.5) {
            self.pulse_active = true;
            self.pulse_position = 0.0;
            self.last_pulse_time = Instant::now();
        }

        if self.pulse_active {
            self.pulse_position += 0.025; // Adjust for speed (twice as slow)
            if self.pulse_position >= 1.0 {
                self.pulse_active = false;
            }
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    loop {
        let start_time = Instant::now();

        terminal.draw(|f| ui(f, app))?;

        app.on_tick();

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        let elapsed = start_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();
    let block = Block::default().borders(Borders::ALL).title("Flowing Pulse");
    let inner_rect = block.inner(size); // Get the inner area of the block
    f.render_widget(block, size);

    let line_width = (inner_rect.width as f64 * 0.8) as usize; // Use inner_rect.width
    let line_y = inner_rect.y + inner_rect.height / 2; // Position relative to inner_rect.y

    let mut line = vec![' '; inner_rect.width as usize]; // Use inner_rect.width for line buffer
    let line_start = (inner_rect.width as usize - line_width) / 2; // Relative to inner_rect

    for i in 0..line_width {
        line[line_start + i] = '=';
    }

    if app.pulse_active {
        let pulse_center_x = line_start + (app.pulse_position * line_width as f64) as usize;
        let max_blob_width = 10; // Maximum width of the blob
        let current_blob_width = (app.pulse_position * PI).sin() * max_blob_width as f64;
        let current_blob_width = current_blob_width.max(1.0); // Ensure minimum width of 1

        let start_blob = (pulse_center_x as f64 - current_blob_width / 2.0).floor() as usize;
        let end_blob = (pulse_center_x as f64 + current_blob_width / 2.0).ceil() as usize;

        for pos in start_blob..end_blob {
            if pos >= line_start && pos < line_start + line_width { // Ensure within the main line
                let distance_from_center = (pos as f64 - pulse_center_x as f64).abs();
                let normalized_distance = distance_from_center / (current_blob_width / 2.0);
                let local_intensity = 1.0 - normalized_distance.powf(2.0); // Parabolic falloff

                let overall_pulse_intensity = (app.pulse_position * PI).sin();
                let _character_intensity = overall_pulse_intensity * local_intensity;

                let character = '#';
                line[pos] = character;
            }
        }
    }

    f.render_widget(Paragraph::new(line.into_iter().collect::<String>()), ratatui::layout::Rect::new(inner_rect.x, line_y, inner_rect.width, 1));
}