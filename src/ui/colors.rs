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
        let running = self
            .containers
            .iter()
            .filter(|container| container.state == State::Running)
            .count();

        match running {
            r if r == self.containers.len() => Color::Green,
            r if r > 0 => Color::Yellow,
            _ => Color::White
        }
    }
}

impl Colorize for Container {
    fn colorize(&self) -> Color {
        self.state.colorize()
    }
}
