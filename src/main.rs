use sysinfo::System;
use rand::distr::{Distribution, Uniform};
use rand::rngs::ThreadRng;

use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Color, Style};
use ratatui::text::{Span, Line};
use ratatui::widgets::{Dataset, Axis, Block, Tabs, Chart};
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;
    let tick_rate = std::time::Duration::from_millis(250);
    ratatui::run(|terminal| App::new().run(terminal, tick_rate))
}

#[derive(Clone, Debug)]
struct RandomSignal {
    distribution: Uniform<u64>,
    rng: ThreadRng,
}

impl RandomSignal {
    fn new(lower: u64, upper: u64) -> Self {
        Self {
            distribution: Uniform::new(lower, upper).expect("invalid range"),
            rng: rand::rng(),
        }
    }
}

impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

#[derive(Clone, Debug)]
struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    const fn new(interval: f64, period: f64, scale: f64) -> Self {
        Self {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

#[derive(Debug)]
struct Signal<S: Iterator> {
    source: S,
    points: Vec<S::Item>,
    tick_rate: usize,
}

impl<S> Signal<S>
where
    S: Iterator,
{
    fn on_tick(&mut self) {
        self.points.drain(0..self.tick_rate);
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

#[derive(Debug)]
struct Signals {
    sin: Signal<SinSignal>,
    window: [f64; 2],
}

impl Signals {
    fn on_tick(&mut self) {
        self.sin.on_tick();
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

#[derive(Debug, Default)]
struct TabsState<'a> {
    titles: Vec<&'a str>,
    index: usize,
}

impl<'a> TabsState<'a> {
    const fn new(titles: Vec<&'a str>) -> Self {
        Self { titles, index: 0 }
    }

    fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

#[derive(Debug)]
struct App<'a> {
    title: &'a str,
    tabs: TabsState<'a>,
    sparkline: Signal<RandomSignal>,
    signals: Signals,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let mut rand_signal = RandomSignal::new(0, 100);
        let sparkline_points = rand_signal.by_ref().take(300).collect();
        let mut sin_signal = SinSignal::new(0.2, 3.0, 18.0);
        let sin1_points = sin_signal.by_ref().take(100).collect();
        Self {
            title: "TITLE",
            tabs: TabsState::new(vec!["Perfomance", "Info", "Other"]),
            sparkline: Signal {
                source: rand_signal,
                points: sparkline_points,
                tick_rate: 1,
            },
            signals: Signals {
                sin: Signal {
                    source: sin_signal,
                    points: sin1_points,
                    tick_rate: 5,
                },
                window: [0.0, 20.0],
            },
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    fn run(mut self, terminal: &mut DefaultTerminal, tick_rate: std::time::Duration) -> Result<()> {
        let mut last_tick = std::time::Instant::now();
        loop {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if !event::poll(timeout)? {
                self.on_tick();
                last_tick = std::time::Instant::now();
                continue;
            }

            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Tab => self.on_right(),
                    KeyCode::BackTab => self.on_left(),
                    _ => {}   
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) { 
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());
        let tabs = self
            .tabs
            .titles
            .iter()
            .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White))))
            .collect::<Tabs>()
            .block(Block::bordered().style(Color::Black).title(self.title))
            .highlight_style(Style::default().fg(Color::LightBlue))
            .select(self.tabs.index);
        frame.render_widget(tabs, chunks[0]);
        match self.tabs.index {
            0 => self.render_first_tab(frame, chunks[1]),
            1 => self.render_second_tab(frame, chunks[1]),
            _ => {}
        };
    }

    fn render_first_tab(&self, frame: &mut Frame, area: Rect) {
        let [h1, h2] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(area);

        let [cpu_usage, gpu_usage] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(h1);

        let [disk_usage, wifi_usage] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(h2);

        self.draw_chart(frame, cpu_usage);

        let gpu = Block::bordered()
            .style(Style::new().yellow().on_black().bold())
            .title("GPU usage");
        frame.render_widget(gpu, gpu_usage);

        let disk = Block::bordered()
            .style(Style::new().green().on_black().bold())
            .title("Disk usage");
        frame.render_widget(disk, disk_usage);
        
        let wifi = Block::bordered()
            .style(Style::new().blue().on_black().bold())
            .title("Wi-fi usage");
        frame.render_widget(wifi, wifi_usage);
    }

    fn render_second_tab(&self, frame: &mut Frame, area: Rect) {
        let [h1, h2] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(area);

        let [cpu_usage, gpu_usage] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(h1);

        let [disk_usage, wifi_usage] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(h2);

        let cpu = Block::bordered()
            .style(Style::new().red().on_black().bold())
            .title("CPU usage");
        frame.render_widget(cpu, cpu_usage);

        let gpu = Block::bordered()
            .style(Style::new().yellow().on_black().bold())
            .title("GPU usage");
        frame.render_widget(gpu, gpu_usage);

        let disk = Block::bordered()
            .style(Style::new().green().on_black().bold())
            .title("Disk usage");
        frame.render_widget(disk, disk_usage);
        
        let wifi = Block::bordered()
            .style(Style::new().blue().on_black().bold())
            .title("Wi-fi usage");
        frame.render_widget(wifi, wifi_usage);
    }

    fn draw_chart(&self, frame: &mut Frame, area: Rect) {
        let x_labels = vec![
            Span::styled(
                format!("{}", self.signals.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{}",
                (self.signals.window[0] + self.signals.window[1]) / 2.0
            )),
            Span::styled(
                format!("{}", self.signals.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![
            Dataset::default()
                .name("data2")
                .marker(ratatui::symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.signals.sin.points),
        ];
        let chart = Chart::new(datasets)
            .block(
                Block::bordered().title(Span::styled(
                    "Chart",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
            )
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.signals.window)
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 100.0])
                    .labels([
                        Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("50"),
                        Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
            );
        frame.render_widget(chart, area);
    }

    fn on_tick(&mut self) {
        self.sparkline.on_tick();
        self.signals.on_tick();
    }
}
