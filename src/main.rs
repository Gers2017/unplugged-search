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
    parse_episodes_by_id, parse_episodes_by_tag, parse_query, Episode, EpisodesById, EpisodesByTag,
};

pub fn compile_templates() -> Tera {
    let tera = Tera::new("templates/**/*.html").expect("Error at compiling templates");
    tera
}

#[derive(Clone)]
pub struct AppState {
    pub episodes_by_tag: EpisodesByTag,
    pub episodes_by_id: EpisodesById,
    pub tera: Tera,
}

const DEBUG: bool = false;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let episodes_by_tag = parse_episodes_by_tag().await;
    let episodes_by_id = parse_episodes_by_id().await;
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

    let common_words = [
        "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on",
        "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we",
        "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their",
        "what", "so", "up", "out", "if", "about", "who", "get", "which", "me", "when", "make",
        "can", "like", "time", "no", "just", "him", "know", "take", "people", "into", "year",
        "your", "some", "could", "them", "see", "other", "than", "then", "now", "look", "only",
        "come", "its", "it's", "over", "think", "also", "back", "after", "use", "two", "how",
        "our", "work", "first", "well", "way", "even", "new", "want", "because", "any", "these",
        "give", "day", "most", "us", "was", "from",
    ];

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
        .filter(|x| !common_words.contains(&x.as_str()))
        .collect();

    let parsed_terms = parse_query(&query);
    terms.extend(parsed_terms);

    for (tag, episodes) in state.episodes_by_tag.iter() {
        if terms
            .iter()
            .any(|term| tag.contains(term) || term.contains(tag))
        {
            search_results.extend(episodes.iter().map(|episode| episode.clone()));
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
