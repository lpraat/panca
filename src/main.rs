use std::process::exit;

use panca::cli::Panca;

fn main() -> anyhow::Result<()> {
    let panca = Panca::new();
    if let Err(err) = panca.run() {
        println!("{}", err);
        exit(1);
    };
    Ok(())
}
