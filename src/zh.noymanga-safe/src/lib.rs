#![no_std]

extern crate alloc;

use aidoku::{
    BasicLoginHandler, Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, ImageRequestProvider,
    Listing, ListingProvider, Manga, MangaPageResult, NotificationHandler, Page, PageContent,
    PageContext, Result, Source,
    alloc::{String, Vec, format, string::ToString, vec},
    imports::net::Request,
    prelude::*,
};

mod models;
mod net;

const WEB_URL: &str = "https://noymanga.com";
const IMAGE_URL: &str = "https://img.noymanga.com";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) Aidoku/NoyAcgSafe";
const PAGE_SIZE: i64 = 20;

struct NoyMangaSafe;

impl NoyMangaSafe {
    fn ensure_safe(book: &models::Book) -> Result<()> {
        if book.is_safe() {
            Ok(())
        } else {
            Err(error!("该作品不是全年龄内容"))
        }
    }

    fn details(&self, key: &str) -> Result<models::DetailResponse> {
        let detail: models::DetailResponse = net::get(&format!("/v4/book/{key}"))?;
        Self::ensure_safe(&detail.book.info)?;
        Ok(detail)
    }

    fn chapters(&self, detail: &models::DetailResponse, manga_key: &str) -> Vec<Chapter> {
        if detail.book.info.Mode != 1 {
            return vec![Chapter {
                key: "0".into(),
                title: Some("全一册".into()),
                chapter_number: Some(1.0),
                url: Some(format!("{WEB_URL}/manga/{manga_key}")),
                language: Some("zh".into()),
                ..Default::default()
            }];
        }
        let mut result = Vec::new();
        for category in &detail.chapters.categories {
            if let Some(items) = detail.chapters.data.get(&category.id.to_string()) {
                for item in items {
                    result.push(Chapter {
                        key: item.id.to_string(),
                        title: Some(item.name.clone()),
                        chapter_number: Some(item.sort as f32 + 1.0),
                        date_uploaded: Some(item.created_at),
                        scanlators: if category.name.is_empty() {
                            None
                        } else {
                            Some(vec![category.name.clone()])
                        },
                        url: Some(format!("{WEB_URL}/reader/{manga_key}/{}", item.id)),
                        language: Some("zh".into()),
                        ..Default::default()
                    });
                }
            }
        }
        result.sort_by(|a, b| {
            b.chapter_number
                .partial_cmp(&a.chapter_number)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        result
    }

    fn listing(&self, path: &str, level: &str, page: i32) -> Result<MangaPageResult> {
        let body = if level.is_empty() {
            format!("page={page}")
        } else {
            format!("page={page}&type={level}")
        };
        let response: models::ListResponse = net::post(path, &body)?;
        let entries = response
            .info
            .into_iter()
            .filter(models::Book::is_safe)
            .map(models::Book::into_manga)
            .collect();
        Ok(MangaPageResult {
            entries,
            has_next_page: i64::from(page) * PAGE_SIZE < response.len,
        })
    }
}

impl Source for NoyMangaSafe {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let mut value = query.unwrap_or_default();
        let mut mode = "default";
        let mut sort = "default";
        let mut finished = "all";
        for filter in filters {
            match filter {
                FilterValue::Text { id, value: text } if !text.is_empty() => match id.as_str() {
                    "author" => {
                        value = text;
                        mode = "author";
                    }
                    "tag" => {
                        value = text;
                        mode = "tag";
                    }
                    _ => {}
                },
                FilterValue::Select { id, value } if id == "finished" => {
                    finished = match value.as_str() {
                        "true" => "true",
                        "false" => "false",
                        _ => "all",
                    };
                }
                FilterValue::Sort { id, index, .. } if id == "sort" => {
                    sort = match index {
                        1 => "views",
                        2 => "favorites",
                        3 => "rating",
                        _ => "default",
                    };
                }
                _ => {}
            }
        }
        if value.is_empty() {
            let browse_sort = if sort == "default" { "new" } else { sort };
            let body = format!("page={page}&sort={browse_sort}&finished={finished}");
            let response: models::ListResponse = net::post("/b1/booklist", &body)?;
            let entries = response
                .info
                .into_iter()
                .filter(models::Book::is_safe)
                .map(models::Book::into_manga)
                .collect();
            Ok(MangaPageResult {
                entries,
                has_next_page: i64::from(page) * PAGE_SIZE < response.len,
            })
        } else {
            let value = aidoku::helpers::uri::encode_uri_component(&value);
            let body = format!(
                "value={value}&mode={mode}&sort={sort}&type=book&page={page}&finished={finished}"
            );
            let response: models::SearchResponse = net::post("/v4/search/fetch", &body)?;
            let entries = response
                .data
                .into_iter()
                .filter(models::SearchBook::is_safe_manga)
                .map(models::SearchBook::into_manga)
                .collect();
            Ok(MangaPageResult {
                entries,
                has_next_page: i64::from(page) * PAGE_SIZE < response.count,
            })
        }
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let detail = self.details(&manga.key)?;
        if needs_chapters {
            manga.chapters = Some(self.chapters(&detail, &manga.key));
        }
        if needs_details {
            let chapters = manga.chapters.take();
            manga.copy_from(detail.book.info.into_manga());
            manga.chapters = chapters;
        }
        Ok(manga)
    }

    fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let count = if chapter.key == "0" {
            self.details(&manga.key)?.book.info.Len
        } else {
            let response: models::ReaderResponse =
                net::get(&format!("/v4/book/detail/{}/{}", manga.key, chapter.key))?;
            if response.data.adult {
                bail!("该章节不是全年龄内容");
            }
            response.chapter.current.count
        };
        let prefix = if chapter.key == "0" {
            format!("{IMAGE_URL}/{}", manga.key)
        } else {
            format!("{IMAGE_URL}/{}/{}", manga.key, chapter.key)
        };
        Ok((1..=count)
            .map(|page| Page {
                content: PageContent::url(format!("{prefix}/{page}.webp")),
                ..Default::default()
            })
            .collect())
    }
}

