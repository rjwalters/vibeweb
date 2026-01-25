//! Navigation history model for back/forward/reload functionality.
//!
//! This module provides a session history implementation that tracks visited pages
//! and enables navigation through the browsing session.

use std::time::Instant;

/// A single entry in the navigation history.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The URL of the page.
    pub url: String,
    /// Page title (if available).
    pub title: Option<String>,
    /// Timestamp when the page was visited.
    pub visited_at: Instant,
    /// Scroll position X when the user left the page (for restoration).
    pub scroll_x: f32,
    /// Scroll position Y when the user left the page (for restoration).
    pub scroll_y: f32,
}

impl HistoryEntry {
    /// Creates a new history entry with the given URL.
    fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: None,
            visited_at: Instant::now(),
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }
}

/// Session history with back/forward navigation.
///
/// The history maintains a list of visited pages and a cursor pointing to the
/// current position. Navigation operations move this cursor, while new navigations
/// may truncate forward history.
///
/// # Example
///
/// ```
/// use browser::History;
///
/// let mut history = History::new();
///
/// // Navigate to some pages
/// history.navigate("https://example.com");
/// history.navigate("https://example.com/page1");
/// history.navigate("https://example.com/page2");
///
/// // Go back
/// assert!(history.can_go_back());
/// let entry = history.back().unwrap();
/// assert_eq!(entry.url, "https://example.com/page1");
///
/// // Go forward
/// assert!(history.can_go_forward());
/// let entry = history.forward().unwrap();
/// assert_eq!(entry.url, "https://example.com/page2");
/// ```
#[derive(Debug)]
pub struct History {
    /// All history entries.
    entries: Vec<HistoryEntry>,
    /// Current position in the history (index into entries).
    /// When entries is non-empty, this is always a valid index.
    current: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// Creates a new empty history.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current: 0,
        }
    }

    /// Navigates to a new URL, adding it to history.
    ///
    /// This clears any forward history entries (entries after the current position).
    /// Returns a reference to the newly created entry.
    ///
    /// # Example
    ///
    /// ```
    /// use browser::History;
    ///
    /// let mut history = History::new();
    /// let entry = history.navigate("https://example.com");
    /// assert_eq!(entry.url, "https://example.com");
    /// ```
    pub fn navigate(&mut self, url: &str) -> &HistoryEntry {
        // If we're not at the end, truncate forward history
        if !self.entries.is_empty() {
            self.entries.truncate(self.current + 1);
        }

        // Add new entry
        self.entries.push(HistoryEntry::new(url));
        self.current = self.entries.len() - 1;

        &self.entries[self.current]
    }

    /// Goes back one entry, returns the entry to navigate to.
    ///
    /// Returns `None` if there is no previous entry (already at the beginning).
    ///
    /// # Example
    ///
    /// ```
    /// use browser::History;
    ///
    /// let mut history = History::new();
    /// history.navigate("https://a.com");
    /// history.navigate("https://b.com");
    ///
    /// let entry = history.back().unwrap();
    /// assert_eq!(entry.url, "https://a.com");
    /// ```
    pub fn back(&mut self) -> Option<&HistoryEntry> {
        if self.can_go_back() {
            self.current -= 1;
            Some(&self.entries[self.current])
        } else {
            None
        }
    }

    /// Goes forward one entry, returns the entry to navigate to.
    ///
    /// Returns `None` if there is no next entry (already at the end).
    ///
    /// # Example
    ///
    /// ```
    /// use browser::History;
    ///
    /// let mut history = History::new();
    /// history.navigate("https://a.com");
    /// history.navigate("https://b.com");
    /// history.back();
    ///
    /// let entry = history.forward().unwrap();
    /// assert_eq!(entry.url, "https://b.com");
    /// ```
    pub fn forward(&mut self) -> Option<&HistoryEntry> {
        if self.can_go_forward() {
            self.current += 1;
            Some(&self.entries[self.current])
        } else {
            None
        }
    }

    /// Checks if back navigation is possible.
    ///
    /// Returns `true` if there is at least one entry before the current position.
    #[inline]
    pub fn can_go_back(&self) -> bool {
        self.current > 0
    }

    /// Checks if forward navigation is possible.
    ///
    /// Returns `true` if there is at least one entry after the current position.
    #[inline]
    pub fn can_go_forward(&self) -> bool {
        !self.entries.is_empty() && self.current < self.entries.len() - 1
    }

    /// Gets the current entry.
    ///
    /// Returns `None` if the history is empty.
    pub fn current(&self) -> Option<&HistoryEntry> {
        if self.entries.is_empty() {
            None
        } else {
            Some(&self.entries[self.current])
        }
    }

    /// Gets the current URL.
    ///
    /// Returns `None` if the history is empty.
    pub fn current_url(&self) -> Option<&str> {
        self.current().map(|entry| entry.url.as_str())
    }

    /// Updates the current entry's scroll position.
    ///
    /// This is useful for restoring scroll position when navigating back/forward.
    /// Does nothing if the history is empty.
    pub fn update_scroll(&mut self, x: f32, y: f32) {
        if let Some(entry) = self.entries.get_mut(self.current) {
            entry.scroll_x = x;
            entry.scroll_y = y;
        }
    }

    /// Updates the current entry's title.
    ///
    /// Does nothing if the history is empty.
    pub fn update_title(&mut self, title: &str) {
        if let Some(entry) = self.entries.get_mut(self.current) {
            entry.title = Some(title.to_string());
        }
    }

    /// Reloads the current page (returns current entry without modifying history).
    ///
    /// Returns `None` if the history is empty.
    pub fn reload(&self) -> Option<&HistoryEntry> {
        self.current()
    }

    /// Gets the number of entries in the history.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the history is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gets all entries for debugging/display.
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Gets the current position (index) in the history.
    ///
    /// Returns 0 if the history is empty.
    #[inline]
    pub fn position(&self) -> usize {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_is_empty() {
        let history = History::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
        assert!(history.current().is_none());
        assert!(history.current_url().is_none());
    }

    #[test]
    fn test_basic_navigation() {
        let mut history = History::new();
        assert!(!history.can_go_back());

        history.navigate("http://example.com");
        assert_eq!(history.current_url(), Some("http://example.com"));
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
        assert_eq!(history.len(), 1);

        history.navigate("http://example.com/page2");
        assert_eq!(history.current_url(), Some("http://example.com/page2"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_back_forward() {
        let mut history = History::new();
        history.navigate("http://a.com");
        history.navigate("http://b.com");
        history.navigate("http://c.com");

        // Go back once
        let entry = history.back().unwrap();
        assert_eq!(entry.url, "http://b.com");
        assert_eq!(history.current_url(), Some("http://b.com"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        // Go back again
        let entry = history.back().unwrap();
        assert_eq!(entry.url, "http://a.com");
        assert_eq!(history.current_url(), Some("http://a.com"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        // Can't go back further
        assert!(history.back().is_none());

        // Go forward
        let entry = history.forward().unwrap();
        assert_eq!(entry.url, "http://b.com");

        // Go forward again
        let entry = history.forward().unwrap();
        assert_eq!(entry.url, "http://c.com");
        assert!(!history.can_go_forward());

        // Can't go forward further
        assert!(history.forward().is_none());
    }

    #[test]
    fn test_navigate_clears_forward() {
        let mut history = History::new();
        history.navigate("http://a.com");
        history.navigate("http://b.com");
        history.navigate("http://c.com");

        // Go back to b.com
        history.back();
        assert_eq!(history.current_url(), Some("http://b.com"));
        assert!(history.can_go_forward());
        assert_eq!(history.len(), 3);

        // Navigate to new page - should clear forward history (c.com)
        history.navigate("http://d.com");
        assert_eq!(history.current_url(), Some("http://d.com"));
        assert!(!history.can_go_forward());
        assert_eq!(history.len(), 3); // a.com, b.com, d.com (c.com cleared)

        // Verify history contents
        let entries = history.entries();
        assert_eq!(entries[0].url, "http://a.com");
        assert_eq!(entries[1].url, "http://b.com");
        assert_eq!(entries[2].url, "http://d.com");
    }

    #[test]
    fn test_navigate_from_middle() {
        let mut history = History::new();
        history.navigate("http://1.com");
        history.navigate("http://2.com");
        history.navigate("http://3.com");
        history.navigate("http://4.com");

        // Go back twice (now at 2.com)
        history.back();
        history.back();
        assert_eq!(history.current_url(), Some("http://2.com"));

        // Navigate to new page
        history.navigate("http://new.com");

        // Should have 1.com, 2.com, new.com
        assert_eq!(history.len(), 3);
        assert_eq!(history.current_url(), Some("http://new.com"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_update_scroll() {
        let mut history = History::new();
        history.navigate("http://example.com");

        // Default scroll is 0
        let entry = history.current().unwrap();
        assert_eq!(entry.scroll_x, 0.0);
        assert_eq!(entry.scroll_y, 0.0);

        // Update scroll
        history.update_scroll(100.0, 250.5);
        let entry = history.current().unwrap();
        assert_eq!(entry.scroll_x, 100.0);
        assert_eq!(entry.scroll_y, 250.5);

        // Navigate to new page
        history.navigate("http://example.com/page2");

        // Go back - scroll should be preserved
        history.back();
        let entry = history.current().unwrap();
        assert_eq!(entry.scroll_x, 100.0);
        assert_eq!(entry.scroll_y, 250.5);
    }

    #[test]
    fn test_update_title() {
        let mut history = History::new();
        history.navigate("http://example.com");

        // Default title is None
        let entry = history.current().unwrap();
        assert!(entry.title.is_none());

        // Update title
        history.update_title("Example Website");
        let entry = history.current().unwrap();
        assert_eq!(entry.title.as_deref(), Some("Example Website"));

        // Navigate to new page and back - title preserved
        history.navigate("http://example.com/page2");
        history.update_title("Page 2");
        history.back();
        assert_eq!(
            history.current().unwrap().title.as_deref(),
            Some("Example Website")
        );
    }

    #[test]
    fn test_reload() {
        let mut history = History::new();

        // Reload on empty history returns None
        assert!(history.reload().is_none());

        history.navigate("http://example.com");
        history.update_title("Example");

        // Reload returns current entry
        let entry = history.reload().unwrap();
        assert_eq!(entry.url, "http://example.com");
        assert_eq!(entry.title.as_deref(), Some("Example"));

        // Reload doesn't change history length
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_position() {
        let mut history = History::new();
        assert_eq!(history.position(), 0);

        history.navigate("http://a.com");
        assert_eq!(history.position(), 0);

        history.navigate("http://b.com");
        assert_eq!(history.position(), 1);

        history.navigate("http://c.com");
        assert_eq!(history.position(), 2);

        history.back();
        assert_eq!(history.position(), 1);

        history.back();
        assert_eq!(history.position(), 0);

        history.forward();
        assert_eq!(history.position(), 1);
    }

    #[test]
    fn test_default_trait() {
        let history = History::default();
        assert!(history.is_empty());
    }

    #[test]
    fn test_empty_back_forward() {
        let mut history = History::new();

        // Should handle gracefully
        assert!(history.back().is_none());
        assert!(history.forward().is_none());
    }

    #[test]
    fn test_single_entry_navigation() {
        let mut history = History::new();
        history.navigate("http://only.com");

        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
        assert!(history.back().is_none());
        assert!(history.forward().is_none());
    }

    #[test]
    fn test_update_on_empty_history() {
        let mut history = History::new();

        // Should not panic
        history.update_scroll(10.0, 20.0);
        history.update_title("Title");

        assert!(history.is_empty());
    }

    #[test]
    fn test_entries_slice() {
        let mut history = History::new();
        history.navigate("http://a.com");
        history.navigate("http://b.com");
        history.navigate("http://c.com");

        let entries = history.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].url, "http://a.com");
        assert_eq!(entries[1].url, "http://b.com");
        assert_eq!(entries[2].url, "http://c.com");
    }
}
