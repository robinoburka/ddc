use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) fn popup_area_clamped(
    area: Rect,
    min_width: u16,
    max_width: u16,
    width_percent: u16,
    min_height: u16,
    max_height: u16,
    height_percent: u16,
) -> Rect {
    fn clamp_percent(total: u16, percent: u16, min: u16, max: u16) -> u16 {
        if total == 0 {
            return 0;
        }

        let percent_size = total.saturating_mul(percent) / 100;
        let clamped = percent_size.clamp(min, max);
        clamped.min(total)
    }

    let width = clamp_percent(area.width, width_percent, min_width, max_width);
    let height = clamp_percent(area.height, height_percent, min_height, max_height);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1]);

    horizontal[1]
}
