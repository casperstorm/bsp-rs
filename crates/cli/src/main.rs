use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();

    match opts.subcommand {
        Subcommand::Decode { path } => {
            let reader = BufReader::new(File::open(path).unwrap());
            let bsp = decoder::Bsp::from_reader(reader);

            dbg!(bsp);
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    /// Decode the supplied .bsp file
    Decode {
        /// Path of the .bsp file
        path: PathBuf,
    },
}
