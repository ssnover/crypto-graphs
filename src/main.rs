#[allow(dead_code)]
mod util;

use crate::util::{
    event::{Event, Events},
    SinSignal,
};
use reqwest::blocking::Client;
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Terminal,
};

struct App {
    signal1: SinSignal,
    data1: Vec<(f64, f64)>,
    signal2: SinSignal,
    data2: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl App {
    fn new() -> App {
        let mut signal1 = SinSignal::new(0.2, 3.0, 18.0);
        let mut signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let data1 = signal1.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        let data2 = signal2.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        App {
            signal1,
            data1,
            signal2,
            data2,
            window: [0.0, 20.0],
        }
    }

    fn update(&mut self) {
        for _ in 0..5 {
            self.data1.remove(0);
        }
        self.data1.extend(self.signal1.by_ref().take(5));
        for _ in 0..10 {
            self.data2.remove(0);
        }
        self.data2.extend(self.signal2.by_ref().take(10));
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    let query_url = "https://api.coingecko.com/api/v3/coins/helium/market_chart?vs_currency=usd&days=3".to_string();
    let client = Client::new();
    let response = client
        .get(&query_url)
        .header("accept", "application/json")
        .send()
        .unwrap()
        .text()
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response)?;
    let data = response_json["prices"].as_array().unwrap();
    let mut hnt_data: Vec<(f64, f64)> = vec![];
    for val in data {
        let secs = val.as_array().unwrap()[0].as_f64().unwrap() / 1000.;
        let value_in_usd = val.as_array().unwrap()[1].as_f64().unwrap();
        hnt_data.push((secs, value_in_usd));
    }
    let hnt_data = hnt_data;
    let mut min_time = hnt_data[0].0;
    let mut min_usd = hnt_data[0].1;
    let mut max_time = min_time;
    let mut max_usd = min_usd;

    for datum in &hnt_data {
        if datum.0 < min_time {
            min_time = datum.0;
        }
        if datum.0 > max_time {
            max_time = datum.0;
        }
        if datum.1 < min_usd {
            min_usd = datum.1;
        }
        if datum.1 > max_usd {
            max_usd = datum.1;
        }
    }

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    // App
    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Ratio(1, 2),
                        Constraint::Ratio(1, 2),
                    ]
                    .as_ref(),
                )
                .split(size);

            let datasets = vec![Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::LightBlue))
                .graph_type(GraphType::Line)
                .data(&hnt_data)];
            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(Span::styled(
                            "HNT vs USD",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("USD")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([min_time, max_time])
                        .labels(vec![
                            Span::styled("-3d", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("-1.5d"),
                            Span::styled("Now", Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("HNT")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([min_usd, max_usd])
                        .labels(vec![
                            Span::styled(format!("{:.2}", min_usd), Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(format!("{:.2}", (min_usd + max_usd) / 2.)),
                            Span::styled(format!("{:.2}", max_usd), Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                );
            f.render_widget(chart, chunks[0]);

            
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
