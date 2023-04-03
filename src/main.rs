use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, get_service};
use axum::{Router, Server};
use tower_http::services::{ServeDir, ServeFile};

use serde::Deserialize;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tera::{Context, Tera};
use unplugged_engine::{
    get_episodes_from_ids, load_common_words, parse_episodes_by_id, parse_episodes_by_tag,
    parse_query, parse_tag_by_tags, Episode, EpisodesById, EpisodesByTag, TagsByTag,
};

pub fn compile_templates() -> Tera {
    let tera = Tera::new("templates/**/*.html").expect("Error at compiling templates");
    tera
}

#[derive(Clone)]
pub struct AppState {
    pub episodes_by_tag: EpisodesByTag,
    pub episodes_by_id: EpisodesById,
    pub tags_by_tag: TagsByTag,
    pub common_words: HashSet<String>,
    pub tera: Tera,
}

const DEBUG: bool = false;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let episodes_by_tag = parse_episodes_by_tag().await;
    let episodes_by_id = parse_episodes_by_id().await;
    let tags_by_tag = parse_tag_by_tags().await;
    let common_words: HashSet<_> = load_common_words();

    let tera = compile_templates();

    let serve_dir = ServeDir::new("static");

    let app = Router::new()
        // bellow maybe serve a svelte site?
        .route("/", get_service(ServeFile::new("static/index.html")))
        .route("/search", get(handle_search)) // search?query=foo
        .fallback_service(serve_dir)
        .with_state(Arc::new(AppState {
            episodes_by_tag,
            episodes_by_id,
            tags_by_tag,
            common_words,
            tera,
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

async fn handle_search(
    search: Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let query = search.query.clone();
    let mut search_results: HashSet<Episode> = HashSet::new();

    let mut terms: HashSet<_> = query
        .split_whitespace()
        .map(|x| x.trim().to_lowercase())
        .map(|x| {
            if x.contains("\"") {
                x.replace("\"", "")
            } else {
                x
            }
        })
        .filter(|x| !state.common_words.contains(x))
        .collect();

    let parsed_terms = parse_query(&query);
    terms.extend(parsed_terms);

    let episodes_by_tag: Vec<_> = state
        .episodes_by_tag
        .iter()
        .map(|(tag, ids)| (tag, get_episodes_from_ids(ids, &state.episodes_by_id)))
        .collect();

    for (tag, episodes) in episodes_by_tag.iter() {
        if terms
            .iter()
            .any(|term| tag.contains(term) || term.contains(*tag))
        {
            search_results.extend(episodes.iter().map(|episode| (**episode).clone()));
        }
    }

    for (id, episode) in state.episodes_by_id.iter() {
        // if any of the search terms matches a word in the title
        let episode_id = id.to_string();

        if terms.contains(&episode_id)
            || terms
                .iter()
                .any(|term| episode.title.to_lowercase().contains(term))
        {
            search_results.insert(episode.clone());
        }
    }

    // sorting results

    let mut search_results_with_score: Vec<_> = search_results
        .iter()
        .map(|episode| {
            let mut score = episode
                .tags
                .iter()
                .fold(0, |acc, tag| acc + if terms.contains(tag) { 50 } else { 0 });

            score += terms.iter().fold(0, |acc, term| {
                acc + if episode.title.to_lowercase().contains(term) {
                    100
                } else {
                    0
                }
            });

            (score, episode)
        })
        .collect();

    search_results_with_score.sort_by(|(a_score, _), (b_score, _)| b_score.cmp(a_score));

    tracing::debug!("query: {:?}", &search.query);
    tracing::debug!("search terms: {:?}", &terms);

    if DEBUG {
        println!("{}", "-------".repeat(3));
        println!("Query: {}", &search.query);
        println!("Search terms: {:?}", &terms);

        println!("score  |  title");
        for (score, ep) in &search_results_with_score[..] {
            println!("{}  | {}", score, ep.title);
        }
    }

    let search_results: Vec<_> = search_results_with_score
        .iter()
        .map(|(_, ep)| *ep)
        .collect();

    // reply with a tera template

    let query = &(search.query);
    let episodes = search_results;

    let html = state
        .tera
        .render(
            "results.html",
            &Context::from_serialize(&serde_json::json!({ "episodes": episodes, "query": query }))
                .unwrap(),
        )
        .unwrap();

    Html(html)
}
