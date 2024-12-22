use crate::{
    args::{Args, ClockStyle, Content},
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler, Events},
    storage::AppStorage,
    terminal::Terminal,
    widgets::{
        clock::{self, Clock, ClockArgs},
        countdown::{Countdown, CountdownWidget},
        footer::Footer,
        header::Header,
        pomodoro::{Mode as PomodoroMode, Pomodoro, PomodoroArgs, PomodoroWidget},
        timer::{Timer, TimerWidget},
    },
};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use std::time::Duration;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quit,
}

#[derive(Debug)]
pub struct App {
    content: Content,
    mode: Mode,
    show_menu: bool,
    countdown: Countdown,
    timer: Timer,
    pomodoro: Pomodoro,
    clock_style: ClockStyle,
    with_decis: bool,
}

pub struct AppArgs {
    pub style: ClockStyle,
    pub with_decis: bool,
    pub content: Content,
    pub pomodoro_mode: PomodoroMode,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub initial_value_pause: Duration,
    pub current_value_pause: Duration,
    pub initial_value_countdown: Duration,
    pub current_value_countdown: Duration,
    pub current_value_timer: Duration,
}

/// Getting `AppArgs` by merging `Args` and `AppStorage`.
/// `Args` wins btw.
impl From<(Args, AppStorage)> for AppArgs {
    fn from((args, stg): (Args, AppStorage)) -> Self {
        AppArgs {
            with_decis: args.decis || stg.with_decis,
            content: args.mode.unwrap_or(stg.content),
            style: args.style.unwrap_or(stg.clock_style),
            pomodoro_mode: stg.pomodoro_mode,
            initial_value_work: args.work.unwrap_or(stg.inital_value_work),
            current_value_work: stg.current_value_work,
            initial_value_pause: args.pause,
            current_value_pause: stg.current_value_pause,
            initial_value_countdown: args.countdown,
            current_value_countdown: stg.current_value_countdown,
            current_value_timer: stg.current_value_timer,
        }
    }
}

impl App {
    pub fn new(args: AppArgs) -> Self {
        let AppArgs {
            style,
            initial_value_work,
            initial_value_pause,
            initial_value_countdown,
            current_value_work,
            current_value_pause,
            current_value_countdown,
            current_value_timer,
            content,
            with_decis,
            pomodoro_mode,
        } = args;
        Self {
            mode: Mode::Running,
            content,
            show_menu: false,
            clock_style: style,
            with_decis,
            countdown: Countdown::new(Clock::<clock::Countdown>::new(ClockArgs {
                initial_value: initial_value_countdown,
                current_value: current_value_countdown,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                style,
                with_decis,
            })),
            timer: Timer::new(Clock::<clock::Timer>::new(ClockArgs {
                initial_value: Duration::ZERO,
                current_value: current_value_timer,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                style,
                with_decis,
            })),
            pomodoro: Pomodoro::new(PomodoroArgs {
                mode: pomodoro_mode,
                initial_value_work,
                current_value_work,
                initial_value_pause,
                current_value_pause,
                style,
                with_decis,
            }),
        }
    }

    pub async fn run(mut self, mut terminal: Terminal, mut events: Events) -> Result<Self> {
        while self.is_running() {
            if let Some(event) = events.next().await {
                // Pipe events into subviews and handle only 'unhandled' events afterwards
                if let Some(unhandled) = match self.content {
                    Content::Countdown => self.countdown.update(event.clone()),
                    Content::Timer => self.timer.update(event.clone()),
                    Content::Pomodoro => self.pomodoro.update(event.clone()),
                } {
                    match unhandled {
                        Event::Render | Event::Resize => {
                            self.draw(&mut terminal)?;
                        }
                        Event::Key(key) => self.handle_key_event(key),
                        _ => {}
                    }
                }
            }
        }
        Ok(self)
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        debug!("Received key {:?}", key.code);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('c') => self.content = Content::Countdown,
            KeyCode::Char('t') => self.content = Content::Timer,
            KeyCode::Char('p') => self.content = Content::Pomodoro,
            KeyCode::Char('m') => self.show_menu = !self.show_menu,
            KeyCode::Char(',') => {
                self.clock_style = self.clock_style.next();
                // update clocks
                self.timer.set_style(self.clock_style);
                self.countdown.set_style(self.clock_style);
                self.pomodoro.set_style(self.clock_style);
            }
            KeyCode::Char('.') => {
                self.with_decis = !self.with_decis;
                // update clocks
                self.timer.set_with_decis(self.with_decis);
                self.countdown.set_with_decis(self.with_decis);
                self.pomodoro.set_with_decis(self.with_decis);
            }
            KeyCode::Up => self.show_menu = true,
            KeyCode::Down => self.show_menu = false,
            _ => {}
        };
    }

    fn draw(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), self);
        })?;
        Ok(())
    }

    pub fn to_storage(&self) -> AppStorage {
        AppStorage {
            content: self.content,
            show_menu: self.show_menu,
            clock_style: self.clock_style,
            with_decis: self.with_decis,
            pomodoro_mode: self.pomodoro.get_mode().clone(),
            inital_value_work: self.pomodoro.get_clock_work().initial_value,
            current_value_work: self.pomodoro.get_clock_work().current_value,
            inital_value_pause: self.pomodoro.get_clock_pause().initial_value,
            current_value_pause: self.pomodoro.get_clock_pause().current_value,
            inital_value_countdown: self.countdown.clock.initial_value,
            current_value_countdown: self.countdown.clock.current_value,
            current_value_timer: self.timer.clock.current_value,
        }
    }
}

struct AppWidget;

impl AppWidget {
    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut App) {
        match state.content {
            Content::Timer => TimerWidget.render(area, buf, &mut state.timer.clone()),
            Content::Countdown => CountdownWidget.render(area, buf, &mut state.countdown.clone()),
            Content::Pomodoro => PomodoroWidget.render(area, buf, &mut state.pomodoro.clone()),
        };
    }
}

impl StatefulWidget for AppWidget {
    type State = App;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(if state.show_menu { 2 } else { 1 }),
        ]);
        let [v0, v1, v2] = vertical.areas(area);

        Header::new(true).render(v0, buf);
        self.render_content(v1, buf, state);
        Footer::new(state.show_menu, state.content).render(v2, buf);
    }
}
