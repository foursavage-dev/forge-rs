use rust_tokenizer::tokenize;

fn main() {
    let input = "Hello Fish AI Microservices";
    let tokens = tokenize(input);
    println!("Tokens: {:?}", tokens);
}
