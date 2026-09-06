use aidoku::{
    ContentRating, Manga, MangaStatus, Viewer,
    alloc::{String, Vec, format, string::ToString, vec},
};
use serde::Deserialize;

use crate::{IMAGE_URL, WEB_URL};

#[derive(Deserialize)]
pub struct ListResponse {
    #[serde(default)]
    pub info: Vec<Book>,
    #[serde(default)]
    pub len: i64,
}

#[derive(Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct Book {
    pub Bid: i64,
    #[serde(default)]
    pub Mode: i32,
    #[serde(default)]
    pub Bookname: String,
    #[serde(default)]
    pub Description: String,
    #[serde(default)]
    pub Author: String,
    #[serde(default)]
    pub Pname: String,
    #[serde(default)]
    pub Ptag: String,
    #[serde(default)]
    pub Otag: String,
    #[serde(default)]
    pub Adult: i32,
    #[serde(default)]
    pub Status: i32,
    #[serde(default)]
    pub Len: i32,
}

impl Book {
    pub fn is_safe(&self) -> bool {
        self.Adult == 0
    }

    pub fn into_manga(self) -> Manga {
        let key = self.Bid.to_string();
        let mut tags = Vec::new();
        for values in [&self.Ptag, &self.Pname, &self.Otag] {
            for value in values.split(' ') {
                if !value.is_empty() && !tags.iter().any(|old| old == value) {
                    tags.push(value.to_string());
                }
            }
        }
        Manga {
            key: key.clone(),
            title: self.Bookname,
            cover: Some(format!("{IMAGE_URL}/{key}/m1.webp")),
            authors: if self.Author.is_empty() {
                None
            } else {
                Some(vec![self.Author])
            },
            description: if self.Description.is_empty() {
                None
            } else {
                Some(self.Description)
            },
            url: Some(format!("{WEB_URL}/manga/{key}")),
            tags: if tags.is_empty() { None } else { Some(tags) },
            status: if self.Status == 1 {
                MangaStatus::Completed
            } else {
                MangaStatus::Ongoing
            },
            content_rating: ContentRating::Safe,
            viewer: Viewer::RightToLeft,
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub data: Vec<SearchBook>,
}

#[derive(Deserialize)]
pub struct SearchBook {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pname: Vec<String>,
    #[serde(default)]
    pub otag: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub adult: i32,
    #[serde(default)]
    pub status: i32,
}

impl SearchBook {
    pub fn is_safe_manga(&self) -> bool {
        self.adult == 0 && self.source != "stream"
    }

    pub fn into_manga(self) -> Manga {
        let key = self.id.to_string();
        let mut tags = Vec::new();
        for value in self.tags.into_iter().chain(self.pname).chain(self.otag) {
            if !value.is_empty() && !tags.contains(&value) {
                tags.push(value);
            }
        }
        Manga {
            key: key.clone(),
            title: self.name,
            cover: Some(format!("{IMAGE_URL}/{key}/m1.webp")),
            authors: if self.author.is_empty() {
                None
            } else {
                Some(vec![self.author])
            },
            description: if self.description.is_empty() {
                None
            } else {
                Some(self.description)
            },
            url: Some(format!("{WEB_URL}/manga/{key}")),
            tags: if tags.is_empty() { None } else { Some(tags) },
            status: if self.status == 1 {
                MangaStatus::Completed
            } else {
                MangaStatus::Ongoing
            },
            content_rating: ContentRating::Safe,
            viewer: Viewer::RightToLeft,
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
pub struct DetailResponse {
    pub book: BookSection,
    pub chapters: ChapterSection,
}

#[derive(Deserialize)]
pub struct BookSection {
    pub info: Book,
}

#[derive(Deserialize)]
pub struct ChapterSection {
    #[serde(default)]
    pub categories: Vec<ChapterCategory>,
    #[serde(default)]
    pub data: alloc::collections::BTreeMap<String, Vec<ChapterInfo>>,
}

#[derive(Deserialize)]
pub struct ChapterCategory {
    pub id: i64,
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Deserialize)]
pub struct ChapterInfo {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub sort: i32,
    #[serde(default)]
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct ReaderResponse {
    pub chapter: ReaderChapter,
    pub data: ReaderData,
}

#[derive(Deserialize)]
pub struct ReaderChapter {
    #[serde(rename = "this")]
    pub current: ChapterInfo,
}

#[derive(Deserialize)]
pub struct ReaderData {
    #[serde(default)]
    pub adult: bool,
}
