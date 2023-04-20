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

    pub fn is_end(&self) -> bool {
        self.index >= self.source.len()
    }

    pub fn is_not_end(&self) -> bool {
        self.index < self.source.len()
    }

    pub fn peek(&self) -> char {
        self.source[self.index]
    }

    pub fn peek_previous(&self) -> Option<char> {
        self.source.get(self.index - 1).copied()
    }

    pub fn peek_next(&self) -> Option<char> {
        self.source.get(self.index + 1).copied()
    }

    pub fn is_whitespace(&self) -> bool {
        self.peek().is_whitespace()
    }

    pub fn is_not_whitespace(&self) -> bool {
        !self.peek().is_whitespace()
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    // skips whitespace, advance to next char
    pub fn trim_left(&mut self) {
        while self.is_not_end() && self.is_whitespace() {
            self.advance();
        }
    }

    pub fn trim_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while self.is_not_end() && predicate(self.peek()) {
            self.advance();
        }
    }

    // saves the current char, advance to next char
    // returns the saved char
    pub fn peek_advance(&mut self) -> char {
        let ch = self.peek();
        self.advance();
        ch
    }

    pub fn get_token(&mut self) -> Option<String> {
        let mut token = String::new();

        if self.is_not_end() && self.is_whitespace() {
            self.trim_left();
        }

        if self.is_end() {
            return None;
        }

        let ch = self.peek();

        if ch == '-' {
            self.trim_while(|ch| ch == '-');
            if self.peek_next().is_some() {
                return self.get_token().map(|mut s| {
                    s.insert(0, '-');
                    s
                });
            }
        } else if ch == '"' {
            // skip '"'
            self.advance();

            while self.is_not_end() && self.peek() != '"' {
                token.push(self.peek_advance());
            }

            // skip '"'
            self.advance();
        } else if !ch.is_whitespace() {
            // accept all that isn't whitespace
            while self.is_not_end() && self.is_not_whitespace() {
                token.push(self.peek_advance());
            }
        }

        Some(token)
    }

    pub fn parse(&mut self) -> ParseResult {
        let mut terms = Vec::new();
        let mut exclude = Vec::new();

        while let Some(token) = self.get_token() {
            if token.starts_with('-') {
                let exclude_token = token.trim_start_matches('-').to_string();
                if !exclude_token.is_empty() {
                    exclude.push(exclude_token.trim().to_string());
                }
            } else {
                terms.push(token.trim().to_string());
            }
        }

        ParseResult { terms, exclude }
    }
}

pub struct ParseResult {
    pub terms: Vec<String>,
    pub exclude: Vec<String>,
}

pub fn parse_query(query: &str) -> ParseResult {
    let mut parser = QueryParser::new(query);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use crate::ParseResult;

    use super::QueryParser;

    #[test]
    fn test_get_token() {
        let query = String::from("    duck duck go \"spy dog\" bat -cat  ");
        let mut parser = QueryParser::new(&query);

        assert_eq!(parser.get_token(), Some(String::from("duck")));
        assert_eq!(parser.get_token(), Some(String::from("duck")));
        assert_eq!(parser.get_token(), Some(String::from("go")));

        assert_eq!(parser.get_token(), Some(String::from("spy dog")));
        assert_eq!(parser.get_token(), Some(String::from("bat")));
        assert_eq!(parser.get_token(), Some(String::from("-cat")));
        assert_eq!(parser.get_token(), None);
    }

    // cargo test -- parser::tests::test_parse --exact --nocapture
    #[test]
    fn test_parse() {
        let query = String::from(
            "  \"docker compose\"  -dotnet -\"windows server\"  \"chocolate cupcakes\" 😸😸😸 ",
        );

        let mut parser = QueryParser::new(&query);
        let ParseResult { terms, exclude } = parser.parse();

        println!("results\nterms: {:?}\nexclude: {:?}", &terms, &exclude);

        assert!(terms.len() > 0);
        assert!(exclude.len() == 2);

        assert_eq!(terms[0], String::from("docker compose"));
        assert_eq!(terms[1], String::from("chocolate cupcakes"));
        assert_eq!(terms[2], String::from("😸😸😸"));

        assert_eq!(exclude[0], String::from("dotnet"));
        assert_eq!(exclude[1], String::from("windows server"));
    }

    #[test]
    fn test_parse_edge() {
        let query =
            String::from("  -  nixos \"-- kde\"  ----- docker low-memory-monitor  dnf-fedora ");
        //                  ^ counts as exclude    ^ ignore extra '-' and exclude    ^ this is ok

        let mut parser = QueryParser::new(&query);
        let ParseResult { terms, exclude } = parser.parse();

        println!("results\nterms: {:?}\nexclude: {:?}", &terms, &exclude);

        assert!(terms.len() == 2);
        assert!(exclude.len() == 3);

        assert_eq!(terms[0], String::from("low-memory-monitor"));
        assert_eq!(terms[1], String::from("dnf-fedora"));

        assert_eq!(exclude[0], String::from("nixos"));
        assert_eq!(exclude[1], String::from("kde"));
        assert_eq!(exclude[2], String::from("docker"));
    }
}
