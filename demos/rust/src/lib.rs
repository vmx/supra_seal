// Copyright Supranational LLC

// This is a basic demonstration of the sealing pipeline, the bindings
// interface and order of operations from a rust perspective

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;

pub mod cli;

// TODO vmx 2023-08-16: use proper Replica IDs
pub type ReplicaId = [u8; 32];
//pub type ReplicaId = Vec<u8>;

// C Bindings
mod extern_c {
    use std::ffi::c_char;

    extern "C" {
        // Optional init function. Default config file is supra_config.cfg
        pub fn supra_seal_init(config_filename: *const c_char);

        pub fn get_max_block_offset() -> usize;

        pub fn get_slot_size(num_sectors: usize) -> usize;

        pub fn pc1(
            block_offset: usize,
            num_sectors: usize,
            replica_ids: *const u8,
            parents_filename: *const c_char,
        ) -> u32;

        pub fn pc2(
            block_offset: usize,
            num_sectors: usize,
            output_dir: *const c_char,
            data_filenames: *const *const c_char,
        ) -> u32;

        pub fn pc2_cleanup(num_sectors: usize, output_dir: *const c_char) -> u32;

        pub fn c1(
            block_offset: usize,
            num_sectors: usize,
            sector_slot: usize,
            replica_id: *const u8,
            seed: *const u8,
            ticket: *const u8,
            cache_path: *const c_char,
            parents_filename: *const c_char,
            replica_path: *const c_char,
            //output_dir: *const c_char,
        ) -> u32;
    }
}

pub fn init<T: AsRef<std::path::Path>>(config: T) {
    let config_c = CString::new(config.as_ref().as_os_str().as_bytes()).unwrap();
    unsafe {
        extern_c::supra_seal_init(config_c.as_ptr());
    };
}

pub fn get_max_block_offset() -> usize {
    let max_offset = unsafe { extern_c::get_max_block_offset() };
    println!("Max Offset returned {:x}", max_offset);
    return max_offset;
}

pub fn get_slot_size(num_sectors: usize) -> usize {
    let slot_size = unsafe { extern_c::get_slot_size(num_sectors) };
    println!(
        "Slot size  returned {:x} for {} sectors",
        slot_size, num_sectors
    );
    return slot_size;
}

pub fn pc1<T: AsRef<std::path::Path>>(
    block_offset: usize,
    num_sectors: usize,
    replica_ids: Vec<ReplicaId>,
    path: T,
) -> u32 {
    let replica_ids_bytes = replica_ids
        .iter()
        .map(|replica_id| replica_id.to_vec())
        .flatten()
        .collect::<Vec<u8>>();
    let path_c = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let pc1_status = unsafe {
        extern_c::pc1(
            block_offset,
            num_sectors,
            replica_ids_bytes.as_ptr(),
            path_c.as_ptr(),
        )
    };
    println!("PC1 returned {}", pc1_status);
    return pc1_status;
}

pub fn pc2<T: AsRef<std::path::Path>>(block_offset: usize, num_sectors: usize, path: T) -> u32 {
    let path_c = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let pc2_status =
        unsafe { extern_c::pc2(block_offset, num_sectors, path_c.as_ptr(), std::ptr::null()) };
    println!("PC2 returned {}", pc2_status);
    return pc2_status;
}

pub fn pc2_cleanup<T: AsRef<std::path::Path>>(num_sectors: usize, path: T) -> u32 {
    let path_c = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let pc2_status = unsafe { extern_c::pc2_cleanup(num_sectors, path_c.as_ptr()) };
    println!("PC2 cleanup returned {}", pc2_status);
    return pc2_status;
}

pub fn c1<T: AsRef<std::path::Path>>(
    block_offset: usize,
    num_sectors: usize,
    sector_id: usize,
    replica_id: &[u8; 32],
    seed: &[u8; 32],
    ticket: &[u8; 32],
    cache_path: T,
    parents_filename: T,
    replica_path: T,
    //output_dir: T,
) -> u32 {
    let cache_path_c = CString::new(cache_path.as_ref().as_os_str().as_bytes()).unwrap();
    let parents_c = CString::new(parents_filename.as_ref().as_os_str().as_bytes()).unwrap();
    let replica_path_c = CString::new(replica_path.as_ref().as_os_str().as_bytes()).unwrap();
    //let output_dir_c = CString::new(output_dir.as_ref().as_os_str().as_bytes()).unwrap();

    let c1_status = unsafe {
        extern_c::c1(
            block_offset,
            num_sectors,
            sector_id,
            replica_id.as_ptr(),
            seed.as_ptr(),
            ticket.as_ptr(),
            cache_path_c.as_ptr(),
            parents_c.as_ptr(),
            replica_path_c.as_ptr(),
            //output_dir_c.as_ptr(),
        )
    };
    println!("C1 returned {}", c1_status);
    return c1_status;
}
