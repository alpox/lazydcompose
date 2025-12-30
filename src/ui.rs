use ratatui::{
    buffer::Buffer, layout::{Alignment, Rect}, style::{Color}, text::Line, widgets::{Block, BorderType, HighlightSpacing, List, ListItem, Widget}
};

use crate::{cli::Project, model::Model};

impl From<&Project> for ListItem<'_> {
    fn from(value: &Project) -> Self {
        ListItem::new(Line::styled(format!("Project: {}", value.name), Color::Cyan))
    }
}

impl Widget for &Model {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("lazydcompose")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let items = self.projects.iter().map(|project| {
            ListItem::from(project)
        });

        let list = List::new(items)
            .block(block)
            .highlight_style(Color::Blue)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // let text = format!(
        //     "This is a tui template.\n\
        //         Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
        //         Press left and right to increment and decrement the counter respectively.\n\
        //         Counter: {}",
        //     self.counter
        // );

        // let paragraph = Paragraph::new(text)
        //     .block(block)
        //     .fg(Color::Cyan)
        //     .bg(Color::Black)
        //     .centered();

        list.render(area, buf);
    }
}
