use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

pub struct QueryParser {
    pub index: usize,
    pub source: Vec<char>,
}

impl QueryParser {
    pub fn new(source: &str) -> Self {
        Self {
            index: 0,
            source: source.chars().collect(),
        }
    }

    pub fn peek(&self) -> char {
        self.source[self.index]
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    pub fn is_at_end(&self) -> bool {
        self.index >= self.source.len()
    }

    pub fn is_not_at_end(&self) -> bool {
        !self.is_at_end()
    }
}

pub fn parse_query(query: &str) -> Vec<String> {
    let mut parser = QueryParser::new(query);
    let mut parsed_items = Vec::new();

    while parser.is_not_at_end() {
        if parser.peek().is_whitespace() {
            parser.advance();
        } else if parser.peek() == '"' {
            // skip '"'
            parser.advance();

            let mut inside_quotes = String::new();

            while parser.is_not_at_end() && parser.peek() != '"' {
                inside_quotes.push(parser.peek());
                parser.advance();
            }

            let terms = parse_query(&inside_quotes.clone());
            parsed_items.extend(terms);
            parsed_items.push(inside_quotes);

            // skip '"'
            parser.advance();
        } else {
            let mut term = String::new();

            while parser.is_not_at_end() && !parser.peek().is_whitespace() {
                term.push(parser.peek());
                parser.advance();
            }

            parsed_items.push(term);

            // skip ' '
            parser.advance();
        }
    }

    parsed_items
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

#[cfg(test)]
mod tests {
    use super::parse_query;

    #[test]
    fn test_basic_query() {
        let query = String::from("foo bar bee");
        let results = parse_query(&query);
        assert!(results.len() > 0);

        assert_eq!(results.get(0), Some(&String::from("foo")));
        assert_eq!(results.get(1), Some(&String::from("bar")));
        assert_eq!(results.get(2), Some(&String::from("bee")));

        let query = String::from("??? --- ### foo bar $$$ 😸😸😸");
        let results = parse_query(&query);
        assert!(results.len() > 0);

        assert!(results.contains(&String::from("???")));
        assert!(results.contains(&String::from("foo")));
        assert!(results.contains(&String::from("bar")));
        assert!(results.contains(&String::from("😸😸😸")));
    }

    #[test]
    fn test_quote_query() {
        let query = String::from(r#"   "foo bar"   "fizz buzz"   "#);
        let results = parse_query(&query);
        assert!(results.len() > 0);

        assert!(results.contains(&String::from("foo")));
        assert!(results.contains(&String::from("bar")));
        assert!(results.contains(&String::from("foo bar")));

        assert!(results.contains(&String::from("fizz")));
        assert!(results.contains(&String::from("buzz")));
        assert!(results.contains(&String::from("fizz buzz")));
    }

    #[test]
    fn test_mixed_query() {
        let query = String::from(r#"foo bar "wire shark" "super user""#);
        let results = parse_query(&query);

        assert!(results.len() > 0);
        assert!(results.contains(&String::from("foo")));
        assert!(results.contains(&String::from("wire")));
        assert!(results.contains(&String::from("shark")));
        assert!(results.contains(&String::from("wire shark")));
        assert!(results.contains(&String::from("super")));
        assert!(results.contains(&String::from("user")));
        assert!(results.contains(&String::from("super user")));
    }
}
