
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
fn main() {
    let matches = find_line_containing_with_context("Every face, every shop, bedroom window, public-house, and
    dark square is a picture feverishly turned--in search of what?
    It is the same with books.
    What do we seek through millions of pages?", "oo", 1);
    
    for local_match in matches {
        println!("New match");
        for line in local_match{
            println!("{:?}", line)
        }
    }
}
