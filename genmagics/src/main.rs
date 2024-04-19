// most of this comes from https://analog-hors.github.io/site/magic-bitboards/

use std::env;

use genmagics::write_magics_to_file;

fn main() {
    let args: Vec<String> = env::args().collect();
    write_magics_to_file(args[1].clone())
}
