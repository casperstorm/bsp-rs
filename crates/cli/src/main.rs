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
                    let level_model = bsp.models[0];
                    let mut level_nodes = vec![bsp.nodes[level_model.idx_head_nodes[0] as usize]];
                    let mut level_leaves = vec![];

                    while !level_nodes.is_empty() {
                        let node = level_nodes.pop().unwrap();
                        let front = node.idx_children[0];
                        let back = node.idx_children[1];

                        let mut parse = |x: i16| {
                            if x < -1 {
                                level_leaves.push(x.abs() - 1);
                            } else if x >= 0 {
                                if let Some(n) = bsp.nodes.get(x as usize).copied() {
                                    level_nodes.push(n);
                                }
                            }
                        };

                        parse(front);
                        parse(back);
                    }

                    for leaf_idx in level_leaves {
                        let leaf = bsp.leaves[leaf_idx as usize];

                        for idx in 0..leaf.num_mark_surfaces as usize {
                            if let Some(face) =
                                bsp.faces.get(leaf.idx_first_mark_surface as usize + idx)
                            {
                                dbg!(face);
                            }
                        }
                    }
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
