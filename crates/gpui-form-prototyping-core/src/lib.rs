pub mod code_gen;
pub mod implementations;

pub fn has_koruma_support() -> bool {
    cfg!(feature = "koruma")
}
