use axum::routing::get;
use axum::Router;
use askama::Template;
use axum::extract::{State, Path};
use axum::response::{IntoResponse, Response, Html, };
use axum_extra::response::Css;


static WIKI_CSS : &str = include_str!("../css/wiki-style.css");

async fn wiki_page(State(site): State<crate::site::Site>, Path(slug): Path<String>) -> Response {
    // Find the note by slug
    let note = site.site_notes
        .iter()
        .find(|note| note.slug == slug);

    // Return note content or error message if not found
    match note {
        Some(note) => {
            let page = crate::templates::WikiPage { site: site.clone(), note: note.clone() };
            let html = Html(page.render().unwrap()); 
            html.into_response()
        }
        None => {
            "Note not found".into_response()
        }
    }
}

pub fn static_router() -> Router<crate::site::Site> {
    // router for static files
    Router::new()
        .route("/css/wiki-style.css", get(|| async {
            Css(WIKI_CSS).into_response()
        }))
}


pub fn wiki_router(site: crate::site::Site) -> Router {
    let static_router = static_router(); 
    Router::new()
        .route("/{slug}", get(wiki_page))
        .route("/", get(|| async { axum::response::Redirect::to("/index") }))
        .merge(static_router)
        .with_state(site)
        
}

