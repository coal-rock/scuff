use crate::token::Token;

pub fn error(line: usize, string: String) {
    println!("[{:?}] {:?}", line, string);
}

pub fn parse_error(tok: Token, string: String) {
    println!("PARSE ERROR: [{:?}] {:?}", tok, string);
    todo!();
}
