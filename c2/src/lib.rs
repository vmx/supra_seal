// Copyright Supranational LLC

sppark::cuda_error!();

#[repr(C)]
struct points_c {
    points: *mut core::ffi::c_void,
    size: usize,
    skip: usize,
    density_map: *const core::ffi::c_void,
    total_density: usize,
}

#[repr(C)]
struct ntt_msm_h_inputs_c {
    h: *mut core::ffi::c_void,
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
    c: *const core::ffi::c_void,
    lg_domain_size: usize,
    actual_size: usize, // this value is very close to a power of 2
}

#[repr(C)]
struct msm_l_a_b_g1_b_g2_inputs_c {
    points_l: points_c,
    points_a: points_c,
    points_b_g1: points_c,
    points_b_g2: points_c,
    input_assignments: *const core::ffi::c_void,
    aux_assignments: *const core::ffi::c_void,
    input_assignment_size: usize,
    aux_assignment_size: usize,
}

extern "C" {
    fn generate_groth16_proof_c(
        ntt_msm_h_inputs: &mut ntt_msm_h_inputs_c,
        msm_l_a_b_g1_b_g2_inputs: &mut msm_l_a_b_g1_b_g2_inputs_c,
        num_circuits: usize,
        r_s: *const core::ffi::c_void,
        s_s: *const core::ffi::c_void,
        proofs: *mut core::ffi::c_void,
    ) -> cuda::Error;

    fn read_srs_c(srs_path: *const std::os::raw::c_char);

    fn reset_srs_c();
}

pub fn read_srs(srs_path: String) {
    let c_srs_path = std::ffi::CString::new(srs_path).unwrap();

    unsafe { read_srs_c(c_srs_path.as_ptr()) };
}

pub fn reset_srs() {
    unsafe { reset_srs_c() };
}

pub fn generate_groth16_proof<S, D, PR>(
    ntt_a_scalars: &[*const S],
    ntt_b_scalars: &[*const S],
    ntt_c_scalars: &[*const S],
    ntt_scalars_actual_size: usize,
    input_assignments: &mut [*const S],
    aux_assignments: &mut [*const S],
    input_assignments_size: usize,
    aux_assignments_size: usize,
    a_aux_density_bv: &[D],
    b_g1_aux_density_bv: &[D],
    a_aux_total_density: usize,
    b_g1_aux_total_density: usize,
    num_circuits: usize,
    r_s: &[S],
    s_s: &[S],
    proofs: &mut [PR],
) {
    let lg_domain_size = (std::mem::size_of::<usize>() * 8) as u32 - (ntt_scalars_actual_size - 1).leading_zeros();

    let bv_element_size: usize = std::mem::size_of::<D>() * 8; // length of D in bits
    assert!(bv_element_size == 64, "only 64-bit elements in bit vectors are supported");

    let mut ntt_msm_h_inputs = ntt_msm_h_inputs_c {
        h: std::ptr::null::<u8>() as *mut core::ffi::c_void,
        a: ntt_a_scalars.as_ptr() as *const core::ffi::c_void,
        b: ntt_b_scalars.as_ptr() as *const core::ffi::c_void,
        c: ntt_c_scalars.as_ptr() as *const core::ffi::c_void,
        lg_domain_size: lg_domain_size as usize,
        actual_size: ntt_scalars_actual_size as usize,
    };

    let points_l = points_c {
        points: std::ptr::null::<u8>() as *mut core::ffi::c_void,
        size: aux_assignments_size,
        skip: 0usize,
        density_map: std::ptr::null() as *const core::ffi::c_void, // l always has FullDensity
        total_density: aux_assignments_size,
    };

    let points_a = points_c {
         points: std::ptr::null::<u8>() as *mut core::ffi::c_void,
         size: a_aux_total_density + input_assignments_size,
         skip: input_assignments_size,
         density_map: a_aux_density_bv.as_ptr() as *const core::ffi::c_void,
         total_density: a_aux_total_density,
    };

    let points_b_g1 = points_c {
         points: std::ptr::null::<u8>() as *mut core::ffi::c_void,
         size: b_g1_aux_total_density + 1,
         skip: 1,
         density_map: b_g1_aux_density_bv.as_ptr() as *const core::ffi::c_void,
         total_density: b_g1_aux_total_density,
    };

    let points_b_g2 = points_c {
         points: std::ptr::null::<u8>() as *mut core::ffi::c_void,
         size: b_g1_aux_total_density + 1,
         skip: 1,
         density_map: b_g1_aux_density_bv.as_ptr() as *const core::ffi::c_void,
         total_density: b_g1_aux_total_density,
    };

    let mut msm_l_a_b_g1_b_g2_inputs = msm_l_a_b_g1_b_g2_inputs_c {
        points_l: points_l,
        points_a: points_a,
        points_b_g1: points_b_g1,
        points_b_g2: points_b_g2,
        input_assignments: input_assignments.as_ptr() as *const core::ffi::c_void,
        aux_assignments: aux_assignments.as_ptr() as *const core::ffi::c_void,
        input_assignment_size: input_assignments_size,
        aux_assignment_size: aux_assignments_size,
    };

    let err = unsafe {
        generate_groth16_proof_c(
            &mut ntt_msm_h_inputs,
            &mut msm_l_a_b_g1_b_g2_inputs,
            num_circuits,
            r_s.as_ptr() as *const core::ffi::c_void,
            s_s.as_ptr() as *const core::ffi::c_void,
            proofs.as_mut_ptr() as *mut core::ffi::c_void,
        )
    };

    if err.code != 0 {
        panic!("{}", String::from(err));
    }
}
