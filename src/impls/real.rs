use std::borrow::Cow;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

#[cfg(target_arch = "wasm32")]
use std::io::Error;
#[cfg(target_arch = "wasm32")]
use std::io::ErrorKind;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use crate::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct RealSys;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = chmodSync, catch)]
  fn deno_chmod_sync(path: &str, mode: u32)
    -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = chdir, catch)]
  fn deno_chdir(path: &str) -> std::result::Result<(), JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = cwd, catch)]
  fn deno_cwd() -> std::result::Result<String, JsValue>;
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
  #[wasm_bindgen(method, structural, js_name = writeSync)]
  fn write_sync_internal(this: &DenoFsFile, data: &[u8]) -> usize;
  #[wasm_bindgen(method, structural, js_name = syncSync)]
  fn sync_internal(this: &DenoFsFile);
  #[wasm_bindgen(method, structural, js_name = readSync)]
  fn read_sync_internal(this: &DenoFsFile, buffer: &mut [u8]) -> Option<usize>;

  // Deno.build
  #[wasm_bindgen(js_namespace = Deno, js_name = build)]
  static BUILD: JsValue;

  // Deno.env
  #[wasm_bindgen(js_namespace = Deno, js_name = env)]
  static ENV: JsValue;
}

// ==== Environment ====

#[cfg(not(target_arch = "wasm32"))]
impl EnvCurrentDir for RealSys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    std::env::current_dir()
  }
}

