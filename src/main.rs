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

struct MarketData {
    pub price_data: Vec<(f64, f64)>,
    pub min_time: f64,
    pub max_time: f64,
    pub min_usd: f64,
    pub max_usd: f64,
}
    

fn retrieve_market_data(token: String) -> Result<MarketData, Box<dyn Error>> {
    let query_url = format!("https://api.coingecko.com/api/v3/coins/{}/market_chart?vs_currency=usd&days=3", token);
    let response = Client::new()
        .get(&query_url)
        .header("accept", "application/json")
        .send()
        .unwrap()
        .text()
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response)?;
    let data = response_json["prices"].as_array().unwrap();
    let mut market_price_data: Vec<(f64, f64)> = vec![];
    for val in data {
        let secs = val.as_array().unwrap()[0].as_f64().unwrap() / 1000.;
        let value_in_usd = val.as_array().unwrap()[1].as_f64().unwrap();
        market_price_data.push((secs, value_in_usd));
    }
    let market_price_data = market_price_data;

    let mut min_time = market_price_data[0].0;
    let mut min_usd = market_price_data[0].1;
    let mut max_time = min_time;
    let mut max_usd = min_usd;

    for elem in &market_price_data {
        if elem.0 < min_time {
            min_time = elem.0;
        }
        if elem.0 > max_time {
            max_time = elem.0;
        }
        if elem.1 < min_usd {
            min_usd = elem.1;
        }
        if elem.1 > max_usd {
            max_usd = elem.1;
        }
    }

    Ok( MarketData { price_data: market_price_data,
        min_time: min_time,
        max_time: max_time,
        min_usd: min_usd,
        max_usd: max_usd })
}

fn main() -> Result<(), Box<dyn Error>> {

    let hnt_data = retrieve_market_data("helium".to_string()).unwrap();
    let ada_data = retrieve_market_data("cardano".to_string()).unwrap();

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
                .data(&hnt_data.price_data)];
            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(Span::styled(
                            format!("HNT vs USD, Current Price: {:.4}", hnt_data.price_data[hnt_data.price_data.len()-1].1),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .style(Style::default().fg(Color::Gray))
                        .bounds([hnt_data.min_time, hnt_data.max_time])
                        .labels(vec![
                            Span::styled("-3d", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("-1.5d"),
                            Span::styled("Now", Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("USD")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([hnt_data.min_usd, hnt_data.max_usd])
                        .labels(vec![
                            Span::styled(format!("{:.2}", hnt_data.min_usd), Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(format!("{:.2}", (hnt_data.min_usd + hnt_data.max_usd) / 2.)),
                            Span::styled(format!("{:.2}", hnt_data.max_usd), Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                );
            f.render_widget(chart, chunks[0]);

            let datasets = vec![Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Red))
                .graph_type(GraphType::Line)
                .data(&ada_data.price_data)];
            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(Span::styled(
                            format!("ADA vs USD, Current Price: {:.4}", ada_data.price_data[ada_data.price_data.len()-1].1),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .style(Style::default().fg(Color::Gray))
                        .bounds([ada_data.min_time, ada_data.max_time])
                        .labels(vec![
                            Span::styled("-3d", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("-1.5d"),
                            Span::styled("Now", Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("USD")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([ada_data.min_usd, ada_data.max_usd])
                        .labels(vec![
                            Span::styled(format!("{:.2}", ada_data.min_usd), Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(format!("{:.2}", (ada_data.min_usd + ada_data.max_usd) / 2.)),
                            Span::styled(format!("{:.2}", ada_data.max_usd), Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                );
            f.render_widget(chart, chunks[1]);

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