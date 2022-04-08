extern crate pest;
#[macro_use]
extern crate pest_derive;

mod vm;

use std::collections::HashMap;
use pest::Parser;

#[derive(Debug)]
pub enum Statement {
    Event(Event),
    Listener(Listener),
    Behavior(Behavior),
}

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct Type {
    identifier: String,
    fields: HashMap<String, Type>,
}

#[derive(Debug)]
pub struct Event {
    args: Vec<(String, Type)>,
    executor: Block,
}

#[derive(Debug)]
pub struct Listener {
    event: Event,
    executor: Block,
}

#[derive(Debug)]
pub struct Behavior {
    identifier: String,
    executor: Block,
}

pub type AST = Block;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct StillwindParser;

pub fn parse(input: &str) -> Result<AST, pest::error::Error<&str>> {
    let pairs = StillwindParser::parse(Rule::program, input).unwrap_or_else(|e| panic!("{}", e));
    
    for pair in pairs {
        // A pair is a combination of the rule which matched and a span of input
        println!("Rule:    {:?}", pair.as_rule());
        println!("Span:    {:?}", pair.as_span());
        println!("Text:    {}", pair.as_str());

        // A pair can be converted to an iterator of the tokens which make it up:
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::event_stmt => println!("Event:  {}", inner_pair.as_str()),
                Rule::identifier => println!("Identifier: {}", inner_pair.as_str()),
                Rule::statement => println!("Statement: {}", inner_pair.as_str()),
                Rule::block => println!("Block: {}", inner_pair.as_str()),
                _ => println!("Unknown: {}", inner_pair.as_str()),
            };
        }
    }

    Ok(AST { statements: Vec::new() })
}