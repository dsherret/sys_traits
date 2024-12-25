#[cfg(feature = "memory")]
mod in_memory;
mod real;

#[cfg(feature = "memory")]
pub use in_memory::InMemoryFile;
#[cfg(feature = "memory")]
pub use in_memory::InMemorySys;
pub use real::wasm_path_to_str;
pub use real::wasm_string_to_path;
pub use real::RealSys;

#[cfg(target_arch = "wasm32")]
pub use real::WasmFile;

#[cfg(target_arch = "wasm32")]
pub type RealFsFile = WasmFile;
#[cfg(not(target_arch = "wasm32"))]
pub type RealFsFile = real::RealFsFile;
