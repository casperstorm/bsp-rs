use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();

    match opts.subcommand {
        Subcommand::Decode { path } => {
            let reader = BufReader::new(File::open(path).unwrap());

            let bsp = {
                let mut decoder = decoder::BspDecoder::from_reader(reader).unwrap();

                match decoder.version() {
                    decoder::BspVersion::GoldSrc30 => decoder.decode_gold_src_30(),
                }
            }
            .unwrap();

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
