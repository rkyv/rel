pub mod from_data;
pub mod gen;
mod log;
mod mc_savedata;
mod mesh;

#[test]
fn test_log_bench() {
    log::make_bench(&mut gen::default_rng(), 10)();
}

#[test]
fn test_mesh_bench() {
    mesh::make_bench(&mut gen::default_rng(), 10)();
}

#[test]
fn test_mc_savedata_bench() {
    mc_savedata::make_bench(&mut gen::default_rng(), 10)();
}
