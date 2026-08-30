use crate::Point;
use k2f_core::{CanvasMode, GeometryNode, Page, PageConfig, Pt};

/// Returns the available content width inside the page margins.
pub fn content_width(page_config: &PageConfig) -> Pt {
    page_config.width - page_config.margin[1] - page_config.margin[3]
}

/// Returns the available content height inside the page margins.
pub fn content_height(page_config: &PageConfig) -> Pt {
    page_config.height - page_config.margin[0] - page_config.margin[2]
}

/// Validates `PageConfig` for consistent margin behavior.
///
/// This is a layout-engine concern (not semantic-tree validation).
pub fn validate_page_config(page_config: &PageConfig) -> Result<(), String> {
    let w = page_config.width;
    let h = page_config.height;
    if w.0 <= 0 {
        return Err(format!("Invalid PageConfig.width: {} (must be > 0)", w.0));
    }
    if h.0 <= 0 {
        return Err(format!("Invalid PageConfig.height: {} (must be > 0)", h.0));
    }

    let [top, right, bottom, left] = page_config.margin;
    for (name, v) in [
        ("margin.top", top),
        ("margin.right", right),
        ("margin.bottom", bottom),
        ("margin.left", left),
    ] {
        if v.0 < 0 {
            return Err(format!(
                "Invalid PageConfig.{}: {} (must be >= 0)",
                name, v.0
            ));
        }
    }

    let cw = content_width(page_config);
    if cw.0 < 0 {
        return Err(format!(
            "Invalid PageConfig margins: left+right exceed width (content_width={})",
            cw.0
        ));
    }
    let ch = content_height(page_config);
    if ch.0 < 0 {
        return Err(format!(
            "Invalid PageConfig margins: top+bottom exceed height (content_height={})",
            ch.0
        ));
    }

    Ok(())
}

/// Active multi-column packing region on the current page.
#[derive(Debug, Clone)]
pub struct ColumnBand {
    pub content_x: Pt,
    pub column_widths: Vec<Pt>,
    pub gap: Pt,
    pub col: usize,
    pub band_top: Pt,
    pub band_bottom: Pt,
    /// Highest y reached by any column in this band (used when exiting).
    pub used_max_y: Pt,
}

impl ColumnBand {
    pub fn count(&self) -> usize {
        self.column_widths.len()
    }

    pub fn column_x(&self) -> Pt {
        let mut x = self.content_x;
        for i in 0..self.col {
            x += self.column_widths[i] + self.gap;
        }
        x
    }

    pub fn column_width(&self) -> Pt {
        self.column_widths[self.col]
    }
}

pub struct Paginator {
    pub pages: Vec<Page>,
    pub current_page_index: usize,
    pub current_y: Pt,
    pub page_config: PageConfig,
    pub mode: CanvasMode,
    pub column_band: Option<ColumnBand>,
}

impl Paginator {
    pub fn new(page_config: PageConfig, mode: CanvasMode) -> Self {
        let first_page = Page {
            index: 0,
            width: page_config.width,
            height: page_config.height,
            root: GeometryNode {
                id: "root_page_0".to_string(),
                x: Pt::ZERO,
                y: Pt::ZERO,
                width: page_config.width,
                height: page_config.height,
                glyphs: vec![],
                text_runs: vec![],
                fill_rects: vec![],
                children: vec![],
            },
        };

        Paginator {
            pages: vec![first_page],
            current_page_index: 0,
            current_y: page_config.margin[0], // Start at top margin
            page_config,
            mode,
            column_band: None,
        }
    }

    pub fn content_left(&self) -> Pt {
        self.page_config.margin[3]
    }

    pub fn page_bottom_limit(&self) -> Pt {
        self.page_config.height - self.page_config.margin[2]
    }

    /// Begin packing into equal column slots within `[band_top, band_bottom)`.
    pub fn enter_column_band(
        &mut self,
        content_x: Pt,
        column_widths: Vec<Pt>,
        gap: Pt,
        band_top: Pt,
        band_bottom: Pt,
    ) -> Result<(), String> {
        if column_widths.is_empty() {
            return Err("column band requires at least one column".into());
        }
        if band_bottom < band_top {
            return Err(format!(
                "invalid column band: bottom {} < top {}",
                band_bottom.0, band_top.0
            ));
        }
        self.column_band = Some(ColumnBand {
            content_x,
            column_widths,
            gap,
            col: 0,
            band_top,
            band_bottom,
            used_max_y: band_top,
        });
        self.current_y = band_top;
        Ok(())
    }

    /// Leave the column band; `current_y` becomes the max y used across columns.
    pub fn exit_column_band(&mut self) {
        if let Some(band) = self.column_band.take() {
            self.current_y = band.used_max_y;
        }
    }

    pub fn in_column_band(&self) -> bool {
        self.column_band.is_some()
    }

    pub fn remaining_height(&self) -> Pt {
        if self.mode == CanvasMode::Infinite {
            return Pt(i128::MAX);
        }
        let bottom_limit = match &self.column_band {
            Some(b) => b.band_bottom,
            None => self.page_bottom_limit(),
        };
        if self.current_y >= bottom_limit {
            Pt::ZERO
        } else {
            bottom_limit - self.current_y
        }
    }

    /// True when at the top of the current packing region (page content or column band).
    pub fn at_content_top(&self) -> bool {
        match &self.column_band {
            Some(b) => self.current_y == b.band_top,
            None => self.current_y == self.page_config.margin[0],
        }
    }

    pub fn page_content_height(&self) -> Pt {
        match &self.column_band {
            Some(b) => b.band_bottom - b.band_top,
            None => content_height(&self.page_config),
        }
    }

