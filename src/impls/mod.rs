#[cfg(feature = "memory")]
mod in_memory;
#[cfg(feature = "real")]
mod real;

#[cfg(feature = "memory")]
pub use in_memory::InMemoryFile;
#[cfg(feature = "memory")]
pub use in_memory::InMemorySys;
#[cfg(feature = "real")]
pub use real::wasm_path_to_str;
#[cfg(feature = "real")]
pub use real::wasm_string_to_path;
#[cfg(feature = "real")]
pub use real::RealSys;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use real::WasmFile;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub type RealFsFile = WasmFile;
#[cfg(all(feature = "real", not(target_arch = "wasm32")))]
pub type RealFsFile = real::RealFsFile;
