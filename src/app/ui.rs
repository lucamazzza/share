use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;
use crate::app::Chat;

pub fn draw<B>(rect: &mut Frame<B>, _chat: &Chat) where B: Backend {
    let size = rect.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3)].as_ref())
        .split(size);
    let title = draw_title();
    rect.render_widget(title, chunks[0]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Share")
        .style(Style::default().fg(Color::LightGreen))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Gray))
                .border_type(BorderType::Rounded),
        )
}

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        panic!("Terminal too small. Min width is 52");
    }
    if rect.height < 28 {
        panic!("Terminal too small. Min height is 28");
    }
}