    /// Remaining height from the page's current y to the page bottom (ignores band).
    pub fn remaining_page_height(&self) -> Pt {
        if self.mode == CanvasMode::Infinite {
            return Pt(i128::MAX);
        }
        let bottom = self.page_bottom_limit();
        if self.current_y >= bottom {
            Pt::ZERO
        } else {
            bottom - self.current_y
        }
    }

    pub fn allocate_space(&mut self, height: Pt) -> Point {
        if self.mode == CanvasMode::Infinite {
            let pos = Point::new(self.page_config.margin[3], self.current_y);
            self.current_y += height;
            let current_page = &mut self.pages[0];
            if self.current_y + self.page_config.margin[2] > current_page.height {
                current_page.height = self.current_y + self.page_config.margin[2];
                current_page.root.height = current_page.height;
            }
            return pos;
        }

        if self.column_band.is_some() {
            let (x, band_top_y) = {
                let band = self.column_band.as_ref().unwrap();
                let mut x = band.content_x;
                for i in 0..band.col {
                    x += band.column_widths[i] + band.gap;
                }
                (x, self.current_y)
            };
            let pos = Point::new(x, band_top_y);
            self.current_y += height;
            let y = self.current_y;
            if let Some(b) = self.column_band.as_mut() {
                if y > b.used_max_y {
                    b.used_max_y = y;
                }
            }
            return pos;
        }

        let bottom_limit = self.page_bottom_limit();
        if self.current_y + height > bottom_limit {
            self.add_new_page();
        }

        let pos = Point::new(self.page_config.margin[3], self.current_y);
        self.current_y += height;
        pos
    }

    /// Advance to the next column in the band. Returns true if a page break was taken.
    pub fn column_break(&mut self) -> Result<bool, String> {
        let Some(band) = &mut self.column_band else {
            return Err("column_break without active column band".into());
        };
        if band.col + 1 < band.count() {
            band.col += 1;
            self.current_y = band.band_top;
            return Ok(false);
        }
        // Last column full → new page, reset to column 0 with full-page band.
        self.page_break();
        Ok(true)
    }

    /// Force a page break in paged mode. No-op in infinite mode.
    /// Preserves an active column band geometry on the new page (full page height).
    pub fn page_break(&mut self) {
        if self.mode != CanvasMode::Paged {
            return;
        }
        let resume = self
            .column_band
            .as_ref()
            .map(|b| (b.content_x, b.column_widths.clone(), b.gap));
        self.add_new_page();
        if let Some((content_x, widths, gap)) = resume {
            let top = self.page_config.margin[0];
            let bottom = self.page_bottom_limit();
            let _ = self.enter_column_band(content_x, widths, gap, top, bottom);
        }
    }

    fn add_new_page(&mut self) {
        self.current_page_index += 1;
        let new_page = Page {
            index: self.current_page_index,
            width: self.page_config.width,
            height: self.page_config.height,
            root: GeometryNode {
                id: format!("root_page_{}", self.current_page_index),
                x: Pt::ZERO,
                y: Pt::ZERO,
                width: self.page_config.width,
                height: self.page_config.height,
                glyphs: vec![],
                text_runs: vec![],
                fill_rects: vec![],
                children: vec![],
            },
        };
        self.pages.push(new_page);
        self.current_y = self.page_config.margin[0];
        self.column_band = None;
    }

    pub fn add_item(&mut self, item: GeometryNode) {
        self.pages[self.current_page_index].root.children.push(item);
    }
}

#[cfg(test)]
mod column_band_tests {
    use super::*;

    fn page() -> PageConfig {
        PageConfig {
            width: Pt(200_000),
            height: Pt(100_000),
            margin: [Pt(10_000), Pt(10_000), Pt(10_000), Pt(10_000)],
        }
    }

    #[test]
    fn allocate_uses_column_x() {
        let mut p = Paginator::new(page(), CanvasMode::Paged);
        p.enter_column_band(
            Pt(10_000),
            vec![Pt(85_000), Pt(85_000)],
            Pt(10_000),
            Pt(10_000),
            Pt(90_000),
        )
        .unwrap();
        let a = p.allocate_space(Pt(5_000));
        assert_eq!(a.x, Pt(10_000));
        assert_eq!(a.y, Pt(10_000));
        p.column_break().unwrap();
        let b = p.allocate_space(Pt(5_000));
        assert_eq!(b.x, Pt(105_000));
        assert_eq!(b.y, Pt(10_000));
    }

    #[test]
    fn exit_band_uses_max_y() {
        let mut p = Paginator::new(page(), CanvasMode::Paged);
        p.enter_column_band(
            Pt(10_000),
            vec![Pt(85_000), Pt(85_000)],
            Pt(10_000),
            Pt(10_000),
            Pt(90_000),
        )
        .unwrap();
        p.allocate_space(Pt(20_000));
        p.column_break().unwrap();
        p.allocate_space(Pt(5_000));
        p.exit_column_band();
        assert_eq!(p.current_y, Pt(30_000));
        assert!(!p.in_column_band());
    }

    #[test]
    fn last_column_break_starts_new_page() {
        let mut p = Paginator::new(page(), CanvasMode::Paged);
        p.enter_column_band(
            Pt(10_000),
            vec![Pt(85_000), Pt(85_000)],
            Pt(10_000),
            Pt(10_000),
            Pt(90_000),
        )
        .unwrap();
        p.column_break().unwrap();
        let paged = p.column_break().unwrap();
        assert!(paged);
        assert_eq!(p.pages.len(), 2);
        assert!(p.in_column_band());
        assert_eq!(p.column_band.as_ref().unwrap().col, 0);
    }
}
