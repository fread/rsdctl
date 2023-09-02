use dioxus::prelude::*;

use crate::article_parser::{Token, Section};
use crate::game::{Game, TokenTreatment};

pub fn launch() {
    dioxus_desktop::launch(app);
}

struct App {
    game: Game,
}

#[inline_props]
fn Title(cx:Scope, tokens: Vec<Token>) -> Element {
    cx.render(rsx! {
	h1 {
	    for token in tokens {
		token.get_str()
	    }
	}
    })
}

#[inline_props]
fn ArticleParagraph(cx: Scope, tokens: Vec<Token>) -> Element {
    cx.render(rsx!(
	p {
	    for token in tokens {
		token.get_str()
	    }
	}
    ))
}

#[inline_props]
fn ArticleSection(cx: Scope, section: Section) -> Element {
    match section {
	Section::Paragraph(tokens) => {
	    cx.render(rsx!(ArticleParagraph { tokens: tokens.clone() }))
	}
	_ => {
	    cx.render(rsx!(p { "???" }))
	}
    }
}

#[inline_props]
fn Article(cx: Scope, sections: Vec<Section>) -> Element {
    cx.render(rsx! {
	for section in sections {
	    rsx!(
		ArticleSection { section: section.clone() }
	    )
	}
    })
}

fn app(cx: Scope) -> Element {
    let game = use_ref(cx, || Game::new());

    cx.render(rsx! (
	button {
	    onclick: move |_| {
		game.write().load_article("en", "Rust (programming language)").unwrap();
		println!("Loaded");
            },

	    "Example game"
	},

	if let Some(wiki_article) = &game.read().wiki_article {
	    rsx!(
		Title { tokens: wiki_article.title.clone() },

		Article { sections: wiki_article.content.clone() }
	    )
	} else {
	    rsx!(
		div { "Nothing to see" },
	    )
	}
    ))
}
