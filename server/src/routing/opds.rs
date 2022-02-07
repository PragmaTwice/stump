extern crate base64;

use crate::database::queries;
use crate::database::queries::series::{
    get_lastest_series, get_series, get_series_by_id_with_media,
};
use crate::fs::{self, media_file};
use crate::opds::feed::OpdsFeed;
use crate::opds::link::{OpdsLink, OpdsLinkRel, OpdsLinkType};
use crate::types::rocket::ImageResponse;
use crate::{
    database::entities::{self, media},
    opds,
    types::rocket::XmlResponse,
    State,
};

// BASE URL: /opds/v1.2

/// A handler for GET /opds/v1.2/catalog. Returns an OPDS catalog as an XML document
#[get("/catalog")]
pub fn catalog(_state: &State) -> XmlResponse {
    // TODO: media from database
    let entries = vec![
        opds::entry::OpdsEntry::new(
            "allseries".to_string(),
            chrono::Utc::now(),
            "All Series".to_string(),
            Some(String::from("Browse by series")),
            None,
            Some(vec![OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::Subsection,
                href: String::from("/opds/v1.2/series"),
            }]),
            None,
        ),
        opds::entry::OpdsEntry::new(
            "latestseries".to_string(),
            chrono::Utc::now(),
            "Latest Series".to_string(),
            Some(String::from("Browse latest series")),
            None,
            Some(vec![OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::Subsection,
                href: String::from("/opds/v1.2/series/latest"),
            }]),
            None,
        ),
        // TODO: books/latest
        // TODO: libraries
    ];

    let feed = opds::feed::OpdsFeed::new(
        "root".to_string(),
        "Stump OPDS catalog".to_string(),
        None,
        entries,
    );

    XmlResponse(feed.build().unwrap())
}

#[get("/libraries")]
pub async fn libraries(state: &State) -> Result<XmlResponse, String> {
    let conn = state.get_connection();

    let libraries = queries::library::get_libraries(&conn).await?;

    let entries = libraries
        .into_iter()
        .map(|l| opds::entry::OpdsEntry::from(l))
        .collect();

    let feed = opds::feed::OpdsFeed::new(
        "allLibraries".to_string(),
        "All libraries".to_string(),
        Some(vec![
            OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::ItSelf,
                href: String::from("/opds/v1.2/libraries"),
            },
            OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::Start,
                href: String::from("/opds/v1.2/catalog"),
            },
        ]),
        entries,
    );

    // FIXME: change unsafe unwrap
    Ok(XmlResponse(feed.build().unwrap()))
}

#[get("/libraries/<id>")]
pub async fn library_by_id(state: &State, id: String) -> Result<XmlResponse, String> {
    let res = queries::library::get_library_by_id_with_series(state.get_connection(), id).await?;

    if res.len() != 1 {
        return Err("Library not found".to_string());
    }

    let library_with_series = res[0].to_owned();

    let feed = OpdsFeed::from(library_with_series);

    Ok(XmlResponse(feed.build().unwrap()))
}

/// A handler for GET /opds/v1.2/series
#[get("/series")]
pub async fn series(state: &State) -> Result<XmlResponse, String> {
    let res = get_series(state.get_connection()).await?;

    let entries = res
        .into_iter()
        .map(|s| opds::entry::OpdsEntry::from(s))
        .collect();

    let feed = opds::feed::OpdsFeed::new(
        "root".to_string(),
        "Stump OPDS All Series".to_string(),
        Some(vec![
            OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::ItSelf,
                href: String::from("/opds/v1.2/series"),
            },
            OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::Start,
                href: String::from("/opds/v1.2/catalog"),
            },
        ]),
        entries,
    );

    Ok(XmlResponse(feed.build().unwrap()))
}

#[get("/series/latest")]
pub async fn series_latest(state: &State) -> Result<XmlResponse, String> {
    let res = get_lastest_series(state.get_connection()).await?;

    let entries = res
        .into_iter()
        .map(|s| opds::entry::OpdsEntry::from(s))
        .collect();

    let feed = opds::feed::OpdsFeed::new(
        "root".to_string(),
        "Stump OPDS All Series".to_string(),
        Some(vec![
            OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::ItSelf,
                href: String::from("/opds/v1.2/series/latest"),
            },
            OpdsLink {
                link_type: OpdsLinkType::Navigation,
                rel: OpdsLinkRel::Start,
                href: String::from("/opds/v1.2/catalog"),
            },
        ]),
        entries,
    );

    Ok(XmlResponse(feed.build().unwrap()))
}

#[get("/series/<id>?<page>")]
pub async fn series_by_id(
    id: String,
    page: Option<usize>,
    state: &State,
) -> Result<XmlResponse, String> {
    let res = get_series_by_id_with_media(state.get_connection(), id).await?;

    // TODO: see comment for function `get_series_by_id_with_media`

    if res.len() != 1 {
        return Err("Series not found".to_string());
    }

    let (series, mut media) = res[0].to_owned();

    // page size is 20
    // take a slice of the media vector representing page
    let corrected_page = page.unwrap_or(0);
    let page_size = 20;
    let start = corrected_page * page_size;
    let end = (start + page_size) - 1;

    let mut next_page = None;

    if start > media.len() {
        media = vec![];
    } else if end < media.len() {
        next_page = Some(page.unwrap_or(1));
        media = media.get(start..end).ok_or("Invalid page")?.to_vec();
    }

    // FIXME: this type is kinda disgusting. Consider refactoring to a struct?
    let payload = ((series, media), (page.unwrap_or(0), next_page));

    let feed = OpdsFeed::from(payload);

    Ok(XmlResponse(feed.build().unwrap()))
}

#[get("/books/<id>/thumbnail")]
pub async fn book_thumbnail(id: String, state: &State) -> Result<ImageResponse, String> {
    let book = queries::book::get_book_by_id(state.get_connection(), id).await?;

    if let Some(b) = book {
        match media_file::get_page(&b.path, 1) {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    } else {
        Err("Book not found".to_string())
    }
}

// TODO: generalize the function call
// TODO: cache this? Look into this, I can send a cache-control header to the client, but not sure if I should
// also cache on server. Check my types::rocket crate
#[get("/books/<id>/pages/<page>?<zero_based>")]
pub async fn book_page(
    id: String,
    page: usize,
    zero_based: Option<bool>,
    state: &State,
) -> Result<ImageResponse, String> {
    let book = queries::book::get_book_by_id(state.get_connection(), id).await?;

    let mut correct_page = match zero_based {
        Some(true) => page + 1,
        _ => page,
    };

    if let Some(b) = book {
        // TODO: move this elsewhere?? Doing this to load the cover photo instead of page 1. Not ideal solution
        if b.path.ends_with(".epub") && correct_page == 1 {
            correct_page = 0;
        }
        // match media_file::get_page(&b.path, correct_page) {
        match media_file::get_page(&b.path, correct_page) {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    } else {
        Err("Book not found".to_string())
    }
}
