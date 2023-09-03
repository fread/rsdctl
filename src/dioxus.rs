#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::article_parser::{Token, Section};
use crate::game::{Game, TokenTreatment};

pub fn launch() {
    dioxus_desktop::launch(app);
}

#[derive(Props)]
struct HeadingNProps<'a> {
    level: usize,
    children: Element<'a>,
}

fn HeadingN<'a>(cx: Scope<'a, HeadingNProps<'a>>) -> Element {
    match cx.props.level {
	0 => panic!("Invalid heading level 0"),
	1 => cx.render(rsx!( h1 { &cx.props.children })),
	2 => cx.render(rsx!( h2 { &cx.props.children })),
	3 => cx.render(rsx!( h3 { &cx.props.children })),
	4 => cx.render(rsx!( h4 { &cx.props.children })),
	5 => cx.render(rsx!( h5 { &cx.props.children })),
	6 => cx.render(rsx!( h6 { &cx.props.children })),
	_ => cx.render(rsx!( h6 { &cx.props.children })),
    }
}

#[inline_props]
fn Token(cx: Scope, token: Token) -> Element {
    let game = use_shared_state::<Game>(cx).unwrap();

    match game.read().get_token_treatment(token) {
        TokenTreatment::Blank => {
            let dashes: Vec<&str> = std::iter::repeat("_").take(token.char_count()).collect();
            let dashes = dashes.concat();
            cx.render(rsx!(dashes))
        }

	TokenTreatment::Show => {
	    cx.render(rsx!(token.get_str()))
	}

	TokenTreatment::Highlight => {
	    cx.render(rsx!(
		span {
		    background_color: "cyan",

		    token.get_str(),
		}
	    ))
	}
    }
}

#[inline_props]
fn Title(cx: Scope, tokens: Vec<Token>) -> Element {
    cx.render(rsx! {
	h1 {
	    for token in tokens {
		cx.render(rsx!(Token { token: token.clone() }))
	    }
	}
    })
}

#[inline_props]
fn ArticleSection(cx: Scope, section: Section) -> Element {
    match section {
        Section::Heading(level, tokens) => {
	    cx.render(rsx!(
		HeadingN {
		    level: *level,
		    for token in tokens {
			cx.render(rsx!(Token { token: token.clone() }))
		    }
		}
	    ))
	}
	Section::Paragraph(tokens) => {
	    cx.render(rsx!(
		p {
		    for token in tokens {
			cx.render(rsx!(Token { token: token.clone() }))
		    }
		}
	    ))
	}
	Section::UnorderedList(list_items) => {
	    cx.render(rsx!(
		ul {
		    for item in list_items {
			cx.render(rsx!(
			    li {
				ArticleSections { sections: item.clone() }
			    }))
		    }
		}
	    ))
	}
	Section::OrderedList(list_items) => {
	    cx.render(rsx!(
		ol {
		    for item in list_items {
			cx.render(rsx!(
			    li {
				ArticleSections { sections: item.clone() }
			    }))
		    }
		}
	    ))
	}
    }
}

#[inline_props]
fn ArticleSections(cx: Scope, sections: Vec<Section>) -> Element {
    cx.render(rsx! {
	for section in sections {
	    rsx!(
		ArticleSection { section: section.clone() }
	    )
	}
    })
}

fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, || Game::new());
    let game = use_shared_state::<Game>(cx).unwrap();

    cx.render(rsx! (
	button {
	    onclick: move |_| {
		game.write().load_random_article().unwrap();
		game.write().guess("language");
		game.write().guess("the");
		game.write().guess("rust");
		game.write().selected_guess = String::from("rust");
            },

	    "Random article"
	},

	if let Some(wiki_article) = &game.read().wiki_article {
	    rsx!(
		div {
		    font_family: "monospace",
		    letter_spacing: "0.1em",

		    Title { tokens: wiki_article.title.clone() },

		    ArticleSections { sections: wiki_article.content.clone() }
		}
	    )
	} else {
	    rsx!(
		div { },
	    )
	}
    ))
}
