use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(help = "Input text", required = true)]
    text: Vec<String>,

    #[arg(short)]
    no_newline: bool,
}

fn main() {
    let Cli { text, no_newline } = Cli::parse();

    print!("{}{}", text.join(" "), if no_newline { "" } else { "\n" });
}
