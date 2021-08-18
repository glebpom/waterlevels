use std::{error::Error, io};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    Terminal,
    widgets::{BarChart, Block, Borders},
};

use snapview_test_lib::Model;

use crate::util::event::{Event, Events};

mod util;

struct App {
    time: f64,
    model: Model,
    current: Vec<(&'static str, u64)>,
}

impl App {
    fn new() -> App {
        // let v = (0..20).map(|_| thread_rng().gen_range(0.0..20.0f64)).collect::<Vec<f64>>();
        let v = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut app = App {
            time: 0.0,
            model: Model::new(&v, 1.0).unwrap(),
            current: vec![],
        };

        app.update();

        app
    }

    fn update(&mut self) {
        if self.time >= 30.0 {
            return;
        }
        self.current = self.model.calculate_levels(self.time).unwrap()
            .into_iter()
            .map(move |height| ("", (height * 100.0) as u64))
            .collect();

        self.time += 0.025;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let events = Events::new();

    // App
    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(100), Constraint::Percentage(100)].as_ref())
                .split(f.size());
            let barchart = BarChart::default()
                .block(Block::default().title("Water Levels").borders(Borders::ALL))
                .data(&app.current)
                .bar_width(9)
                .bar_style(Style::default().fg(Color::Blue))
                .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
            f.render_widget(barchart, chunks[0]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                }
            }
            Event::Tick => {
                app.update();
            }
        }
    }

    Ok(())
}
