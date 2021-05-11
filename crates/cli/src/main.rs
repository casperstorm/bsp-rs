use structopt::StructOpt;

fn main() {
    let opts = Opts::from_args();

    dbg!(opts);
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
    Decode,
}
