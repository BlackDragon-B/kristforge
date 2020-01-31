pub mod interface;
pub mod ocl;

use crate::krist::address::Address;
use crate::krist::block::ShortHash;
use crate::miner::interface::MinerInterface;
use crate::miner::ocl::OclMiner;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub struct MinerConfig {
    /// Don't use OpenCL for mining.
    #[structopt(long)]
    no_gpu: bool,
    // TODO: allow selecting individual devices
    /// OpenCL miner target kernel execution time, in seconds
    #[structopt(long, default_value = "0.1")]
    gpu_rate: f32,

    /// OpenCL miner max work size (default 2^30)
    #[structopt(long, default_value = "1073741824")]
    gpu_max_worksize: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MinerError {
    #[error("OpenCL error: {0}")]
    OclError(#[from] dynamic_ocl::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub work: u64,
    pub block: ShortHash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Solution {
    pub address: Address,
    pub nonce: [u8; 12],
}

pub trait Miner {
    /// Get a human-readable description of this miner
    fn describe(&self) -> String;

    /// Start a long-lived mining operation, blocking the thread and using the
    /// given interface for state operations.
    fn mine(self: Box<Self>, interface: MinerInterface) -> Result<(), MinerError>;
}

pub fn create_miners(opts: MinerConfig) -> Result<Vec<Box<dyn Miner + Send>>, MinerError> {
    let mut miners = Vec::<Box<dyn Miner + Send>>::new();

    if !opts.no_gpu {
        for device in ocl::get_opencl_devices()? {
            miners.push(Box::new(OclMiner::new(device, &opts)?));
        }
    }

    Ok(miners)
}

pub fn calculate_work(hash: [u8; 6]) -> u64 {
    let mut hash_out = [0u8; 8];
    hash_out[2..].copy_from_slice(&hash);
    u64::from_be_bytes(hash_out)
}
