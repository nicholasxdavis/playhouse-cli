//! Terminal layout that stays valid at any window size (resize-safe).

use ratatui::layout::Rect;

use crate::tui::app::App;
use crate::tui::components;
use crate::tui::mascot;

#[derive(Debug, Clone, Copy)]
pub struct MainPanels {
    pub header: Rect,
    pub feed: Rect,
    pub input: Rect,
    pub status: Rect,
}

/// Split the main area into panes. Feed always gets at least one row when height >= 4.
pub fn split_main(area: Rect, app: &App) -> MainPanels {
    let width = area.width.max(1);
    let height = area.height.max(1);
    let x = area.x;
    let mut y = area.y;

    if height < 4 {
        return MainPanels {
            header: Rect::default(),
            feed: Rect::default(),
            input: Rect::default(),
            status: Rect {
                x,
                y,
                width,
                height,
            },
        };
    }

    let status_h = 1u16;
    let ideal_header = mascot::welcome_header_height(&app.workspace);

    // Shrink header and input until feed can have at least one row.
    let mut header_h = ideal_header.min(height.saturating_sub(5)).max(1);
    let mut input_h = components::input::block_height(app, width, height)
        .clamp(3, 6)
        .min(height.saturating_sub(header_h + status_h + 1));

    let mut feed_h = height.saturating_sub(header_h + input_h + status_h);
    if feed_h == 0 {
        input_h = 3.min(height.saturating_sub(header_h + status_h + 1));
        feed_h = height.saturating_sub(header_h + input_h + status_h);
    }
    if feed_h == 0 {
        header_h = 1.min(height.saturating_sub(input_h + status_h + 1));
        feed_h = height.saturating_sub(header_h + input_h + status_h);
    }
    if feed_h == 0 {
        // Last resort: single feed column + status
        header_h = 0;
        input_h = height.saturating_sub(status_h + 1).min(3);
        feed_h = height.saturating_sub(input_h + status_h).max(1);
    }

    let header = if header_h > 0 {
        let r = Rect {
            x,
            y,
            width,
            height: header_h,
        };
        y += header_h;
        r
    } else {
        Rect::default()
    };

    let feed = Rect {
        x,
        y,
        width,
        height: feed_h.max(1),
    };
    y += feed.height;

    let input = Rect {
        x,
        y,
        width,
        height: input_h,
    };
    y += input_h;

    let status = Rect {
        x,
        y,
        width,
        height: status_h,
    };

    MainPanels {
        header,
        feed,
        input,
        status,
    }
}

pub fn is_tiny(area: Rect) -> bool {
    area.width < 16 || area.height < 4
}

pub fn is_compact(area: Rect) -> bool {
    area.width < 48 || area.height < 10
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;

    #[test]
    fn split_main_always_gives_feed_a_row() {
        let app = App::new(".", false);
        for h in 4..30 {
            let panels = split_main(Rect::new(0, 0, 80, h), &app);
            assert!(panels.feed.height >= 1, "height {h}");
            assert_eq!(panels.status.height, 1);
        }
    }

    #[test]
    fn split_main_narrow_width() {
        let app = App::new(".", false);
        let panels = split_main(Rect::new(0, 0, 30, 20), &app);
        assert!(panels.feed.width >= 1);
    }
}
