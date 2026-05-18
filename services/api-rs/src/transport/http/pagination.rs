use crate::shared::errors::{bad_request, ApiResult};

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 100;

#[derive(Clone, Copy)]
pub struct PageParams {
    pub limit: usize,
    pub offset: u64,
}

impl PageParams {
    pub fn fetch_limit(self) -> usize {
        self.limit + 1
    }

    pub fn storage_offset(self) -> ApiResult<i64> {
        i64::try_from(self.offset)
            .map_err(|_| bad_request("cursor_out_of_range", "cursor exceeds storage range"))
    }
}

pub fn parse_offset_page(cursor: Option<String>, limit: Option<u32>) -> ApiResult<PageParams> {
    let raw_limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT as u32);
    if raw_limit == 0 {
        return Err(bad_request(
            "limit_invalid",
            "limit must be greater than zero",
        ));
    }
    if raw_limit as usize > MAX_PAGE_LIMIT {
        return Err(bad_request(
            "limit_invalid",
            "limit exceeds maximum page size",
        ));
    }

    let offset = match cursor {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed != value {
                return Err(bad_request(
                    "cursor_invalid",
                    "cursor must be a numeric string without surrounding whitespace",
                ));
            }
            let parsed = trimmed
                .parse::<u64>()
                .map_err(|_| bad_request("cursor_invalid", "cursor must be numeric"))?;
            if parsed > i64::MAX as u64 {
                return Err(bad_request(
                    "cursor_out_of_range",
                    "cursor exceeds storage range",
                ));
            }
            parsed
        }
        None => 0,
    };

    Ok(PageParams {
        limit: raw_limit as usize,
        offset,
    })
}

pub fn trim_page<T>(items: &mut Vec<T>, page: PageParams) -> Option<String> {
    if items.len() <= page.limit {
        return None;
    }

    items.truncate(page.limit);
    page.offset
        .checked_add(page.limit as u64)
        .map(|value| value.to_string())
}

pub fn page_vec<T>(items: Vec<T>, page: PageParams) -> (Vec<T>, Option<String>) {
    let offset = usize::try_from(page.offset).unwrap_or(usize::MAX);
    let mut page_items = items
        .into_iter()
        .skip(offset)
        .take(page.fetch_limit())
        .collect::<Vec<_>>();
    let next_cursor = trim_page(&mut page_items, page);

    (page_items, next_cursor)
}

#[cfg(test)]
mod tests {
    use super::{page_vec, parse_offset_page, trim_page, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};

    #[test]
    fn parses_default_offset_page() {
        let page = parse_offset_page(None, None).unwrap_or_else(|_| panic!("default page parses"));

        assert_eq!(page.limit, DEFAULT_PAGE_LIMIT);
        assert_eq!(page.offset, 0);
        assert_eq!(page.fetch_limit(), DEFAULT_PAGE_LIMIT + 1);
    }

    #[test]
    fn rejects_invalid_page_inputs() {
        assert!(parse_offset_page(None, Some(0)).is_err());
        assert!(parse_offset_page(None, Some((MAX_PAGE_LIMIT + 1) as u32)).is_err());
        assert!(parse_offset_page(Some("not-a-number".to_string()), None).is_err());
        assert!(parse_offset_page(Some(" 1".to_string()), None).is_err());
    }

    #[test]
    fn trims_extra_row_and_returns_next_offset_cursor() {
        let page = parse_offset_page(Some("10".to_string()), Some(2))
            .unwrap_or_else(|_| panic!("page parses"));
        let mut items = vec![1, 2, 3];

        let next_cursor = trim_page(&mut items, page);

        assert_eq!(items, vec![1, 2]);
        assert_eq!(next_cursor.as_deref(), Some("12"));
    }

    #[test]
    fn pages_in_memory_vectors_with_the_same_offset_cursor() {
        let page = parse_offset_page(Some("2".to_string()), Some(2))
            .unwrap_or_else(|_| panic!("page parses"));

        let (items, next_cursor) = page_vec(vec![1, 2, 3, 4, 5], page);

        assert_eq!(items, vec![3, 4]);
        assert_eq!(next_cursor.as_deref(), Some("4"));
    }
}
