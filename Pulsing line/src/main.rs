use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui::widgets::{Block, Borders, Paragraph};

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
            self.pulse_position += 0.05; // Adjust for speed
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
    f.render_widget(block, size);

    let line_width = (size.width as f64 * 0.8) as usize;
    let line_y = size.height / 2;

    let mut line = vec![' '; size.width as usize];
    let line_start = (size.width as usize - line_width) / 2;

    for i in 0..line_width {
        line[line_start + i] = '─';
    }

    if app.pulse_active {
        let pulse_x = line_start + (app.pulse_position * line_width as f64) as usize;
        if pulse_x < size.width as usize {
            // Improved pulse effect
            let pulse_width = 4;
            for i in 0..pulse_width {
                let pos = pulse_x + i;
                if pos < size.width as usize {
                    let intensity = 1.0 - ((app.pulse_position * line_width as f64) % 1.0);
                    let character = match intensity {
                        x if x > 0.75 => '█',
                        x if x > 0.5 => '▓',
                        x if x > 0.25 => '▒',
                        _ => '░',
                    };
                    line[pos] = character;
                }
            }
        }
    }

    f.render_widget(Paragraph::new(line.into_iter().collect::<String>()), ratatui::layout::Rect::new(0, line_y, size.width, 1));
}