/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::collections::HashMap;

use quote::{Tokens, ToTokens};
use syn::Delimited;
use syn::DelimToken::{Brace, Paren};
use syn::TokenTree::{self, Token};
use syn::Token::{Colon, Comma, FatArrow, Ident, ModSep};

#[derive(Debug)]
pub struct Event {
    pub name: String,
    pub params: Vec<String>,
}

impl Event {
    fn new() -> Self {
        Event {
            name: String::new(),
            params: vec!["_".to_string()],
        }
    }
}

#[derive(Debug)]
pub struct Widget {
    pub children: Vec<Widget>,
    pub events: HashMap<String, Event>,
    pub init_parameters: Vec<String>,
    pub name: String,
    pub properties: HashMap<String, String>,
}

impl Widget {
    fn new(name: String) -> Self {
        Widget {
            children: vec![],
            events: HashMap::new(),
            init_parameters: vec![],
            name: name,
            properties: HashMap::new(),
        }
    }
}

pub fn parse(tokens: &[TokenTree]) -> Widget {
    let (widget, _) = parse_widget(tokens);
    widget
}

fn parse_widget(tokens: &[TokenTree]) -> (Widget, &[TokenTree]) {
    let (name, mut tokens) = parse_qualified_name(tokens);
    let mut widget = Widget::new(name);
    if let TokenTree::Delimited(Delimited { delim: Paren, ref tts }) = tokens[0] {
        let parameters = parse_comma_list(tts);
        widget.init_parameters = parameters;
        tokens = &tokens[1..];
    }
    if let TokenTree::Delimited(Delimited { delim: Brace, ref tts }) = tokens[0] {
        let mut tts = &tts[..];
        while !tts.is_empty() {
            if try_parse_name(tts).is_some() {
                // Widget.
                match tts[1] {
                    Token(ModSep) | TokenTree::Delimited(Delimited { delim: Brace, .. }) => {
                        let (child, new_tts) = parse_widget(tts);
                        tts = new_tts;
                        widget.children.push(child);
                    },
                    _ => panic!("Expected property, event or child but found `{:?}{:?}` in view! macro", tts[0], tts[1]),
                }
            }
            else {
                // Property or event.
                let (ident, _) = parse_ident(tts);
                tts = &tts[1..];
                match tts[0] {
                    Token(Colon) => {
                        tts = &tts[1..];
                        let (value, new_tts) = parse_value(tts);
                        tts = new_tts;
                        widget.properties.insert(ident, value);
                    },
                    TokenTree::Delimited(Delimited { delim: Paren, .. }) | Token(FatArrow) => {
                        let (event, new_tts) = parse_event(tts);
                        widget.events.insert(ident, event);
                        tts = new_tts;
                    },
                    _ => panic!("Expected `:`, `(` or `[` but found `{:?}` in view! macro", tts[0]),
                }
            }

            if tts.first() == Some(&Token(Comma)) {
                tts = &tts[1..];
            }
        }
    }
    else {
        panic!("Expected {{ but found `{:?}` in view! macro", tokens[0]);
    }
    (widget, &tokens[1..])
}

fn parse_ident(tokens: &[TokenTree]) -> (String, &[TokenTree]) {
    match tokens[0] {
        Token(Ident(ref ident)) => {
            (ident.to_string(), &tokens[1..])
        },
        _ => panic!("Expected ident but found `{:?}` in view! macro", tokens[0]),
    }
}

fn parse_qualified_name(tokens: &[TokenTree]) -> (String, &[TokenTree]) {
    try_parse_name(tokens)
        .unwrap_or_else(|| panic!("Expected qualified name but found `{:?}` in view! macro", tokens[0]))
}

fn try_parse_name(mut tokens: &[TokenTree]) -> Option<(String, &[TokenTree])> {
    let mut segments = vec![];
    while !tokens.is_empty() {
        match tokens[0] {
            Token(Ident(ref ident)) => {
                segments.push(ident.to_string());
            },
            Token(ModSep) => (), // :: is part of a name.
            _ => break,
        }
        tokens = &tokens[1..];
    }
    if segments.is_empty() || segments.last().unwrap().chars().next().unwrap().is_lowercase() {
        None
    }
    else {
        Some((segments.join("::"), tokens))
    }
}

fn parse_comma_list(tokens: &[TokenTree]) -> Vec<String> {
    let mut params = vec![];
    let mut current_param = Tokens::new();
    for token in tokens {
        if *token == Token(Comma) {
            params.push(current_param.to_string());
            current_param = Tokens::new();
        }
        else {
            token.to_tokens(&mut current_param);
        }
    }
    params.push(current_param.to_string());
    params
}

fn parse_event(mut tokens: &[TokenTree]) -> (Event, &[TokenTree]) {
    let mut event = Event::new();
    if let TokenTree::Delimited(Delimited { delim: Paren, ref tts }) = tokens[0] {
        event.params = parse_comma_list(tts);
        tokens = &tokens[1..];
    }
    if tokens[0] != Token(FatArrow) {
        panic!("Expected `=>` but found `{:?}` in view! macro", tokens[0]);
    }
    tokens = &tokens[1..];
    // TODO: parse event like changed(entry) => TextChange(entry.get_text().unwrap()).
    let (name, new_tokens) = parse_qualified_name(tokens);
    event.name = name;
    (event, new_tokens)
}

fn parse_value(tokens: &[TokenTree]) -> (String, &[TokenTree]) {
    let mut current_param = Tokens::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            Token(Comma) => break,
            Token(ref token) => {
                token.to_tokens(&mut current_param);
            },
            _ => panic!("Expected simple token but found `{:?}` in view! macro", tokens[i]),
        }
        i += 1;
    }
    (current_param.to_string(), &tokens[i..])
}
