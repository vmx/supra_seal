use std::fmt;

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, StrictPfx};

use supra_seal_demo::cli;

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
struct Bytes(#[serde(with = "SerHex::<StrictPfx>")] [u8; 32]);

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct C1Parameters {
    /// Directory where the PC2 output is resides and where the C1 output is stored, the
    /// `commit-phase1-output` file.
    cache_dir: String,
    /// The number of sectors that were processed in parallel.
    num_sectors: usize,
    ///// Directory where the output of the C1 is stored `commit-phase1-output` is stored.
    //output_dir: String,
    /// Path to the parents cache file matching the sector size it was compiled for.
    parents_cache_path: String,
    #[serde(with = "SerHex::<StrictPfx>")]
    replica_id: [u8; 32],
    /// Directory to the replica file (the sealed sector), it must contain a file called
    /// `sealed-file`.
    replica_dir: String,
    /// The sector to process
    sector_slot: usize,
    #[serde(with = "SerHex::<StrictPfx>")]
    seed: [u8; 32],
    /// Path to a Supraseal configutation file.
    supraseal_config_path: String,
    #[serde(with = "SerHex::<StrictPfx>")]
    ticket: [u8; 32],
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct C1Output {
    // This is a hack to serialize a struct into an empty Object instead of null
    #[serde(skip_serializing)]
    _placeholder: (),
}

// NOTE vmx 2023-08-17: For now we always start at the beginning of the NVMes, I currently don't
// see a reason not to.
const BLOCK_OFFSET: usize = 0;

//fn main() -> Result<()> {
fn main() -> Result<()> {
    fil_logger::maybe_init();

    let params: C1Parameters = cli::parse_stdin()?;
    info!("{:?}", params);

    supra_seal_demo::init(params.supraseal_config_path);
    supra_seal_demo::c1(
        BLOCK_OFFSET,
        params.num_sectors,
        params.sector_slot,
        &params.replica_id,
        &params.seed,
        &params.ticket,
        &params.cache_dir,
        &params.parents_cache_path,
        &params.replica_dir,
    );

    let output = C1Output::default();
    cli::print_stdout(output)?;

    Ok(())
}
