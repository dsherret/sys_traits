use std::borrow::Cow;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use std::io::Error;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use std::io::ErrorKind;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::prelude::*;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::JsValue;

use crate::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct RealSys;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = chmodSync, catch)]
  fn deno_chmod_sync(path: &str, mode: u32)
    -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = chdir, catch)]
  fn deno_chdir(path: &str) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = cwd, catch)]
  fn deno_cwd() -> std::result::Result<String, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = linkSync, catch)]
  fn deno_link_sync(src: &str, dst: &str) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = ["Deno"], js_name = lstatSync, catch)]
  fn deno_lstat_sync(
    path: &str,
  ) -> std::result::Result<JsValue, wasm_bindgen::JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = mkdirSync, catch)]
  fn deno_mkdir_sync(
    path: &str,
    options: &JsValue,
  ) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = openSync, catch)]
  fn deno_open_sync(
    path: &str,
    options: &JsValue,
  ) -> std::result::Result<JsValue, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = readFileSync, catch)]
  fn deno_read_file_sync(path: &str) -> std::result::Result<JsValue, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = readTextFileSync, catch)]
  fn deno_read_text_file_sync(
    path: &str,
  ) -> std::result::Result<String, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = readDirSync, catch)]
  fn deno_read_dir_sync(
    path: &str,
  ) -> std::result::Result<js_sys::Iterator, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = realPathSync, catch)]
  fn deno_real_path_sync(path: &str) -> std::result::Result<String, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = removeSync, catch)]
  fn deno_remove_sync(path: &str) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = removeSync, catch)]
  fn deno_remove_sync_options(
    path: &str,
    options: &JsValue,
  ) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = renameSync, catch)]
  fn deno_rename_sync(
    oldpath: &str,
    newpath: &str,
  ) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = ["Deno"], js_name = statSync, catch)]
  fn deno_stat_sync(
    path: &str,
  ) -> std::result::Result<JsValue, wasm_bindgen::JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = symlinkSync, catch)]
  fn deno_symlink_sync(
    old_path: &str,
    new_path: &str,
    options: &JsValue,
  ) -> std::result::Result<(), wasm_bindgen::JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = writeFileSync, catch)]
  fn deno_write_file_sync(
    path: &str,
    data: &[u8],
  ) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["globalThis", "Date"], js_name = now)]
  fn date_now() -> f64;
  #[wasm_bindgen(js_namespace = ["globalThis", "crypto"], js_name = getRandomValues, catch)]
  fn get_random_values(buf: &mut [u8]) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = Atomics, js_name = wait)]
  fn atomics_wait(
    i32array: &js_sys::Int32Array,
    index: u32,
    value: i32,
    timeout: f64,
  ) -> String;

  // Deno.FsFile
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = FsFile)]
  #[derive(Clone, Debug)]
  type DenoFsFile;
  #[wasm_bindgen(method, structural, js_name = close)]
  fn close_internal(this: &DenoFsFile);
  #[wasm_bindgen(method, structural, js_name = writeSync, catch)]
  fn write_sync_internal(
    this: &DenoFsFile,
    data: &[u8],
  ) -> std::result::Result<usize, JsValue>;
  #[wasm_bindgen(method, structural, js_name = syncSync)]
  fn sync_internal(this: &DenoFsFile);
  #[wasm_bindgen(method, structural, js_name = readSync, catch)]
  fn read_sync_internal(
    this: &DenoFsFile,
    buffer: &mut [u8],
  ) -> std::result::Result<usize, JsValue>;
  #[wasm_bindgen(method, structural, js_name = seekSync, catch)]
  fn seek_sync_i64_internal(
    this: &DenoFsFile,
    offset: i64,
    seek_mode: u32,
  ) -> std::result::Result<u32, wasm_bindgen::JsValue>;
  #[wasm_bindgen(method, structural, js_name = seekSync, catch)]
  fn seek_sync_u64_internal(
    this: &DenoFsFile,
    offset: u64,
    seek_mode: u32,
  ) -> std::result::Result<u32, wasm_bindgen::JsValue>;

  // Deno.build
  #[wasm_bindgen(js_namespace = Deno, js_name = build)]
  static BUILD: JsValue;

  // Deno.env
  #[wasm_bindgen(js_namespace = Deno, js_name = env)]
  static ENV: JsValue;
}

