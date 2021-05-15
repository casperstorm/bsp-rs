use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use decoder::BspFormat;
use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();

    match opts.subcommand {
        Subcommand::Decode { path } => {
            let reader = BufReader::new(File::open(path).unwrap());

            let decoder = decoder::BspDecoder::from_reader(reader).unwrap();

            match decoder.decode_any() {
                Ok(BspFormat::GoldSrc30(bsp)) => {
                    dbg!(bsp);
                }
                Err(e) => {
                    dbg!(&e);
                }
            }
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
