use std::sync::Arc;

use eframe::egui;
use eframe::egui::text::Galley;
use eframe::egui::widgets::*;
use eframe::epaint::{Color32, text::{LayoutJob, TextFormat}};
use egui_notify::{Toasts};

use crate::article_parser::{Token, Section};
use crate::game::{Game, TokenTreatment};

struct App {
    game: Game,

    selected_language: String,

    title_text_box: String,
    toasts: Toasts,
    next_guess: String,
    focus_on_guess: bool,
}

impl App {
    fn load_article(&mut self) {
        let res = self.game.load_article(
            self.selected_language.as_str(),
            self.title_text_box.as_str());

        match res {
            Ok(()) => {
                self.next_guess.clear();
                self.title_text_box.clear();
            }

            Err(e) => {
                self.toasts.error(format!("{}", e));
            }
        }
    }

    fn load_random_article(&mut self) {
        let res = self.game.load_random_article();

        match res {
            Ok(()) => {
                self.selected_language = String::from("en");
            }

            Err(e) => {
                self.toasts.error(format!("{}", e));
            }
        }
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

                let random_btn = ui.button("random");

                if random_btn.clicked() {
                    self.load_random_article();
                }
            });
    }

    fn add_token(&self, job: &mut LayoutJob, format: &TextFormat, token: &Token) {
        match self.game.get_token_treatment(token) {
            TokenTreatment::Blank => {
                let dashes: Vec<&str> = std::iter::repeat("_").take(token.char_count()).collect();
                let dashes = dashes.concat();
                job.append(&dashes, 0.0, format.clone());
            }

	    TokenTreatment::Show => {
		job.append(token.get_str(), 0.0, format.clone());
	    }

	    TokenTreatment::Highlight => {
		let highlit_format = TextFormat {
                    color: Color32::BLACK,
                    background: Color32::LIGHT_BLUE,
                    ..format.clone()
		};
		job.append(token.get_str(), 0.0, highlit_format);
	    }
        }
    }

    fn render_tokens(&self, ui: &egui::Ui, format: &TextFormat, tokens: &Vec<Token>) -> Arc<Galley> {
        let mut job = LayoutJob::default();
        job.wrap.max_width = ui.available_width();

        for token in tokens {
            self.add_token(&mut job, format, token);
        }

        ui.fonts(|fonts| {
            fonts.layout_job(job)
        })
    }

    fn show_title(&self, ui: &mut egui::Ui, tokens: &Vec<Token>) {
	let mut font = egui::TextStyle::Monospace.resolve(ui.style());
	font.size *= 2.0;

	let title_format = TextFormat {
            font_id: font,
            ..Default::default()
	};

	let galley = self.render_tokens(ui, &title_format, tokens);
        ui.label(galley);
        ui.add_space(30.0);
    }

    fn show_paragraph(&self, ui:  &mut egui::Ui, tokens: &Vec<Token>) {
        let format = TextFormat {
            font_id: egui::TextStyle::Monospace.resolve(ui.style()),
            ..Default::default()
        };

        ui.label(self.render_tokens(ui, &format, tokens));
    }

    fn show_sections(&self, ui: &mut egui::Ui, sections: &Vec<Section>) {
        for section in sections {
            match section {
                Section::Heading(_level, tokens) => {
		    let mut font = egui::TextStyle::Monospace.resolve(ui.style());
		    font.size *= 1.5;

		    let heading_format = TextFormat {
			font_id: font,
			..Default::default()
		    };

                    ui.add_space(30.0);
		    let galley = self.render_tokens(ui, &heading_format, tokens);
                    ui.label(galley);
                    ui.add_space(10.0);
                }

                Section::Paragraph(tokens) => {
                    self.show_paragraph(ui, tokens);
                    ui.add_space(10.0);

                    // let text = self.concat_tokens(&tokens);
                    // ui.label(text);
                    // ui.add_space(10.0);
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
        if let Some(wiki_article) = &self.game.wiki_article {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_title(ui, &wiki_article.title);

                self.show_sections(ui, &wiki_article.content);
            });
        }
    }

    fn show_guesses(&mut self, ui: &mut egui::Ui) {

        let next_guess_edit = TextEdit::singleline(&mut self.next_guess);
        let resp = ui.add(next_guess_edit);

        if self.focus_on_guess {
            resp.request_focus();
            self.focus_on_guess = false;
        }

        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !self.next_guess.is_empty() {
                self.game.guess(&self.next_guess);
                self.next_guess.clear();
            }
            self.focus_on_guess = true;
        }

        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            egui::Grid::new("guesses_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for guess in &self.game.guesses {
                        let occurs = self.game.count_word_in_article(guess.as_str()).unwrap();

                        ui.label(format!("{}", occurs));

                        let is_guess_selected = *guess == self.game.selected_guess;
                        if ui.selectable_label(is_guess_selected, guess).clicked() {
                            if is_guess_selected {
                                self.game.selected_guess = String::from("");
                            } else {
                                self.game.selected_guess = guess.clone();
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    }

    fn show_gui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.0);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_top_bar(ui);
        });

        if let Some(_) = self.game.wiki_article {
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
            game: Game::new(),

            selected_language: String::from("en"),

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
