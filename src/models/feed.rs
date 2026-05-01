use super::{Article, Feed};

impl Feed {
    /// Returns `" [N]"` if there are unread articles, empty string otherwise.
    pub fn unread_badge(&self) -> String {
        if self.unread_count > 0 {
            format!(" [{}]", self.unread_count)
        } else {
            String::new()
        }
    }


}

impl Article {
    pub fn read_icon(&self) -> &'static str {
        if self.is_read { "○ " } else { "● " }
    }

}