#[cfg(target_arch = "wasm32")]
impl EnvCurrentDir for RealSys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    deno_cwd()
      .map(wasm_string_to_path)
      .map_err(|err| js_value_to_io_error(err))
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl EnvSetCurrentDir for RealSys {
  fn env_set_current_dir(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::env::set_current_dir(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl EnvSetCurrentDir for RealSys {
  fn env_set_current_dir(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    deno_chdir(&wasm_path_to_str(path.as_ref())).map_err(js_value_to_io_error)
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl EnvVar for RealSys {
  fn env_var_os(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
    std::env::var_os(key)
  }
}

#[cfg(target_arch = "wasm32")]
impl EnvVar for RealSys {
  fn env_var_os(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
    let key = key.as_ref().to_str()?;
    let get_fn = js_sys::Reflect::get(&ENV, &JsValue::from_str("get"))
      .ok()
      .and_then(|v| v.dyn_into::<js_sys::Function>().ok())?;
    let key_js = JsValue::from_str(key);
    let value_js = get_fn.call1(&ENV, &key_js).ok()?;
    return value_js.as_string().map(OsString::from);
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl EnvSetVar for RealSys {
  fn env_set_var(&self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
    std::env::set_var(key, value);
  }
}

#[cfg(target_arch = "wasm32")]
impl EnvSetVar for RealSys {
  fn env_set_var(&self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
    let key = key.as_ref().to_str().unwrap();
    let value = value.as_ref().to_str().unwrap();
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
impl FsCanonicalize for RealSys {
  #[inline]
  fn fs_canonicalize(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
    std::fs::canonicalize(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsCanonicalize for RealSys {
  fn fs_canonicalize(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
    deno_real_path_sync(&wasm_path_to_str(path.as_ref()))
      .map(wasm_string_to_path)
      .map_err(js_value_to_io_error)
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsCreateDirAll for RealSys {
  #[inline]
  fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> Result<()> {
    std::fs::create_dir_all(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsCreateDirAll for RealSys {
  fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> Result<()> {
    let path_str = wasm_path_to_str(path.as_ref());

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

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WasmMetadata(JsValue);

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
impl FsMetadata for RealSys {
  type Metadata = RealFsMetadata;

  #[inline]
  fn fs_metadata(&self, path: impl AsRef<Path>) -> Result<Self::Metadata> {
    std::fs::metadata(path).map(RealFsMetadata)
  }

  #[inline]
  fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> Result<Self::Metadata> {
    std::fs::symlink_metadata(path).map(RealFsMetadata)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsMetadata for RealSys {
  type Metadata = WasmMetadata;

  #[inline]
  fn fs_metadata(&self, path: impl AsRef<Path>) -> Result<WasmMetadata> {
    let s = wasm_path_to_str(path.as_ref());
    match deno_stat_sync(&s) {
      Ok(v) => Ok(WasmMetadata(v)),
      Err(e) => Err(js_value_to_io_error(e)),
    }
  }

  #[inline]
  fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> Result<WasmMetadata> {
    let s = wasm_path_to_str(path.as_ref());
    match deno_lstat_sync(&s) {
      Ok(v) => Ok(WasmMetadata(v)),
      Err(e) => Err(js_value_to_io_error(e)),
    }
  }
}

#[cfg(target_arch = "wasm32")]
fn parse_date(value: &JsValue) -> Result<SystemTime> {
  let date = value
    .dyn_ref::<js_sys::Date>()
    .ok_or_else(|| Error::new(ErrorKind::Other, "value not a date"))?;
  let ms = date.get_time() as u64;
  Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ms))
}

#[cfg(not(target_arch = "wasm32"))]
impl FsOpen for RealSys {
  type File = RealFsFile;

  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File> {
    let mut builder = std::fs::OpenOptions::new();
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

#[cfg(target_arch = "wasm32")]
impl FsOpen for RealSys {
  type File = WasmFile;

  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<WasmFile> {
    let s = wasm_path_to_str(path.as_ref()).into_owned();
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

#[cfg(not(target_arch = "wasm32"))]
impl FsRead for RealSys {
  #[inline]
  fn fs_read(&self, path: impl AsRef<Path>) -> Result<Cow<'static, [u8]>> {
    std::fs::read(path).map(Cow::Owned)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsRead for RealSys {
  fn fs_read(&self, path: impl AsRef<Path>) -> Result<Cow<'static, [u8]>> {
    let s = wasm_path_to_str(path.as_ref());
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

#[cfg(not(target_arch = "wasm32"))]
impl FsReadDir for RealSys {
  type ReadDirEntry = RealFsDirEntry;

  #[inline]
  fn fs_read_dir(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<impl Iterator<Item = std::io::Result<Self::ReadDirEntry>>>
  {
    let iterator = std::fs::read_dir(path)?;
    Ok(iterator.map(|result| result.map(RealFsDirEntry)))
  }
}

#[cfg(target_arch = "wasm32")]
impl FsReadDir for RealSys {
  type ReadDirEntry = WasmFsDirEntry;

  fn fs_read_dir(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<impl Iterator<Item = std::io::Result<Self::ReadDirEntry>>>
  {
    let path_str = wasm_path_to_str(path.as_ref());

    // Use Deno.readDirSync to get directory entries
    let entries =
      deno_read_dir_sync(&path_str).map_err(js_value_to_io_error)?;

    let path = path.as_ref().to_path_buf();
    Ok(entries.into_iter().map(move |entry| {
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
    }))
  }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct WasmFsDirEntry {
  parent_path: PathBuf,
  value: JsValue,
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
impl FsRemoveDirAll for RealSys {
  fn fs_remove_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::remove_dir_all(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsRemoveDirAll for RealSys {
  fn fs_remove_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    let s = wasm_path_to_str(path.as_ref());
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

#[cfg(not(target_arch = "wasm32"))]
impl FsRemoveFile for RealSys {
  fn fs_remove_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::remove_file(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsRemoveFile for RealSys {
  fn fs_remove_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    let s = wasm_path_to_str(path.as_ref());
    deno_remove_sync(&s).map_err(js_value_to_io_error)
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsRename for RealSys {
  fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    std::fs::rename(from, to)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsRename for RealSys {
  fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    let f = wasm_path_to_str(from.as_ref());
    let t = wasm_path_to_str(to.as_ref());
    deno_rename_sync(&f, &t).map_err(js_value_to_io_error)
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsSymlinkDir for RealSys {
  fn fs_symlink_dir(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
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

#[cfg(target_arch = "wasm32")]
impl FsSymlinkDir for RealSys {
  fn fs_symlink_dir(
    &self,
    original: impl AsRef<std::path::Path>,
    link: impl AsRef<std::path::Path>,
  ) -> std::io::Result<()> {
    let old_path = wasm_path_to_str(original.as_ref());
    let new_path = wasm_path_to_str(link.as_ref());

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

#[cfg(not(target_arch = "wasm32"))]
impl FsSymlinkFile for RealSys {
  fn fs_symlink_file(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
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

#[cfg(target_arch = "wasm32")]
impl FsSymlinkFile for RealSys {
  fn fs_symlink_file(
    &self,
    original: impl AsRef<std::path::Path>,
    link: impl AsRef<std::path::Path>,
  ) -> std::io::Result<()> {
    let old_path = wasm_path_to_str(original.as_ref());
    let new_path = wasm_path_to_str(link.as_ref());

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

#[cfg(not(target_arch = "wasm32"))]
impl FsWrite for RealSys {
  #[inline]
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()> {
    std::fs::write(path, data)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsWrite for RealSys {
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()> {
    let s = wasm_path_to_str(path.as_ref());
    deno_write_file_sync(&s, data.as_ref()).map_err(js_value_to_io_error)
  }
}

// ==== File System File ====

/// A wrapper type is used in order to force usages to
/// `use sys_traits::FsFile` so that the code
/// compiles under Wasm.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct RealFsFile(std::fs::File);

#[cfg(not(target_arch = "wasm32"))]
impl FsFile for RealFsFile {}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
impl std::io::Read for RealFsFile {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    self.0.read(buf)
  }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct WasmFile {
  file: DenoFsFile,
  path: String,
}

#[cfg(target_arch = "wasm32")]
impl FsFile for WasmFile {}

#[cfg(target_arch = "wasm32")]
impl Drop for WasmFile {
  fn drop(&mut self) {
    self.file.close_internal();
  }
}

#[cfg(target_arch = "wasm32")]
impl FsFileSetPermissions for WasmFile {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()> {
    if is_windows() {
      return Ok(()); // ignore
    }
    deno_chmod_sync(&self.path, mode).map_err(js_value_to_io_error)
  }
}

#[cfg(target_arch = "wasm32")]
impl std::io::Write for WasmFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    Ok(self.file.write_sync_internal(buf))
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.file.sync_internal();
    Ok(())
  }
}

#[cfg(target_arch = "wasm32")]
impl std::io::Read for WasmFile {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    Ok(self.file.read_sync_internal(buf).unwrap_or(0))
  }
}

// ==== System ====

#[cfg(not(target_arch = "wasm32"))]
impl SystemTimeNow for RealSys {
  #[inline]
  fn sys_time_now(&self) -> SystemTime {
    SystemTime::now()
  }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
impl crate::ThreadSleep for RealSys {
  fn thread_sleep(&self, duration: std::time::Duration) {
    std::thread::sleep(duration);
  }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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
  #[cfg(target_arch = "wasm32")]
  {
    // one day we might have:
    // but for now, do this hack for windows users
    if is_windows() {
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

/// Helper that converts a path to a string for Wasm.
///
/// This will convert a path to have backslashes for JS on Windows.
///
/// See notes on `wasm_string_to_path` for more information.
pub fn wasm_path_to_str(path: &Path) -> Cow<str> {
  #[cfg(target_arch = "wasm32")]
  {
    if is_windows() {
      let path = path.to_string_lossy();
      let path = path.strip_prefix('/').unwrap_or(&path);
      Cow::Owned(path.replace("\\", "/"))
    } else {
      path.to_string_lossy()
    }
  }
  #[cfg(not(target_arch = "wasm32"))]
  {
    path.to_string_lossy()
  }
}

#[cfg(target_arch = "wasm32")]
#[inline]
fn is_windows() -> bool {
  build_os() == Os::Windows
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
  Windows,
  Mac,
  Linux,
}

#[cfg(target_arch = "wasm32")]
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
