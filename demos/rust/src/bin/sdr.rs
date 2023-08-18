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
struct SdrParameters {
    /// Path to the parents cache file matching the sector size it was compiled for.
    parents_cache_path: String,
    /// A list of Replica IDs. The amount of Replica IDs determine how many sectors are sealed in
    /// parallel.
    replica_ids: Vec<Bytes>,
    /// Path to a Supraseal configutation file.
    supraseal_config_path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct SdrOutput {
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

    let params: SdrParameters = cli::parse_stdin()?;
    info!("{:?}", params);

    let replica_ids = params
        .replica_ids
        .iter()
        .map(|replica_id| replica_id.0)
        .collect::<Vec<_>>();
    let num_sectors = replica_ids.len();

    supra_seal_demo::init(params.supraseal_config_path);
    supra_seal_demo::pc1(
        BLOCK_OFFSET,
        num_sectors,
        replica_ids,
        params.parents_cache_path,
    );

    let output = SdrOutput::default();
    cli::print_stdout(output)?;

    Ok(())
}
