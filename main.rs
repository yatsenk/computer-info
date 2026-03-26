use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Line};
use ratatui::widgets::{Block, Tabs};
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))
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

#[derive(Debug, Default)]
struct App<'a> {
    title: &'a str,
    tabs: TabsState<'a>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            title: "TITLE",
            tabs: TabsState::new(vec!["Tab0", "Tab1", "Tab2"]),
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

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
            .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
            .collect::<Tabs>()
            .block(Block::bordered().title(self.title))
            .highlight_style(Style::default().fg(Color::Yellow))
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
}
