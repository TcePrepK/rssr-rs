use ratatui::widgets::ListState;

/// Bundles a cursor index with a ratatui `ListState` so that scrollable list
/// views stay in sync without scattering paired fields across `App`.
///
/// Usage:
///   - Call `move_down` / `move_up` instead of manipulating `cursor` directly.
///   - Pass `&mut self.list_state` to `render_stateful_widget`.
pub struct ListScroll {
    /// Logical cursor (0-based index into the underlying vec).
    pub cursor: usize,
    /// Ratatui scroll state — kept in sync with `cursor` automatically.
    pub list_state: ListState,
}

impl Default for ListScroll {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { cursor: 0, list_state }
    }
}

impl ListScroll {
    /// Move the cursor down by one, clamping at `len - 1`.
    pub fn move_down(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        self.cursor = (self.cursor + 1).min(len - 1);
        self.list_state.select(Some(self.cursor));
    }

    /// Move the cursor up by one, clamping at 0.
    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.list_state.select(Some(self.cursor));
    }

    /// Explicitly set the cursor to `index` (does not clamp — caller must ensure validity).
    pub fn set(&mut self, index: usize) {
        self.cursor = index;
        self.list_state.select(Some(index));
    }

    /// Clamp the cursor to `[0, len)`. Call after the underlying list shrinks.
    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.cursor = 0;
            self.list_state.select(None);
        } else if self.cursor >= len {
            self.cursor = len - 1;
            self.list_state.select(Some(self.cursor));
        }
    }
}
