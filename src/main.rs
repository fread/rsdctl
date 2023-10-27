#![feature(iter_intersperse)]

use std::io::stdin;
use std::io::stdout;
use std::io::Write;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod article_parser;
mod dioxus;
mod egui;
mod game;
mod wikipedia_api;

use crate::article_parser::{Section, Token};

#[allow(dead_code)]
fn print_tokens(tokens: &Vec<Token>) {
    for token in tokens {
        match token {
            Token::Word(w) => {
                let blanked: String = std::iter::repeat('_').take(w.len()).collect();
                print!("{}", blanked);
                // print!("{}", w);
            }
            Token::NonWord(w) => {
                print!("{}", w);
            }
        }
    }
}

#[allow(dead_code)]
fn print_sections(sections: &Vec<Section>) {
    for section in sections {
        match section {
            Section::Paragraph(tokens) => {
                print_tokens(tokens);
                print!("\n\n");
            }

            Section::Heading(level, tokens) => {
                let mut heading_marker = String::from(" ");
                for _ in 0..*level { heading_marker += "="; }
                heading_marker += " ";

                print!("{}", heading_marker);
                print_tokens(tokens);
                print!("{}\n\n", heading_marker);
            }

            Section::OrderedList(items) => {
                for (i, item) in items.iter().enumerate() {
                    print!("{}. ", i);
                    print_sections(item);
                }
            }

            Section::UnorderedList(items) => {
                for item in items {
                    print!(" * ");
                    print_sections(item);
                }
            }
        }
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();

    println!("Welcome to RsDactl\r");
    println!("==================\r");
    println!("");
    println!("Select graphics adapter\r");
    println!("");
    println!("    1  dioxus web rendering\r");
    println!("    2  egui immediate mode\r");
    println!("");
    print!("Your choice: ");
    stdout.flush().unwrap();

    let stdin = stdin();

    let k = stdin.keys().next();

    // Put terminal back into cooked mode
    std::mem::drop(stdout);

    if let Some(r) = k {
	if let Ok(key) = r {
	    match key {
		Key::Char('1') => {
		    dioxus::launch();
		}
		Key::Char ('2') => {
		    let _ = egui::launch();
		}
		_ => {
		    println!("Please choose 1 or 2");
		}
	    }
	}
    }
}
