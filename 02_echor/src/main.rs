use clap::Command;

fn main() {
    let _matches = Command::new("echor")
        .version("0.1.0")
        .author("bluthej <joffrey.bluthe@e.email>")
        .about("Rust echo")
        .get_matches();
}
