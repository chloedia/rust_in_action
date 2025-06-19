use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use regex::Regex;
use clap::{Arg, Command};

fn find_line_containing(corpus: &str, search_term: &str)->Vec<(usize,String)>{
    let mut lines : Vec<(usize, String)> = vec![];
    for (i,line) in corpus.lines().enumerate(){
        if line.contains(search_term){
            lines.push((i,line.to_string()));
        }
   }
    lines
}

fn find_line_containing_with_context(corpus: &str, search_term: &str, context_length: usize)-> Vec<Vec<(usize, String)>>{
 // Now we want to have the n lines before and the n lines after the found line
    let matching_lines : Vec<(usize, String)> = find_line_containing(corpus, search_term);
    let lines: Vec<&str> = corpus.lines().collect();

    let mut all_matches : Vec<Vec<(usize,String)>> = vec![];
    for matching_line in matching_lines{
        let mut context = Vec::with_capacity(2*context_length + 1);
        
        let start = matching_line.0.saturating_sub(context_length);
        let end = usize::min(matching_line.0 + context_length + 1, lines.len());
        let slice = &lines[start..end];

        for (i,ref line) in slice.into_iter().enumerate(){
        context.push((i+start, line.to_string()));
        }
        all_matches.push(context)
    }
    all_matches
}


fn find_line_containing_with_regex(corpus: &str, search_term: &str)->Vec<(usize,String)>{
    let re = Regex::new(search_term).unwrap();
    let mut lines : Vec<(usize, String)> = vec![];
    for (i,line) in corpus.lines().enumerate(){
        let contains_substring = re.find(line);
        match contains_substring{
        Some(_) => lines.push((i,line.to_string())),
        None => (),
        }
   }
    lines
}

fn read_file<R: BufRead>(reader: R ) -> String {
    let mut text : String = String::new();
    for line_result in reader.lines(){
        if let Ok(line) = line_result {
            text.push_str(&line);
            text.push('\n');
        }
    }
    text
}

fn main() {
    let args = Command::new("grep-lite")
        .version("0.1")
        .about("search_for_patterns")
        .arg(Arg::new("pattern")
            .help("The pattern to search for")
            .required(true)
            .num_args(1))
        .arg(Arg::new("input")
            .help("The file path to read")
            .required(false)
            .default_value("-")
            .num_args(1))
        .get_matches();

    let pattern = args.get_one::<String>("pattern").expect("Pattern argument is required");

    let input_path = args.get_one::<String>("input").unwrap();
    let mut text = String::new();
    if input_path == "-"{
        let stdin = io::stdin();
        let reader = stdin.lock();
        text = read_file(reader);
    }else {
        let f = File::open(input_path).unwrap();
        let reader = BufReader::new(f); 
        text = read_file(reader);
    }

    
    let matches = find_line_containing_with_context(&text, pattern, 1);
    
    for local_match in matches {
        println!("New match");
        for line in local_match{
            println!("{:?}", line)
        }
    }


    println!("\nWith Regex : ");

     let matches = find_line_containing_with_regex(&text, pattern);
    
    for line in matches {
            println!("{:?}", line)
        }
}
