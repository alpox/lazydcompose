use std::cmp::min;

use ratatui::widgets::TableState;

pub trait TableStateExt {
    fn fit(&mut self, table_count: usize);
}

impl TableStateExt for TableState {
    fn fit(&mut self, table_count: usize) {
        self.select(match (self.selected(), table_count) {
            (Some(idx), pl) => Some(min(idx, pl.saturating_sub(1))),
            (_, 0) => None,
            _ => Some(0),
        });
    }
}
