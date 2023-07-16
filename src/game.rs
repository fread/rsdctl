use std::collections::BTreeSet;

use anyhow::Result;

use crate::article_parser;
use crate::article_parser::{WikiArticle, Section, Token};
use crate::wikipedia_api;

pub enum TokenTreatment {
    Blank,
    Show,
    Highlight
}

pub struct Game {
    pub wiki_article: Option<WikiArticle>,
    pub guesses: BTreeSet<String>,

    pub selected_guess: String,
}

impl Game {
    pub fn new() -> Self {
	Game {
	    wiki_article: None,
	    guesses: BTreeSet::new(),
	    selected_guess: String::from(""),
	}
    }

    pub fn load_article(&mut self, language: &str, title: &str) -> Result<()> {
	let downloaded = wikipedia_api::download_article(language, title);

	// TODO monad?
        match downloaded {
            Ok((title, content)) => {
                self.wiki_article = Some(article_parser::parse(title.as_str(), content.as_str()));
                self.guesses.clear();
		self.selected_guess.clear();
		Ok(())
            }

            Err(e) => {
		Err(e)
            }
        }
    }

    pub fn load_random_article(&mut self) -> Result<()> {
	let article = wikipedia_api::random_english_article();

	// TODO monad?
	match article {
	    Ok(name) => {
		self.load_article("en", name.as_str())
	    }

	    Err(e) => {
		Err(e)
	    }
	}
    }

    pub fn guess(&mut self, raw_guess: &str) {
	self.guesses.insert(raw_guess.trim().to_lowercase());
    }

    fn title_complete(&self) -> bool {
        if let Some(wiki_article) = &self.wiki_article {
            for token in &wiki_article.title {
                if let Token::Word(w) = token {
                    if !self.guesses.contains(&w.to_lowercase()) {
                        return false;
                    }
                }
            }

            true
        } else {
            false
        }
    }

    pub fn get_token_treatment(&self, token: &Token) -> TokenTreatment {
	match token {
	    Token::Word(word) => {
		if self.selected_guess.to_lowercase() == word.to_lowercase() {
		    TokenTreatment::Highlight
		} else if self.guesses.contains(&word.to_lowercase()) || self.title_complete() {
		    TokenTreatment::Show
		} else {
		    TokenTreatment::Blank
		}
	    }

	    Token::NonWord(_) => {
		TokenTreatment::Show
	    }
        }
    }

    fn count_word_in_tokens(word: &str, tokens: &Vec<Token>) -> usize {
        let mut result = 0;

        for token in tokens {
            if let Token::Word(w) = token {
                if w.to_lowercase() == word.to_lowercase() {
                    result += 1;
                }
            }
        }
        result
    }

    fn count_word_in_sections(word: &str, sections: &Vec<Section>) -> usize {
        let mut result = 0;

        for section in sections {
            match section {
                Section::Heading(_level, tokens) => {
                    result += Self::count_word_in_tokens(word, tokens);
                }

                Section::Paragraph(tokens) => {
                    result += Self::count_word_in_tokens(word, tokens);
                }

                Section::UnorderedList(list_items) => {
                    for item in list_items {
                        result += Self::count_word_in_sections(word, item);
                    }
                }

                Section::OrderedList(list_items) => {
                    for item in list_items {
                        result += Self::count_word_in_sections(word, item);
                    }
                }
            }
        }
        result
    }

    pub fn count_word_in_article(&self, word: &str) -> Option<usize> {
        if let Some(wiki_article) = &self.wiki_article {
            Some(Self::count_word_in_tokens(word, &wiki_article.title)
		 + Self::count_word_in_sections(word, &wiki_article.content))
	} else {
            None
        }
    }
}
