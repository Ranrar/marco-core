use std::io::Read;

use crate::cli::Args;

pub fn resolve_input(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(text) = &args.text {
        return Ok(text.clone());
    }
    if let Some(path) = &args.file {
        return Ok(std::fs::read_to_string(path)?);
    }
    if args.stdin || !atty::is(atty::Stream::Stdin) {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    Err("No input provided. Use a file path, --text, --stdin, or --interactive.".into())
}
