use std::collections::BTreeSet;

use eframe::egui;
use eframe::egui::widgets::*;
use egui_notify::{Toasts};

use crate::article_parser;
use crate::article_parser::{WikiArticle, Token, Section};
use crate::wikipedia_api;

struct App {
    selected_language: String,
    wiki_article: Option<WikiArticle>,
    guesses: BTreeSet<String>,

    title_text_box: String,
    toasts: Toasts,
    next_guess: String,
    focus_on_guess: bool,
}

impl App {
    fn load_article(&mut self) {
        let downloaded = wikipedia_api::download_article(
            self.selected_language.as_str(),
            self.title_text_box.as_str());

        match downloaded {
            Ok((title, content)) => {
                self.wiki_article = Some(article_parser::parse(title.as_str(), content.as_str()));
                self.guesses.clear();
                self.next_guess.clear();
                self.title_text_box.clear();
            }

            Err(e) => {
                self.toasts.error(format!("{}", e));
            }
        }
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

    fn get_word(&self, word: &str) -> String {
        if self.guesses.contains(&word.to_lowercase()) || self.title_complete() {
            String::from(word)
        } else {
            let dashes: Vec<&str> = std::iter::repeat("_").take(word.len()).collect();
            dashes.concat()
        }
    }

    fn concat_tokens(&self, tokens: &Vec<Token>) -> String {
        let mut result = String::new();

        for token in tokens {
            match token {
                Token::Word(w) => {
                    // result.push_str(w);
                    result.push_str(self.get_word(w).as_str());
                }
                Token::NonWord(w) => {
                    result.push_str(w);
                }
            }
        }

        result
    }

    fn show_top_bar(&mut self, ui: &mut egui::Ui) {
            ui.horizontal(|ui| {
                ui.label("Language code:");

                let language_code = TextEdit::singleline(&mut self.selected_language)
                    .desired_width(30.0);
                ui.add(language_code);

                ui.label("Article:");

                let title_text_box = TextEdit::singleline(&mut self.title_text_box);
                let resp = ui.add(title_text_box);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.load_article();
                }

                let load_btn = ui.button("load");

                if load_btn.clicked() {
                    self.load_article();
                }
            });
    }

    fn show_title(&self, ui: &mut egui::Ui, tokens: &Vec<Token>) {
        let text = self.concat_tokens(&tokens);
        ui.label(egui::RichText::new(text).heading().monospace());
        ui.add_space(30.0);
    }

    fn show_sections(&self, ui: &mut egui::Ui, sections: &Vec<Section>) {
        for section in sections {
            match section {
                Section::Heading(_level, tokens) => {
                    let text = self.concat_tokens(&tokens);
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new(text).heading().monospace());
                    ui.add_space(10.0);
                }

                Section::Paragraph(tokens) => {
                    let text = self.concat_tokens(&tokens);
                    ui.monospace(text);
                    ui.add_space(10.0);
                }

                Section::UnorderedList(list_items) => {
                    for item in list_items {
                        ui.horizontal_top(|ui| {
                            ui.label("•");

                            ui.vertical(|ui| {
                                self.show_sections(ui, item);
                            });
                        });
                    }
                }

                Section::OrderedList(list_items) => {
                    for (i, item) in list_items.iter().enumerate() {
                        ui.horizontal_top(|ui| {
                            ui.label(format!("{}.", i + 1));

                            ui.vertical(|ui| {
                                self.show_sections(ui, item);
                            });
                        });
                    }
                }
            }
        }
    }

    fn show_article(&self, ui: &mut egui::Ui) {
        if let Some(wiki_article) = &self.wiki_article {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_title(ui, &wiki_article.title);

                self.show_sections(ui, &wiki_article.content);
            });
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

    fn count_word_in_article(&self, word: &str) -> usize {
        let Some(wiki_article) = &self.wiki_article else {
            panic!("count_word_in_article called without article present");
        };

        Self::count_word_in_tokens(word, &wiki_article.title)
            + Self::count_word_in_sections(word, &wiki_article.content)

    }

    fn show_guesses(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            egui::Grid::new("guesses_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for guess in &self.guesses {
                        let occurs = self.count_word_in_article(guess.as_str());

                        ui.label(format!("{}", occurs));
                        ui.label(guess);
                        ui.end_row();
                    }

                    ui.label("");

                    let next_guess_edit = TextEdit::singleline(&mut self.next_guess);
                    let resp = ui.add(next_guess_edit);

                    if self.focus_on_guess {
                        resp.request_focus();
                        self.focus_on_guess = false;
                    }

                    ui.end_row();

                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.next_guess.is_empty() {
                            self.guesses.insert(self.next_guess.to_lowercase());
                            self.next_guess.clear();
                        }
                        self.focus_on_guess = true;
                    }
                });
        });
    }

    fn show_gui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_top_bar(ui);
        });

        if let Some(_) = self.wiki_article {
            egui::SidePanel::right("right_panel")
                .min_width(200.0)
                .resizable(true)
                .show_separator_line(true)
                .show(ctx, |ui| {
                    self.show_guesses(ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_article(ui);
        });

        self.toasts.show(ctx);
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_language: String::from("en"),
            wiki_article: None,
            guesses: BTreeSet::new(),

            toasts: Toasts::new(),
            next_guess: String::from(""),
            focus_on_guess: false,
            title_text_box: String::from(""),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.show_gui(ctx, frame);
    }
}

pub fn launch() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "rsdctl",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )
}
