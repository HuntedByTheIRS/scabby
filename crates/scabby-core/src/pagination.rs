//! Page and `Link` header handling for collection endpoints.

use serde::Serialize;

/// Items returned when the caller does not ask for a size.
pub const DEFAULT_PER_PAGE: u32 = 30;

/// Largest page Scabby will serve.
pub const MAX_PER_PAGE: u32 = 100;

/// A requested page of a collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    /// One-based page number.
    pub number: u32,
    /// Items per page, already clamped.
    pub per_page: u32,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            number: 1,
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl Page {
    /// Clamp caller-supplied values into something the database can serve.
    pub fn new(number: Option<u32>, per_page: Option<u32>) -> Self {
        Self {
            number: number.unwrap_or(1).max(1),
            per_page: per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE),
        }
    }

    /// Value for a SQL `LIMIT`.
    pub fn limit(&self) -> i64 {
        i64::from(self.per_page)
    }

    /// Value for a SQL `OFFSET`.
    pub fn offset(&self) -> i64 {
        i64::from(self.number.saturating_sub(1)) * self.limit()
    }

    /// Number of pages a total falls into.
    pub fn total_pages(&self, total: i64) -> u32 {
        if total <= 0 {
            return 0;
        }
        let pages = (total as u64).div_ceil(u64::from(self.per_page));
        u32::try_from(pages).unwrap_or(u32::MAX)
    }

    /// The `Link` header for a list whose total was not counted.
    ///
    /// Commits are the case: walking a large history to produce an exact page count would be work
    /// nobody asked for, so the list reports what it knows, which is whether a further page exists.
    pub fn link_header_without_total(
        &self,
        url_without_page: &str,
        has_more: bool,
    ) -> Option<String> {
        if !has_more {
            return None;
        }
        let separator = if url_without_page.contains('?') {
            '&'
        } else {
            '?'
        };
        Some(format!(
            "<{url_without_page}{separator}page={}>; rel=\"next\"",
            self.number + 1
        ))
    }

    /// The `Link` header for this page, or `None` when there is only one page.
    ///
    /// `url_without_page` must not already carry a `page` parameter.
    pub fn link_header(&self, url_without_page: &str, total_pages: u32) -> Option<String> {
        if total_pages <= 1 {
            return None;
        }
        let separator = if url_without_page.contains('?') {
            '&'
        } else {
            '?'
        };
        let link = |page: u32, rel: &str| {
            format!("<{url_without_page}{separator}page={page}>; rel=\"{rel}\"")
        };

        let mut links = vec![link(1, "first")];
        if self.number > 1 {
            links.push(link(self.number - 1, "prev"));
        }
        if self.number < total_pages {
            links.push(link(self.number + 1, "next"));
        }
        links.push(link(total_pages, "last"));
        Some(links.join(", "))
    }
}

/// The body of a paginated collection response.
#[derive(Debug, Clone, Serialize)]
pub struct PageResult<T> {
    /// The page's items.
    pub items: Vec<T>,
    /// One-based page number of this response.
    pub page: u32,
    /// Size of the page.
    pub per_page: u32,
    /// Total items across every page, or `null` when the caller did not ask for a count to be taken.
    pub total: Option<i64>,
    /// Total pages available, or `null` when `total` is.
    pub total_pages: Option<u32>,
}

impl<T> PageResult<T> {
    /// Build a response body from a page of items and the unpaginated total.
    pub fn new(items: Vec<T>, page: Page, total: i64) -> Self {
        Self {
            items,
            page: page.number,
            per_page: page.per_page,
            total: Some(total),
            total_pages: Some(page.total_pages(total)),
        }
    }

    /// Build a response body for a list that was not counted from end to end.
    pub fn uncounted(items: Vec<T>, page: Page) -> Self {
        Self {
            items,
            page: page.number,
            per_page: page.per_page,
            total: None,
            total_pages: None,
        }
    }

    /// Whether this page came back full, which is the only hint an uncounted list has that another
    /// page might exist.
    pub fn is_full(&self) -> bool {
        self.items.len() as u32 >= self.per_page
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_sizes() {
        assert_eq!(Page::new(None, None).per_page, DEFAULT_PER_PAGE);
        assert_eq!(Page::new(Some(0), Some(0)).per_page, 1);
        assert_eq!(Page::new(None, Some(5000)).per_page, MAX_PER_PAGE);
        assert_eq!(Page::new(Some(0), None).number, 1);
        assert_eq!(Page::new(Some(3), Some(10)).offset(), 20);
    }

    #[test]
    fn counts_pages() {
        let page = Page::new(None, Some(30));
        assert_eq!(page.total_pages(0), 0);
        assert_eq!(page.total_pages(30), 1);
        assert_eq!(page.total_pages(31), 2);
    }

    #[test]
    fn uncounted_lists_link_only_forward() {
        let page = Page::new(Some(1), Some(30));
        assert_eq!(
            page.link_header_without_total("/api/v1/repos?ref=main", true)
                .as_deref(),
            Some("</api/v1/repos?ref=main&page=2>; rel=\"next\"")
        );
        assert!(
            page.link_header_without_total("/api/v1/repos", false)
                .is_none()
        );

        let result = PageResult::uncounted(vec![1, 2, 3], Page::new(None, Some(3)));
        assert!(result.total.is_none());
        assert!(result.total_pages.is_none());
        assert!(result.is_full());
    }

    #[test]
    fn writes_link_headers() {
        let page = Page::new(Some(2), Some(30));
        let header = page.link_header("/api/v1/repos?per_page=30", 4).unwrap();
        assert!(header.contains("rel=\"first\""));
        assert!(header.contains("rel=\"prev\""));
        assert!(header.contains("rel=\"next\""));
        assert!(header.contains("rel=\"last\""));
        assert!(
            header.contains("</api/v1/repos?per_page=30&page=3>; rel=\"next\""),
            "{header}"
        );
        assert!(
            header.contains("</api/v1/repos?per_page=30&page=1>; rel=\"prev\""),
            "{header}"
        );

        let single = Page::new(None, None);
        assert!(single.link_header("/api/v1/repos", 1).is_none());
        assert!(single.link_header("/api/v1/repos", 0).is_none());
    }
}
