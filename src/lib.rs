use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
mod parser;
pub use parser::*;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Episode {
    pub id: i64,
    pub title: String,
    pub date: String,
    pub duration: String,
    pub tags: Vec<String>,
    pub url: String,
}

impl Into<String> for Episode {
    fn into(self) -> String {
        serde_json::to_string_pretty(&self).expect("Error at stringify json")
    }
}

const EPISODES_BY_ID_FILE: &str = "episodes_by_id_index.json";
const EPISODES_BY_TAG_FILE: &str = "episodes_by_tag_index.json";

pub type EpisodesById = HashMap<usize, Episode>;
pub type EpisodesByTag = HashMap<String, Vec<usize>>;

pub async fn parse_episodes_by_tag() -> EpisodesByTag {
    parse_json_file::<EpisodesByTag>(EPISODES_BY_TAG_FILE).await
}

pub async fn parse_episodes_by_id() -> EpisodesById {
    parse_json_file::<EpisodesById>(EPISODES_BY_ID_FILE).await
}

pub fn get_episodes_from_ids<'a>(ids: &[usize], by_id: &'a EpisodesById) -> Vec<&'a Episode> {
    ids.iter()
        .map(|id| by_id.get(id).expect("Error at getting episode by id"))
        .collect()
}

pub async fn parse_json_file<T>(file: &str) -> T
where
    T: DeserializeOwned,
{
    let contents = tokio::fs::read_to_string(file)
        .await
        .expect(&format!("Error at parsing {} file to json", &file));

    serde_json::from_str::<T>(&contents).expect("Error at parsing to json file")
}

pub fn load_common_words() -> HashSet<String> {
    let common_words = [
        "the", "be", "is", "are", "to", "of", "and", "a", "an", "in", "that", "have", "i", "it",
        "for", "not", "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by",
        "from", "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would",
        "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which", "me",
        "when", "make", "can", "like", "time", "no", "just", "him", "know", "take", "people",
        "into", "year", "your", "some", "could", "them", "see", "other", "than", "then", "now",
        "look", "only", "come", "its", "it's", "over", "think", "also", "back", "after", "use",
        "two", "how", "our", "work", "first", "well", "way", "even", "new", "want", "because",
        "any", "these", "give", "day", "most", "us", "was", "from",
    ];

    common_words.iter().map(|s| s.to_string()).collect()
}