// ==== Environment ====

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl EnvCurrentDir for RealSys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    std::env::current_dir()
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl EnvCurrentDir for RealSys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    deno_cwd()
      .map(wasm_string_to_path)
      .map_err(|err| js_value_to_io_error(err))
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseEnvSetCurrentDir for RealSys {
  fn base_env_set_current_dir(&self, path: &Path) -> std::io::Result<()> {
    std::env::set_current_dir(path)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseEnvSetCurrentDir for RealSys {
  fn base_env_set_current_dir(&self, path: &Path) -> std::io::Result<()> {
    deno_chdir(&wasm_path_to_str(path)).map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseEnvVar for RealSys {
  fn base_env_var_os(&self, key: &OsStr) -> Option<OsString> {
    std::env::var_os(key)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseEnvVar for RealSys {
  fn base_env_var_os(&self, key: &OsStr) -> Option<OsString> {
    let key = key.to_str()?;
    let get_fn = js_sys::Reflect::get(&ENV, &JsValue::from_str("get"))
      .ok()
      .and_then(|v| v.dyn_into::<js_sys::Function>().ok())?;
    let key_js = JsValue::from_str(key);
    let value_js = get_fn.call1(&ENV, &key_js).ok()?;
    return value_js.as_string().map(OsString::from);
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseEnvSetVar for RealSys {
  fn base_env_set_var(&self, key: &OsStr, value: &OsStr) {
    std::env::set_var(key, value);
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseEnvSetVar for RealSys {
  fn base_env_set_var(&self, key: &OsStr, value: &OsStr) {
    let key = key.to_str().unwrap();
    let value = value.to_str().unwrap();
    let set_fn = js_sys::Reflect::get(&ENV, &JsValue::from_str("set"))
      .ok()
      .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
      .unwrap();
    let key_js = JsValue::from_str(key);
    let value_js = JsValue::from_str(value);
    set_fn.call2(&ENV, &key_js, &value_js).unwrap();
  }
}

#[cfg(all(unix, feature = "libc"))]
impl EnvCacheDir for RealSys {
  fn env_cache_dir(&self) -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
      self.env_home_dir().map(|h| h.join("Library/Caches"))
    } else {
      self
        .env_var_path("XDG_CACHE_HOME")
        .or_else(|| self.env_home_dir().map(|home| home.join(".cache")))
    }
  }
}

#[cfg(all(target_os = "windows", feature = "winapi"))]
impl EnvCacheDir for RealSys {
  fn env_cache_dir(&self) -> Option<PathBuf> {
    known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl EnvCacheDir for RealSys {
  fn env_cache_dir(&self) -> Option<PathBuf> {
    match build_os() {
      Os::Linux => self
        .env_var_path("XDG_CACHE_HOME")
        .or_else(|| self.env_home_dir().map(|home| home.join(".cache"))),
      Os::Mac => self.env_home_dir().map(|h| h.join("Library/Caches")),
      Os::Windows => self
        .env_var_path("USERPROFILE")
        .map(|dir| dir.join("AppData/Local")),
    }
  }
}

#[cfg(all(unix, feature = "libc"))]
impl EnvHomeDir for RealSys {
  fn env_home_dir(&self) -> Option<PathBuf> {
    // This piece of code was taken from the deprecated home_dir() function in Rust's standard library
    unsafe fn fallback() -> Option<std::ffi::OsString> {
      let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
        n if n < 0 => 512_usize,
        n => n as usize,
      };
      let mut buf = Vec::with_capacity(amt);
      let mut passwd: libc::passwd = std::mem::zeroed();
      let mut result = std::ptr::null_mut();
      match libc::getpwuid_r(
        libc::getuid(),
        &mut passwd,
        buf.as_mut_ptr(),
        buf.capacity(),
        &mut result,
      ) {
        0 if !result.is_null() => {
          let ptr = passwd.pw_dir as *const _;
          let bytes = std::ffi::CStr::from_ptr(ptr).to_bytes().to_vec();
          Some(std::os::unix::ffi::OsStringExt::from_vec(bytes))
        }
        _ => None,
      }
    }

    self.env_var_path("HOME").or_else(|| {
      // SAFETY: libc
      unsafe { fallback().map(PathBuf::from) }
    })
  }
}

#[cfg(all(target_os = "windows", feature = "winapi"))]
impl EnvHomeDir for RealSys {
  fn env_home_dir(&self) -> Option<PathBuf> {
    self.env_var_path("USERPROFILE").or_else(|| {
      known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_Profile)
    })
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl EnvHomeDir for RealSys {
  fn env_home_dir(&self) -> Option<PathBuf> {
    if is_windows() {
      self.env_var_path("USERPROFILE")
    } else {
      self.env_var_path("HOME")
    }
  }
}

// ==== File System ====

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsCanonicalize for RealSys {
  #[inline]
  fn base_fs_canonicalize(&self, path: &Path) -> Result<PathBuf> {
    std::fs::canonicalize(path).map(strip_unc_prefix)
  }
}

#[cfg(any(not(windows), not(feature = "strip_unc")))]
#[inline]
pub fn strip_unc_prefix(path: PathBuf) -> PathBuf {
  path
}

/// Strips the unc prefix (ex. \\?\) from Windows paths.
#[cfg(all(windows, feature = "strip_unc"))]
pub fn strip_unc_prefix(path: PathBuf) -> PathBuf {
  use std::path::Component;
  use std::path::Prefix;

  let mut components = path.components();
  match components.next() {
    Some(Component::Prefix(prefix)) => {
      match prefix.kind() {
        // \\?\device
        Prefix::Verbatim(device) => {
          let mut path = PathBuf::new();
          path.push(format!(r"\\{}\", device.to_string_lossy()));
          path.extend(components.filter(|c| !matches!(c, Component::RootDir)));
          path
        }
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
        _ => path,
      }
    }
    _ => path,
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsCanonicalize for RealSys {
  fn base_fs_canonicalize(&self, path: &Path) -> Result<PathBuf> {
    deno_real_path_sync(&wasm_path_to_str(path))
      .map(wasm_string_to_path)
      .map(strip_unc_prefix)
      .map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsCreateDirAll for RealSys {
  #[inline]
  fn base_fs_create_dir_all(&self, path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsCreateDirAll for RealSys {
  fn base_fs_create_dir_all(&self, path: &Path) -> Result<()> {
    let path_str = wasm_path_to_str(path);

    // Create the options object for mkdirSync
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
      &options,
      &JsValue::from_str("recursive"),
      &JsValue::from_bool(true),
    )
    .map_err(|e| js_value_to_io_error(e))?;

    // Call the Deno.mkdirSync function
    deno_mkdir_sync(&path_str, &JsValue::from(options))
      .map_err(|e| js_value_to_io_error(e))
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsHardLink for RealSys {
  #[inline]
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> Result<()> {
    std::fs::hard_link(src, dst)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsHardLink for RealSys {
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
    let src_str = wasm_path_to_str(src);
    let dst_str = wasm_path_to_str(dst);

    deno_link_sync(&src_str, &dst_str).map_err(js_value_to_io_error)
  }
}

/// A wrapper type is used in order to force usages to
/// `use sys_traits::FsMetadataValue` so that the code
/// compiles under Wasm.
#[derive(Debug, Clone)]
pub struct RealFsMetadata(std::fs::Metadata);

impl FsMetadataValue for RealFsMetadata {
  fn file_type(&self) -> FileType {
    self.0.file_type().into()
  }

  #[inline]
  fn modified(&self) -> Result<SystemTime> {
    self.0.modified()
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl From<&JsValue> for FileType {
  fn from(value: &JsValue) -> Self {
    let is_file = js_sys::Reflect::get(value, &JsValue::from_str("isFile"))
      .ok()
      .and_then(|v| v.as_bool())
      .unwrap_or(false);
    if is_file {
      return FileType::File;
    }

    let is_directory =
      js_sys::Reflect::get(value, &JsValue::from_str("isDirectory"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if is_directory {
      return FileType::Dir;
    }

    let is_symlink =
      js_sys::Reflect::get(value, &JsValue::from_str("isSymlink"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if is_symlink {
      return FileType::Symlink;
    }

    FileType::Unknown
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[derive(Debug, Clone)]
pub struct WasmMetadata(JsValue);

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl FsMetadataValue for WasmMetadata {
  fn file_type(&self) -> FileType {
    (&self.0).into()
  }

  fn modified(&self) -> Result<SystemTime> {
    let m = js_sys::Reflect::get(&self.0, &JsValue::from_str("mtime"))
      .map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to access mtime")
      })?;

    if m.is_undefined() || m.is_null() {
      Err(Error::new(ErrorKind::Other, "mtime not found"))
    } else {
      parse_date(&m)
    }
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsMetadata for RealSys {
  type Metadata = RealFsMetadata;

  #[inline]
  fn base_fs_metadata(&self, path: &Path) -> Result<Self::Metadata> {
    std::fs::metadata(path).map(RealFsMetadata)
  }

  #[inline]
  fn base_fs_symlink_metadata(&self, path: &Path) -> Result<Self::Metadata> {
    std::fs::symlink_metadata(path).map(RealFsMetadata)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsMetadata for RealSys {
  type Metadata = WasmMetadata;

  #[inline]
  fn base_fs_metadata(&self, path: &Path) -> Result<WasmMetadata> {
    let s = wasm_path_to_str(path);
    match deno_stat_sync(&s) {
      Ok(v) => Ok(WasmMetadata(v)),
      Err(e) => Err(js_value_to_io_error(e)),
    }
  }

  #[inline]
  fn base_fs_symlink_metadata(&self, path: &Path) -> Result<WasmMetadata> {
    let s = wasm_path_to_str(path);
    match deno_lstat_sync(&s) {
      Ok(v) => Ok(WasmMetadata(v)),
      Err(e) => Err(js_value_to_io_error(e)),
    }
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
fn parse_date(value: &JsValue) -> Result<SystemTime> {
  let date = value
    .dyn_ref::<js_sys::Date>()
    .ok_or_else(|| Error::new(ErrorKind::Other, "value not a date"))?;
  let ms = date.get_time() as u64;
  Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ms))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsOpen for RealSys {
  type File = RealFsFile;

  fn base_fs_open(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File> {
    let mut builder = std::fs::OpenOptions::new();
    if let Some(mode) = options.mode {
      #[cfg(unix)]
      {
        use std::os::unix::fs::OpenOptionsExt;
        builder.mode(mode);
      }
      #[cfg(not(unix))]
      let _ = mode;
    }
    builder
      .read(options.read)
      .write(options.write)
      .create(options.create)
      .truncate(options.truncate)
      .append(options.append)
      .create_new(options.create_new)
      .open(path)
      .map(RealFsFile)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsOpen for RealSys {
  type File = WasmFile;

  fn base_fs_open(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> std::io::Result<WasmFile> {
    let s = wasm_path_to_str(path).into_owned();
    let js_options = js_sys::Object::new();
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("read"),
      &JsValue::from_bool(options.read),
    )
    .map_err(js_value_to_io_error)?;
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("write"),
      &JsValue::from_bool(options.write),
    )
    .map_err(js_value_to_io_error)?;
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("create"),
      &JsValue::from_bool(options.create),
    )
    .map_err(js_value_to_io_error)?;
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("truncate"),
      &JsValue::from_bool(options.truncate),
    )
    .map_err(js_value_to_io_error)?;
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("append"),
      &JsValue::from_bool(options.append),
    )
    .map_err(js_value_to_io_error)?;
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("createNew"),
      &JsValue::from_bool(options.create_new),
    )
    .map_err(js_value_to_io_error)?;
    let js_file =
      deno_open_sync(&s, &js_options).map_err(js_value_to_io_error)?;
    let file = js_file
      .dyn_into::<DenoFsFile>()
      .map_err(js_value_to_io_error)?;
    Ok(WasmFile { file, path: s })
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsRead for RealSys {
  #[inline]
  fn base_fs_read(&self, path: &Path) -> Result<Cow<'static, [u8]>> {
    std::fs::read(path).map(Cow::Owned)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsRead for RealSys {
  fn base_fs_read(&self, path: &Path) -> Result<Cow<'static, [u8]>> {
    let s = wasm_path_to_str(path);
    let v = deno_read_file_sync(&s).map_err(js_value_to_io_error)?;
    let b = js_sys::Uint8Array::new(&v).to_vec();
    Ok(Cow::Owned(b))
  }
}

#[derive(Debug)]
pub struct RealFsDirEntry(std::fs::DirEntry);

impl FsDirEntry for RealFsDirEntry {
  type Metadata = RealFsMetadata;

  fn file_name(&self) -> Cow<OsStr> {
    Cow::Owned(self.0.file_name())
  }

  fn file_type(&self) -> std::io::Result<FileType> {
    self.0.file_type().map(FileType::from)
  }

  fn metadata(&self) -> std::io::Result<Self::Metadata> {
    self.0.metadata().map(RealFsMetadata)
  }

  fn path(&self) -> Cow<Path> {
    Cow::Owned(self.0.path())
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsReadDir for RealSys {
  type ReadDirEntry = RealFsDirEntry;

  #[inline]
  fn base_fs_read_dir(
    &self,
    path: &Path,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  > {
    let iterator = std::fs::read_dir(path)?;
    Ok(Box::new(iterator.map(|result| result.map(RealFsDirEntry))))
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsReadDir for RealSys {
  type ReadDirEntry = WasmFsDirEntry;

  fn base_fs_read_dir(
    &self,
    path: &Path,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  > {
    let path_str = wasm_path_to_str(path);

    // Use Deno.readDirSync to get directory entries
    let entries =
      deno_read_dir_sync(&path_str).map_err(js_value_to_io_error)?;

    let path = path.to_path_buf();
    Ok(Box::new(entries.into_iter().map(move |entry| {
      entry
        .map_err(|_| {
          Error::new(ErrorKind::Other, "Failed to iterate over entries")
        })
        .and_then(|value| {
          Ok(WasmFsDirEntry {
            value,
            parent_path: path.clone(),
          })
        })
    })))
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[derive(Debug)]
pub struct WasmFsDirEntry {
  parent_path: PathBuf,
  value: JsValue,
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl FsDirEntry for WasmFsDirEntry {
  type Metadata = WasmMetadata;

  fn file_name(&self) -> Cow<OsStr> {
    let name = js_sys::Reflect::get(&self.value, &JsValue::from_str("name"))
      .ok()
      .and_then(|v| v.as_string())
      .unwrap_or_default();
    Cow::Owned(OsString::from(name))
  }

  fn file_type(&self) -> std::io::Result<FileType> {
    Ok((&self.value).into())
  }

  fn metadata(&self) -> std::io::Result<Self::Metadata> {
    // Use the same `self.inner` for metadata as it includes file stats
    Ok(WasmMetadata(self.value.clone().into()))
  }

  fn path(&self) -> Cow<Path> {
    let name = js_sys::Reflect::get(&self.value, &JsValue::from_str("name"))
      .ok()
      .and_then(|v| v.as_string())
      .unwrap_or_default();
    Cow::Owned(self.parent_path.join(name))
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsRemoveDirAll for RealSys {
  fn base_fs_remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
    std::fs::remove_dir_all(path)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsRemoveDirAll for RealSys {
  fn base_fs_remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
    let s = wasm_path_to_str(path);
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
      &options,
      &JsValue::from_str("recursive"),
      &JsValue::from_bool(true),
    )
    .map_err(js_value_to_io_error)?;
    deno_remove_sync_options(&s, &options).map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsRemoveFile for RealSys {
  fn base_fs_remove_file(&self, path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsRemoveFile for RealSys {
  fn base_fs_remove_file(&self, path: &Path) -> std::io::Result<()> {
    let s = wasm_path_to_str(path);
    deno_remove_sync(&s).map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsRename for RealSys {
  fn base_fs_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::rename(from, to)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsRename for RealSys {
  fn base_fs_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
    let f = wasm_path_to_str(from);
    let t = wasm_path_to_str(to);
    deno_rename_sync(&f, &t).map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsSymlinkDir for RealSys {
  fn base_fs_symlink_dir(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()> {
    #[cfg(windows)]
    {
      std::os::windows::fs::symlink_dir(original, link)
    }
    #[cfg(not(windows))]
    {
      std::os::unix::fs::symlink(original, link)
    }
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsSymlinkDir for RealSys {
  fn base_fs_symlink_dir(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()> {
    let old_path = wasm_path_to_str(original);
    let new_path = wasm_path_to_str(link);

    // Create an options object for Deno.symlinkSync specifying a directory symlink
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
      &options,
      &wasm_bindgen::JsValue::from_str("type"),
      &wasm_bindgen::JsValue::from_str("dir"),
    )
    .map_err(js_value_to_io_error)?;

    deno_symlink_sync(
      &old_path,
      &new_path,
      &wasm_bindgen::JsValue::from(options),
    )
    .map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsSymlinkFile for RealSys {
  fn base_fs_symlink_file(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()> {
    #[cfg(windows)]
    {
      std::os::windows::fs::symlink_file(original, link)
    }
    #[cfg(not(windows))]
    {
      std::os::unix::fs::symlink(original, link)
    }
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsSymlinkFile for RealSys {
  fn base_fs_symlink_file(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()> {
    let old_path = wasm_path_to_str(original);
    let new_path = wasm_path_to_str(link);

    // Create an options object for Deno.symlinkSync specifying a file symlink
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
      &options,
      &wasm_bindgen::JsValue::from_str("type"),
      &wasm_bindgen::JsValue::from_str("file"),
    )
    .map_err(js_value_to_io_error)?;

    deno_symlink_sync(
      &old_path,
      &new_path,
      &wasm_bindgen::JsValue::from(options),
    )
    .map_err(js_value_to_io_error)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl BaseFsWrite for RealSys {
  #[inline]
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, data)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl BaseFsWrite for RealSys {
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
    let s = wasm_path_to_str(path);
    deno_write_file_sync(&s, data).map_err(js_value_to_io_error)
  }
}

// ==== File System File ====

/// A wrapper type is used in order to force usages to
/// `use sys_traits::FsFile` so that the code
/// compiles under Wasm.
#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
#[derive(Debug)]
pub struct RealFsFile(std::fs::File);

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl FsFile for RealFsFile {}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl FsFileSetPermissions for RealFsFile {
  #[inline]
  fn fs_file_set_permissions(&mut self, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let permissions = std::fs::Permissions::from_mode(mode);
      self.0.set_permissions(permissions)
    }
    #[cfg(not(unix))]
    {
      let _ = mode;
      Ok(())
    }
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl std::io::Seek for RealFsFile {
  fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64> {
    self.0.seek(pos)
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl std::io::Write for RealFsFile {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self.0.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> Result<()> {
    self.0.flush()
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl std::io::Read for RealFsFile {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    self.0.read(buf)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[derive(Debug)]
pub struct WasmFile {
  file: DenoFsFile,
  path: String,
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl Drop for WasmFile {
  fn drop(&mut self) {
    self.file.close_internal();
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl FsFile for WasmFile {}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl FsFileSetPermissions for WasmFile {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()> {
    if is_windows() {
      return Ok(()); // ignore
    }
    deno_chmod_sync(&self.path, mode).map_err(js_value_to_io_error)
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl std::io::Seek for WasmFile {
  fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64> {
    match pos {
      std::io::SeekFrom::Start(offset) => self
        .file
        .seek_sync_u64_internal(offset, 0)
        .map(|v| v as u64)
        .map_err(js_value_to_io_error),
      std::io::SeekFrom::End(offset) => self
        .file
        .seek_sync_i64_internal(offset, 2)
        .map(|v| v as u64)
        .map_err(js_value_to_io_error),
      std::io::SeekFrom::Current(offset) => self
        .file
        .seek_sync_i64_internal(offset, 1)
        .map(|v| v as u64)
        .map_err(js_value_to_io_error),
    }
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl std::io::Write for WasmFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self
      .file
      .write_sync_internal(buf)
      .map_err(js_value_to_io_error)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.file.sync_internal();
    Ok(())
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl std::io::Read for WasmFile {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self
      .file
      .read_sync_internal(buf)
      .map_err(js_value_to_io_error)
  }
}

// ==== System ====

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl SystemTimeNow for RealSys {
  #[inline]
  fn sys_time_now(&self) -> SystemTime {
    SystemTime::now()
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl SystemTimeNow for RealSys {
  #[inline]
  fn sys_time_now(&self) -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(date_now() as u64)
  }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "getrandom"))]
impl crate::SystemRandom for RealSys {
  #[inline]
  fn sys_random(&self, buf: &mut [u8]) -> Result<()> {
    getrandom::getrandom(buf)
      .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl crate::SystemRandom for RealSys {
  #[inline]
  fn sys_random(&self, buf: &mut [u8]) -> Result<()> {
    const MAX_BUFFER_SIZE: usize = 65536; // max buffer size for WebCrypto

    for chunk in buf.chunks_mut(MAX_BUFFER_SIZE) {
      if let Err(err) = get_random_values(chunk) {
        return Err(js_value_to_io_error(err));
      }
    }
    Ok(())
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
impl crate::ThreadSleep for RealSys {
  fn thread_sleep(&self, duration: std::time::Duration) {
    std::thread::sleep(duration);
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl crate::ThreadSleep for RealSys {
  fn thread_sleep(&self, duration: std::time::Duration) {
    use js_sys::Int32Array;
    use js_sys::SharedArrayBuffer;

    // Create a SharedArrayBuffer and initialize an Int32Array with it
    let sab = SharedArrayBuffer::new(4);
    let int32_array = Int32Array::new(&sab);

    // Set an arbitrary value at index 0
    int32_array.set_index(0, 0);

    // Calculate timeout in milliseconds
    let timeout = duration.as_millis() as f64;

    // Call Atomics.wait to simulate a blocking sleep
    let _result = atomics_wait(&int32_array, 0, 0, timeout);
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
fn js_value_to_io_error(js_value: wasm_bindgen::JsValue) -> Error {
  use wasm_bindgen::JsCast;

  // Check if the error is a Deno.errors.NotFound
  if let Some(error_obj) = js_value.dyn_ref::<js_sys::Error>() {
    let error_name = error_obj.name();

    if error_name == "NotFound" {
      return Error::new(
        ErrorKind::NotFound,
        error_obj
          .message()
          .as_string()
          .unwrap_or_else(|| "Unknown error".to_string()),
      );
    } else if error_name == "AlreadyExists" {
      return Error::new(
        ErrorKind::AlreadyExists,
        error_obj
          .message()
          .as_string()
          .unwrap_or_else(|| "Unknown error".to_string()),
      );
    } else if let Some(message) = error_obj.message().as_string() {
      return Error::new(ErrorKind::Other, message);
    }
  }

  // Fallback for unknown error types
  if let Some(err_msg) = js_value.as_string() {
    Error::new(ErrorKind::Other, err_msg)
  } else {
    Error::new(ErrorKind::Other, "An unknown JavaScript error occurred")
  }
}

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
pub fn wasm_string_to_path(path: String) -> PathBuf {
  #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
  {
    // one day we might have:
    // but for now, do this hack for windows users
    if is_windows() {
      PathBuf::from("/").join(path.replace("\\", "/"))
    } else {
      PathBuf::from(path)
    }
  }
  #[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
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
pub fn wasm_path_to_str(path: &Path) -> Cow<str> {
  #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
  {
    if is_windows() {
      let path = path.to_string_lossy();
      let path = path.strip_prefix('/').unwrap_or(&path);
      Cow::Owned(path.replace("\\", "/"))
    } else {
      path.to_string_lossy()
    }
  }
  #[cfg(all(not(target_arch = "wasm32"), not(feature = "wasm")))]
  {
    path.to_string_lossy()
  }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[inline]
fn is_windows() -> bool {
  build_os() == Os::Windows
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
  Windows,
  Mac,
  Linux,
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
fn build_os() -> Os {
  static BUILD_OS: std::sync::OnceLock<Os> = std::sync::OnceLock::new();

  *BUILD_OS.get_or_init(|| {
    let os = js_sys::Reflect::get(&BUILD, &JsValue::from_str("os")).unwrap();
    match os.as_string().unwrap().as_str() {
      "windows" => Os::Windows,
      "mac" => Os::Mac,
      _ => Os::Linux,
    }
  })
}

#[cfg(all(windows, feature = "winapi"))]
fn known_folder(folder_id: *const windows_sys::core::GUID) -> Option<PathBuf> {
  use std::ffi::c_void;
  use std::os::windows::ffi::OsStringExt;
  use windows_sys::Win32::Foundation::S_OK;
  use windows_sys::Win32::Globalization::lstrlenW;
  use windows_sys::Win32::System::Com::CoTaskMemFree;
  use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;

  // SAFETY: winapi calls
  unsafe {
    let mut path_ptr = std::ptr::null_mut();
    let result =
      SHGetKnownFolderPath(folder_id, 0, std::ptr::null_mut(), &mut path_ptr);
    if result != S_OK {
      return None;
    }
    let len = lstrlenW(path_ptr) as usize;
    let path = std::slice::from_raw_parts(path_ptr, len);
    let ostr: OsString = OsStringExt::from_wide(path);
    CoTaskMemFree(path_ptr as *mut c_void);
    Some(PathBuf::from(ostr))
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[cfg(any(feature = "winapi", feature = "libc"))]
  #[test]
  fn test_known_folders() {
    assert!(RealSys.env_cache_dir().is_some());
    assert!(RealSys.env_home_dir().is_some());
  }

  #[test]
  fn test_general() {
    assert!(RealSys.sys_time_now().elapsed().is_ok());
  }
}
