use std::borrow::Cow;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use super::strip_unc_prefix;
use super::wasm_path_to_str;
use super::wasm_string_to_path;
use super::RealSys;
use crate::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

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
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = umask, catch)]
  fn deno_umask() -> std::result::Result<u32, JsValue>;
  #[wasm_bindgen(js_namespace = ["Deno"], js_name = umask, catch)]
  fn deno_set_umask(value: u32) -> std::result::Result<u32, JsValue>;
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

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen(module = "node:os")]
extern "C" {
  #[wasm_bindgen(js_name = tmpdir, catch)]
  fn node_tmpdir() -> std::result::Result<String, JsValue>;
}

// ==== Environment ====

impl EnvCurrentDir for RealSys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    deno_cwd()
      .map(wasm_string_to_path)
      .map_err(|err| js_value_to_io_error(err))
  }
}

impl BaseEnvSetCurrentDir for RealSys {
  fn base_env_set_current_dir(&self, path: &Path) -> std::io::Result<()> {
    deno_chdir(&wasm_path_to_str(path)).map_err(js_value_to_io_error)
  }
}

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

impl EnvUmask for RealSys {
  fn env_umask(&self) -> std::io::Result<u32> {
    deno_umask().map_err(js_value_to_io_error)
  }
}

impl EnvSetUmask for RealSys {
  fn env_set_umask(&self, umask: u32) -> std::io::Result<u32> {
    deno_set_umask(umask).map_err(js_value_to_io_error)
  }
}

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

impl EnvHomeDir for RealSys {
  fn env_home_dir(&self) -> Option<PathBuf> {
    if is_windows() {
      self.env_var_path("USERPROFILE")
    } else {
      self.env_var_path("HOME")
    }
  }
}

impl EnvTempDir for RealSys {
  fn env_temp_dir(&self) -> std::io::Result<PathBuf> {
    node_tmpdir()
      .map(wasm_string_to_path)
      .map(strip_unc_prefix)
      .map_err(js_value_to_io_error)
  }
}

// ==== File System ====

impl BaseFsCanonicalize for RealSys {
  fn base_fs_canonicalize(&self, path: &Path) -> Result<PathBuf> {
    deno_real_path_sync(&wasm_path_to_str(path))
      .map(wasm_string_to_path)
      .map(strip_unc_prefix)
      .map_err(js_value_to_io_error)
  }
}

impl BaseFsCreateDir for RealSys {
  fn base_fs_create_dir(
    &self,
    path: &Path,
    options: &CreateDirOptions,
  ) -> Result<()> {
    let path_str = wasm_path_to_str(path);

    // Create the options object for mkdirSync
    let js_options = js_sys::Object::new();
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("recursive"),
      &JsValue::from_bool(options.recursive),
    )
    .map_err(|e| js_value_to_io_error(e))?;
    if let Some(mode) = options.mode {
      js_sys::Reflect::set(
        &js_options,
        &JsValue::from_str("mode"),
        &mode.into(),
      )
      .map_err(|e| js_value_to_io_error(e))?;
    }

    // Call the Deno.mkdirSync function
    deno_mkdir_sync(&path_str, &JsValue::from(js_options))
      .map_err(|e| js_value_to_io_error(e))
  }
}

impl BaseFsHardLink for RealSys {
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
    let src_str = wasm_path_to_str(src);
    let dst_str = wasm_path_to_str(dst);

    deno_link_sync(&src_str, &dst_str).map_err(js_value_to_io_error)
  }
}

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

#[derive(Debug, Clone)]
pub struct WasmMetadata(JsValue);

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

fn parse_date(value: &JsValue) -> Result<SystemTime> {
  let date = value
    .dyn_ref::<js_sys::Date>()
    .ok_or_else(|| Error::new(ErrorKind::Other, "value not a date"))?;
  let ms = date.get_time() as u64;
  Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ms))
}

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

impl BaseFsRead for RealSys {
  fn base_fs_read(&self, path: &Path) -> Result<Cow<'static, [u8]>> {
    let s = wasm_path_to_str(path);
    let v = deno_read_file_sync(&s).map_err(js_value_to_io_error)?;
    let b = js_sys::Uint8Array::new(&v).to_vec();
    Ok(Cow::Owned(b))
  }
}

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

#[derive(Debug)]
pub struct WasmFsDirEntry {
  parent_path: PathBuf,
  value: JsValue,
}

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

impl BaseFsRemoveFile for RealSys {
  fn base_fs_remove_file(&self, path: &Path) -> std::io::Result<()> {
    let s = wasm_path_to_str(path);
    deno_remove_sync(&s).map_err(js_value_to_io_error)
  }
}

impl BaseFsRename for RealSys {
  fn base_fs_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
    let f = wasm_path_to_str(from);
    let t = wasm_path_to_str(to);
    deno_rename_sync(&f, &t).map_err(js_value_to_io_error)
  }
}

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

impl BaseFsWrite for RealSys {
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
    let s = wasm_path_to_str(path);
    deno_write_file_sync(&s, data).map_err(js_value_to_io_error)
  }
}

// ==== File System File ====

#[derive(Debug)]
pub struct WasmFile {
  file: DenoFsFile,
  path: String,
}

impl Drop for WasmFile {
  fn drop(&mut self) {
    self.file.close_internal();
  }
}

impl FsFile for WasmFile {}

impl FsFileSetPermissions for WasmFile {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()> {
    if is_windows() {
      return Ok(()); // ignore
    }
    deno_chmod_sync(&self.path, mode).map_err(js_value_to_io_error)
  }
}

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

impl std::io::Read for WasmFile {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self
      .file
      .read_sync_internal(buf)
      .map_err(js_value_to_io_error)
  }
}

// ==== System ====

impl SystemTimeNow for RealSys {
  #[inline]
  fn sys_time_now(&self) -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(date_now() as u64)
  }
}

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
#[inline]
pub(super) fn is_windows() -> bool {
  build_os() == Os::Windows
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Os {
  Windows,
  Mac,
  Linux,
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub(super) fn build_os() -> Os {
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

fn js_value_to_io_error(js_value: wasm_bindgen::JsValue) -> Error {
  use wasm_bindgen::JsCast;

  // Check if the error is a Deno.errors.NotFound
  if let Some(error_obj) = js_value.dyn_ref::<js_sys::Error>() {
    let error_name = error_obj.name();

    let maybe_kind = if error_name == "NotFound" {
      Some(ErrorKind::NotFound)
    } else if error_name == "AlreadyExists" {
      Some(ErrorKind::AlreadyExists)
    } else if error_name == "NotSupported" {
      Some(ErrorKind::Unsupported)
    } else {
      None
    };

    if let Some(error_kind) = maybe_kind {
      return Error::new(
        error_kind,
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