impl ListingProvider for NoyMangaSafe {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.id.as_str() {
            "read-day" => self.listing("/readLeaderboard", "day", page),
            "read-week" => self.listing("/readLeaderboard", "week", page),
            "read-month" => self.listing("/readLeaderboard", "moon", page),
            "fav-day" => self.listing("/favLeaderboard", "day", page),
            "fav-week" => self.listing("/favLeaderboard", "week", page),
            "fav-month" => self.listing("/favLeaderboard", "moon", page),
            "quality" => self.listing("/proportion", "", page),
            _ => Err(error!("未知列表")),
        }
    }
}

impl ImageRequestProvider for NoyMangaSafe {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("Referer", WEB_URL)
            .header("User-Agent", USER_AGENT))
    }
}

impl BasicLoginHandler for NoyMangaSafe {
    fn handle_basic_login(&self, key: String, username: String, password: String) -> Result<bool> {
        if key != "login" || username.is_empty() || password.is_empty() {
            return Ok(false);
        }
        if net::login(&username, &password)? {
            net::set_credentials(&username, &password);
            net::mark_just_logged_in();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotificationHandler for NoyMangaSafe {
    fn handle_notification(&self, notification: String) {
        if notification == "login" && !net::consume_just_logged_in() {
            net::clear_login();
        }
    }
}

impl DeepLinkHandler for NoyMangaSafe {
    fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
        if let Some(id) = url
            .split("/manga/")
            .nth(1)
            .and_then(|value| value.split(['/', '?', '#']).next())
            && !id.is_empty()
            && id.chars().all(|c| c.is_ascii_digit())
        {
            return Ok(Some(DeepLinkResult::Manga { key: id.into() }));
        }
        for marker in ["/reader/", "/read/"] {
            if let Some(rest) = url.split(marker).nth(1) {
                let mut parts = rest.split('/');
                if let (Some(manga_key), Some(key)) = (parts.next(), parts.next())
                    && manga_key.chars().all(|c| c.is_ascii_digit())
                    && key.chars().all(|c| c.is_ascii_digit())
                {
                    return Ok(Some(DeepLinkResult::Chapter {
                        manga_key: manga_key.into(),
                        key: key.into(),
                    }));
                }
            }
        }
        Ok(None)
    }
}

register_source!(
    NoyMangaSafe,
    ListingProvider,
    ImageRequestProvider,
    BasicLoginHandler,
    NotificationHandler,
    DeepLinkHandler
);
