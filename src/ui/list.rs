use std::cmp::min;

use ratatui::widgets::ListState;

pub trait ListStateExt {
    fn fit(&mut self, list_count: usize);
}

impl ListStateExt for ListState {
    fn fit(&mut self, list_count: usize) {
        self.select(match (self.selected(), list_count) {
            (Some(idx), pl) => Some(min(idx, pl.saturating_sub(1))),
            (_, 0) => None,
            _ => Some(0),
        });
    }
}
