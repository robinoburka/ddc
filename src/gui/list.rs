//! Pieces shared by every selectable, scrollable list in the GUI.

use iced::widget::{Id, container, mouse_area, operation, scrollable};
use iced::{Element, Length, Task, Theme};

pub const ROW_HEIGHT: f32 = 28.0;

/// Used until the first `on_scroll` event tells us the real viewport size.
const FALLBACK_VIEWPORT: f32 = 400.0;

#[derive(Debug, Clone, Copy)]
pub enum Nav {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Computes the new selection for a keyboard navigation request.
pub fn navigate(nav: Nav, selected: usize, len: usize, page: usize) -> usize {
    let last = len.saturating_sub(1);
    match nav {
        Nav::Up => selected.saturating_sub(1),
        Nav::Down => (selected + 1).min(last),
        Nav::PageUp => selected.saturating_sub(page),
        Nav::PageDown => (selected + page).min(last),
        Nav::Home => 0,
        Nav::End => last,
    }
}

/// Tracks where a `scrollable` currently is, so we can keep a selection visible.
#[derive(Debug)]
pub struct ListScroll {
    id: Id,
    offset_y: f32,
    viewport_height: f32,
}

impl ListScroll {
    pub fn new() -> Self {
        Self {
            id: Id::unique(),
            offset_y: 0.0,
            viewport_height: 0.0,
        }
    }

    pub fn id(&self) -> Id {
        self.id.clone()
    }

    pub fn on_scrolled(&mut self, viewport: scrollable::Viewport) {
        self.offset_y = viewport.absolute_offset().y;
        self.viewport_height = viewport.bounds().height;
    }

    pub fn page_size(&self) -> usize {
        ((self.viewport() / ROW_HEIGHT) as usize).max(1)
    }

    /// Scrolls the minimum amount needed to show the row at `index`.
    pub fn reveal<M: 'static>(&mut self, index: usize) -> Task<M> {
        let top = index as f32 * ROW_HEIGHT;
        let bottom = top + ROW_HEIGHT;

        let target = if top < self.offset_y {
            top
        } else if bottom > self.offset_y + self.viewport() {
            bottom - self.viewport()
        } else {
            return Task::none();
        };

        self.offset_y = target;
        operation::scroll_to(
            self.id.clone(),
            scrollable::AbsoluteOffset { x: 0.0, y: target },
        )
    }

    fn viewport(&self) -> f32 {
        if self.viewport_height > 0.0 {
            self.viewport_height
        } else {
            FALLBACK_VIEWPORT
        }
    }
}

/// A list row: click selects, double-click opens.
pub fn row<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    selected: bool,
    on_select: M,
    on_open: M,
) -> Element<'a, M> {
    let body = container(content)
        .padding([4, 8])
        .height(Length::Fixed(ROW_HEIGHT))
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            if selected {
                let primary = theme.extended_palette().primary.base;
                container::Style {
                    background: Some(primary.color.into()),
                    text_color: Some(primary.text),
                    ..container::Style::default()
                }
            } else {
                container::Style::default()
            }
        });

    mouse_area(body)
        .on_press(on_select)
        .on_double_click(on_open)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_clamps_to_list_bounds() {
        assert_eq!(navigate(Nav::Up, 0, 5, 3), 0);
        assert_eq!(navigate(Nav::Down, 4, 5, 3), 4);
        assert_eq!(navigate(Nav::PageDown, 3, 5, 3), 4);
        assert_eq!(navigate(Nav::PageUp, 1, 5, 3), 0);
        assert_eq!(navigate(Nav::Home, 3, 5, 3), 0);
        assert_eq!(navigate(Nav::End, 0, 5, 3), 4);
        assert_eq!(navigate(Nav::End, 0, 0, 3), 0);
    }
}
