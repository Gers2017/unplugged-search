use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, get_service};
use axum::{Router, Server};
use log::{debug, info};
use tower_http::services::{ServeDir, ServeFile};

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tera::{Context, Tera};
use unplugged_engine::{
    get_episodes_from_ids, load_common_words, parse_episodes_by_id, parse_episodes_by_tag,
    parse_query, Episode, EpisodesById, EpisodesByTag, ParseResult,
};

pub fn compile_templates() -> Tera {
    Tera::new("templates/**/*.html").expect("Error at compiling templates")
}

#[derive(Clone)]
pub struct AppState {
    pub episodes_by_tag: EpisodesByTag,
    pub episodes_by_id: EpisodesById,
    pub common_words: HashSet<String>,
    pub tera: Tera,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let episodes_by_tag = parse_episodes_by_tag().await;
    let episodes_by_id = parse_episodes_by_id().await;
    let common_words: HashSet<_> = load_common_words();

    let tera = compile_templates();

    let serve_dir = ServeDir::new("static");

    let app = Router::new()
        .route("/", get_service(ServeFile::new("static/index.html")))
        .route("/search", get(handle_search)) // search?query=foo
        .fallback_service(serve_dir)
        .with_state(Arc::new(AppState {
            episodes_by_tag,
            episodes_by_id,
            common_words,
            tera,
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!(
        "Web server listening on {} (http://localhost:{})",
        addr,
        addr.port()
    );

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
    let mut results: HashSet<Episode> = HashSet::new();

    let ParseResult { terms, exclude } = parse_query(&query);

    let terms: HashSet<_> = terms
        .iter()
        .map(|s| s.to_lowercase())
        .filter(|s| !state.common_words.contains(s))
        .collect();

    let exclude: HashSet<_> = HashSet::from_iter(exclude.into_iter());

    let episodes_by_tag: HashMap<String, Vec<&Episode>> = state
        .episodes_by_tag
        .iter()
        .map(|(tag, ids)| (tag, get_episodes_from_ids(ids, &state.episodes_by_id)))
        .fold(HashMap::new(), |mut acc, (tag, episodes)| {
            acc.insert(tag.to_string(), episodes);
            acc
        });

    for (tag, episodes) in episodes_by_tag.iter() {
        if terms
            .iter()
            .any(|term| tag.contains(term) || term.contains(tag))
        {
            results.extend(episodes.iter().map(|episode| (**episode).clone()));
        }
    }

    for (id, episode) in state.episodes_by_id.iter() {
        // skip episode already seen
        if results.contains(episode) {
            continue;
        }

        // if any of the search terms matches a word in the title
        let episode_id = id.to_string();

        if terms.contains(&episode_id)
            || terms
                .iter()
                .any(|term| episode.title.to_lowercase().contains(term))
        {
            results.insert(episode.clone());
        }
    }

    // filtering the results

    if !exclude.is_empty() {
        results = results
            .into_iter()
            .filter(|episode| {
                !episode
                    .tags
                    .iter()
                    .any(|tag| exclude.iter().any(|excl_token| tag.contains(excl_token)))
            })
            .collect();
    }

    // sorting results

    let mut results_with_score: Vec<_> = results
        .iter()
        .map(|episode| {
            let mut score = episode.tags.iter().fold(0, |acc, tag| {
                // scores for tag
                acc + if terms.contains(tag) || terms.iter().any(|term| tag.contains(term)) {
                    50
                } else {
                    0
                }
            });

            // scores for title
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

    results_with_score.sort_by(|(a_score, _), (b_score, _)| b_score.cmp(a_score));

    debug!(
        "Query: {}, Search terms: {:?}, Exclude: {:?}",
        &search.query, &terms, &exclude
    );

    debug!("score  | title");
    debug!("{}+{}", "_".repeat(7), "_".repeat(8));
    for (score, ep) in &results_with_score[..] {
        debug!("{0:>4}   |  {1}", score, ep.title);
    }
    debug!("{}", "-------".repeat(3));

    let search_results: Vec<_> = results_with_score.iter().map(|(_, ep)| *ep).collect();

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
