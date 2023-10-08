#![allow(non_snake_case)]

use anyhow::Result;

use dioxus::prelude::*;

use keyboard_types::Key;

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

#[inline_props]
fn GuessesTable(cx: Scope) -> Element {
    let game = use_shared_state::<Game>(cx).unwrap();

    let guesses_list: Vec<(usize, String)> = game.read().guesses.iter().map(|guess| {
	let guess = guess.clone();
	(game.read().count_word_in_article(&guess).unwrap(), guess)
    }).collect();

    cx.render(rsx!( table {
	id: "guesses-table",

	for (i, (count, guess)) in guesses_list.iter().enumerate() {
	    tr {
		onclick: move |_| {
		    // TODO This is a rather roundabout way, but we
		    // cannot copy guess itself into the closure
		    let guess = game.read().guesses.iter().nth(i).unwrap().to_string();

		    if game.read().selected_guess == guess {
			game.write().selected_guess = "".to_string();
		    } else {
			game.write().selected_guess = guess;
		    }
		},

		td {
		    class: "guesses-count",
		    count.to_string()
		},

		td {
		    class: "guesses-word",
		    guess.to_string()
		}
	    }
	}
    }))
}

fn load_article(game: &UseSharedState<Game>,
		language_tag: &UseState<String>,
		article_title: &UseState<String>,
		load_result: &UseState<Result<()>>) {
    let res = game.write().load_article(
	language_tag.get(),
	article_title.get());

    if let Ok(_) = res {
	article_title.set(String::from(""));
    }

    load_result.set(res);
}

fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, || Game::new());
    let game = use_shared_state::<Game>(cx).unwrap();

    let language_tag = use_state(cx, || "en".to_string());
    let article_title = use_state(cx, || "".to_string());

    let load_result = use_state::<Result<()>>(cx, || Ok(()));

    let next_guess = use_state(cx, || "".to_string());

    cx.render(rsx! (
        style { include_str!("assets/dioxus.css") },

        div {
            id: "grid-container",

            div {
                id: "top-bar",

                span { class: "toolbar-item", "Language tag:" }

                input {
                    class: "toolbar-item",
                    style: "width: 5em",

                    value: "{language_tag}",
                    oninput: move |evt| language_tag.set(evt.value.clone()),
                }

                span { class: "toolbar-item", "Title:" }

                input {
                    class: "toolbar-item",
                    value: "{article_title}",

                    oninput: move |evt| article_title.set(evt.value.clone()),

		    onkeypress: move |evt| {
			if evt.key() == Key::Enter {
			    load_article(game, language_tag, article_title, load_result);
			}
		    },
                }

                button {
                    class: "toolbar-item",
                    onclick: move |_| {
			load_article(game, language_tag, article_title, load_result);
                    },

                    "load article"
                }

                span { class: "toolbar-spacer" }

                button {
                    class: "toolbar-item",
                    onclick: move |_| {
                        let res = game.write().load_random_article();
                        load_result.set(res);
                    },

                    "Random article"
                }
            }

            div {
                id: "article-area",

                if let Some(wiki_article) = &game.read().wiki_article {
                    rsx!( div {
                        id: "article-body",

                        Title { tokens: wiki_article.title.clone() },

                        ArticleSections { sections: wiki_article.content.clone() }
                    })
                } else {
                    if let Err(e) = load_result.get() {
                        rsx!( div {
                            "Error: {e}"
                        })
                    } else {
                        rsx!( div { } )
                    }
                }
            }

            div {
                id: "next-guess",

                span { class: "toolbar-item", "Guess:" }

                input {
                    class: "toolbar-item flex-fill",
                    value: "{next_guess}",

                    oninput: move |evt| {
			next_guess.set(evt.value.clone());
		    },

		    onkeypress: move |evt| {
			if evt.key() == Key::Enter {
			    game.write().guess(next_guess);
			    next_guess.set(String::from(""));
			}
		    },
                }
            }

            div {
                id: "guesses-table-area",

		GuessesTable { },
            }
        }
    ))
}
