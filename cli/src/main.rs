use anyhow::Result;
use frontend::Lexer;

fn main() -> Result<()> {
    let mut lex = Lexer::new("def aa() -> = 1 + s_s");
    let tokens = lex.scan_all();

    println!("{:#?}", tokens);
    Ok(())
}
