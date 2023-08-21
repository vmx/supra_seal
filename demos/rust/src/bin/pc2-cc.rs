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
struct Pc2CcParameters {
    /// The number of sectors to process in parallel.
    num_sectors: usize,
    /// Directory where the resulting TreeC is stored.
    output_dir: String,
    /// Path to a Supraseal configutation file.
    supraseal_config_path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Pc2CcOutput {
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

    let params: Pc2CcParameters = cli::parse_stdin()?;
    info!("{:?}", params);

    supra_seal_demo::init(params.supraseal_config_path);
    supra_seal_demo::pc2(BLOCK_OFFSET, params.num_sectors, params.output_dir);

    let output = Pc2CcOutput::default();
    cli::print_stdout(output)?;

    Ok(())
}
