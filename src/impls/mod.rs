use std::path::PathBuf;

mod cwd;

pub use cwd::CwdSys;

#[cfg(feature = "real")]
// do not implement Copy so that swapping out the RealSys
// with another implementation that requires Clone based
// on compliation settings will not give a clippy error
#[derive(Debug, Default, Clone)]
pub struct RealSys;

#[cfg(any(
  all(feature = "real", target_os = "windows", feature = "winapi"),
  all(feature = "real", unix, feature = "libc")
))]
pub use real::real_cache_dir_with_env;
#[cfg(any(
  all(feature = "real", target_os = "windows", feature = "winapi"),
  all(feature = "real", unix, feature = "libc")
))]
pub use real::real_home_dir_with_env;

#[cfg(feature = "memory")]
mod in_memory;
#[cfg(all(feature = "real", not(target_arch = "wasm32"),))]
mod real;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm;

#[cfg(feature = "memory")]
pub use in_memory::InMemoryDirEntry;
#[cfg(feature = "memory")]
pub use in_memory::InMemoryFile;
#[cfg(feature = "memory")]
pub use in_memory::InMemoryMetadata;
#[cfg(feature = "memory")]
pub use in_memory::InMemorySys;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use wasm::is_windows;

/// Checks if the current executing environment is Windows.
///
/// This may be useful to check when executing in Wasm.
#[cfg(not(all(feature = "wasm", target_arch = "wasm32")))]
pub fn is_windows() -> bool {
  cfg!(windows)
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub type RealFsFile = wasm::WasmFile;
#[cfg(all(
  feature = "real",
  not(target_arch = "wasm32"),
  not(feature = "wasm")
))]
pub type RealFsFile = real::RealFsFile;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub type RealFsMetadata = wasm::WasmMetadata;
#[cfg(all(
  feature = "real",
  not(target_arch = "wasm32"),
  not(feature = "wasm")
))]
pub type RealFsMetadata = real::RealFsMetadata;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub type RealFsDirEntry = wasm::WasmFsDirEntry;
#[cfg(all(
  feature = "real",
  not(target_arch = "wasm32"),
  not(feature = "wasm")
))]
pub type RealFsDirEntry = real::RealFsDirEntry;

/// Helper that converts a string to a path for Wasm.
///
/// This will handle converting Windows-style paths received from JS
/// to Unix-style paths that work in Wasm in Rust. This is unfortunately
/// necessary because Wasm code in Rust uses Unix-style paths and there's
/// no way to configure it to use Windows style paths when we know we're
/// running on Windows. This is not perfect, but will make things work in
/// 99% of scenarios, which is better than not working at all.
///
/// See and upvote: https://github.com/rust-lang/rust/issues/66621#issuecomment-2561279536
#[cfg(feature = "real")]
pub fn wasm_string_to_path(path: String) -> PathBuf {
  #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
  {
    // one day we might have:
    // but for now, do this hack for windows users
    if wasm::is_windows() && is_windows_absolute_path(&path) {
      PathBuf::from("/").join(path.replace("\\", "/"))
    } else {
      PathBuf::from(path)
    }
  }
  #[cfg(not(target_arch = "wasm32"))]
  {
    PathBuf::from(path)
  }
}

/// Helper that converts a path to a string for Wasm. The `wasm` feature
/// must be enabled for this to work.
///
/// This will convert a path to have backslashes for JS on Windows.
///
/// See notes on `wasm_string_to_path` for more information.
#[cfg(feature = "real")]
pub fn wasm_path_to_str(path: &std::path::Path) -> std::borrow::Cow<'_, str> {
  #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
  {
    if wasm::is_windows() {
      let path = path.to_string_lossy();
      let path = path.strip_prefix('/').unwrap_or(&path);
      std::borrow::Cow::Owned(path.replace("/", "\\"))
    } else {
      path.to_string_lossy()
    }
  }
  #[cfg(any(not(target_arch = "wasm32"), not(feature = "wasm")))]
  {
    path.to_string_lossy()
  }
}

/// Gets if the path is a Windows absolute path (ex. `C:\dir`), which
/// Rust doesn't consider absolute in Wasm because it uses Unix-style paths.
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub(super) fn is_windows_absolute_path(path: &str) -> bool {
  let mut chars = path.chars();
  let Some(first_char) = chars.next() else {
    return false;
  };
  if !first_char.is_alphabetic() {
    return false;
  }
  let Some(second_char) = chars.next() else {
    return false;
  };
  if second_char != ':' {
    return false;
  }
  let third_char = chars.next();
  third_char == Some('\\') || third_char == Some('/')
}

#[cfg(any(not(windows), not(feature = "strip_unc")))]
#[inline]
#[allow(dead_code)]
pub(super) fn strip_unc_prefix(path: PathBuf) -> PathBuf {
  path
}

/// Strips the unc prefix (ex. \\?\) from Windows paths.
#[cfg(all(windows, feature = "strip_unc"))]
#[allow(dead_code)] // only used with the "real" feature
pub(super) fn strip_unc_prefix(path: PathBuf) -> PathBuf {
  use std::path::Component;
  use std::path::Prefix;

  let mut components = path.components();
  match components.next() {
    Some(Component::Prefix(prefix)) => {
      match prefix.kind() {
        // \\?\c:\path
        Prefix::VerbatimDisk(_) => {
          let mut path = PathBuf::new();
          path.push(prefix.as_os_str().to_string_lossy().replace(r"\\?\", ""));
          path.extend(components);
          path
        }
        // \\?\UNC\hostname\share_name\path
        Prefix::VerbatimUNC(hostname, share_name) => {
          let mut path = PathBuf::new();
          path.push(format!(
            r"\\{}\{}\",
            hostname.to_string_lossy(),
            share_name.to_string_lossy()
          ));
          path.extend(components.filter(|c| !matches!(c, Component::RootDir)));
          path
        }
        // Leave other verbatim paths (ex. \\?\Volume{guid}\path) as-is because
        // they have no equivalent non-verbatim form. For example, stripping
        // the prefix to \\Volume{guid}\path would make it a path to a server.
        _ => path,
      }
    }
    _ => path,
  }
}

#[cfg(all(test, windows, feature = "strip_unc"))]
mod tests {
  use super::*;

  #[test]
  fn test_strip_unc_prefix() {
    let run_test = |input: &str, expected: &str| {
      assert_eq!(
        strip_unc_prefix(PathBuf::from(input)),
        PathBuf::from(expected),
        "input: {input}"
      );
    };

    run_test(r"C:\dir\file.txt", r"C:\dir\file.txt");
    run_test(r"\\?\C:\dir\file.txt", r"C:\dir\file.txt");
    run_test(r"\\?\C:\", r"C:\");
    run_test(r"\\server\share\file.txt", r"\\server\share\file.txt");
    run_test(
      r"\\?\UNC\server\share\dir\file.txt",
      r"\\server\share\dir\file.txt",
    );
    run_test(
      r"\\?\Volume{b75e2c83-0000-0000-0000-602f00000000}\dir\file.txt",
      r"\\?\Volume{b75e2c83-0000-0000-0000-602f00000000}\dir\file.txt",
    );
  }
}
