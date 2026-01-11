use ratatui::style::Color;

use crate::cli::{Container, Project, State};

pub trait Colorize {
    fn colorize(&self) -> Color;
}

impl Colorize for State {
    fn colorize(&self) -> Color {
        match self {
            State::Running => Color::Green,
            State::Created => Color::Cyan,
            State::Paused => Color::Yellow,
            State::Restarting => Color::Yellow,
            State::Exited => Color::White,
            State::Removing => Color::Magenta,
            State::Dead => Color::Red,
        }
    }
}

impl Colorize for Project {
    fn colorize(&self) -> Color {
        // Priority: Dead > Exited > Removing > Restarting > Paused > Created > Running
        // let dominated_state = self.state.iter().map(|s| &s.state).max_by_key(|s| match s {
        //     State::Dead => 7,
        //     State::Exited => 6,
        //     State::Removing => 5,
        //     State::Restarting => 4,
        //     State::Paused => 3,
        //     State::Created => 2,
        //     State::Running => 1,
        // });
        //
        // dominated_state
        //     .map(|s| s.colorize())
        //     .unwrap_or(Color::DarkGray)
        //
        Color::Cyan
    }
}

impl Colorize for Container {
    fn colorize(&self) -> Color {
        self.state.colorize()
    }
}
