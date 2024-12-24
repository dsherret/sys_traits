use std::borrow::Cow;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

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
  #[wasm_bindgen(method, structural, js_name = readSync)]
  fn read_sync_internal(this: &DenoFsFile, buffer: &mut [u8]) -> Option<usize>;

  // Deno.build
  #[wasm_bindgen(js_namespace = Deno, js_name = build)]
  static BUILD: JsValue;
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
      .map(string_to_path)
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
    deno_chdir(&path_to_str(path.as_ref())).map_err(js_value_to_io_error)
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
    deno_real_path_sync(&path_to_str(path.as_ref()))
      .map(string_to_path)
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
    let path_str = path_to_str(path.as_ref());

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

#[cfg(not(target_arch = "wasm32"))]
impl FsExists for RealSys {
  #[inline]
  fn fs_exists(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    std::fs::exists(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsExists for RealSys {
  fn fs_exists(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    let path_str = path_to_str(&path.as_ref());

    match deno_lstat_sync(&path_str) {
      Ok(_) => Ok(true),
      Err(err) => {
        let error = js_value_to_io_error(err);
        if error.kind() == std::io::ErrorKind::NotFound {
          Ok(false)
        } else {
          Err(error)
        }
      }
    }
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsIsDir for RealSys {
  #[inline]
  fn fs_is_dir(&self, path: impl AsRef<Path>) -> Result<bool> {
    std::fs::metadata(path).map(|m| m.is_dir())
  }
}

#[cfg(target_arch = "wasm32")]
impl FsIsDir for RealSys {
  fn fs_is_dir(&self, path: impl AsRef<Path>) -> Result<bool> {
    let path_str = path_to_str(path.as_ref());

    match deno_stat_sync(&path_str) {
      Ok(stat_obj) => {
        if let Some(kind) =
          js_sys::Reflect::get(&stat_obj, &JsValue::from_str("isDirectory"))
            .ok()
            .and_then(|v| v.as_bool())
        {
          Ok(kind)
        } else {
          Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to determine if the path is a directory",
          ))
        }
      }
      Err(err) => Err(js_value_to_io_error(err)),
    }
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsIsFile for RealSys {
  #[inline]
  fn fs_is_file(&self, path: impl AsRef<Path>) -> Result<bool> {
    std::fs::metadata(path).map(|m| m.is_file())
  }
}

#[cfg(target_arch = "wasm32")]
impl FsIsFile for RealSys {
  fn fs_is_file(&self, path: impl AsRef<Path>) -> Result<bool> {
    let path_str = path_to_str(path.as_ref());

    match deno_stat_sync(&path_str) {
      Ok(stat_obj) => {
        if let Some(is_file) =
          js_sys::Reflect::get(&stat_obj, &JsValue::from_str("isFile"))
            .ok()
            .and_then(|v| v.as_bool())
        {
          Ok(is_file)
        } else {
          Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to determine if the path is a file",
          ))
        }
      }
      Err(err) => Err(js_value_to_io_error(err)),
    }
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsModified for RealSys {
  fn fs_modified(&self, path: impl AsRef<Path>) -> Result<Result<SystemTime>> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.modified())
  }
}

#[cfg(target_arch = "wasm32")]
impl FsModified for RealSys {
  fn fs_modified(&self, path: impl AsRef<Path>) -> Result<Result<SystemTime>> {
    let s = path_to_str(path.as_ref());
    match deno_stat_sync(&s) {
      Ok(v) => {
        let m = match js_sys::Reflect::get(&v, &JsValue::from_str("mtime")) {
          Ok(m) => m,
          Err(err) => return Ok(Err(js_value_to_io_error(err))),
        };
        if m.is_undefined() || m.is_null() {
          Ok(Err(Error::new(ErrorKind::Other, "mtime not found")))
        } else {
          Ok(parse_date(&m))
        }
      }
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
impl FsOpen<std::fs::File> for RealSys {
  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<std::fs::File> {
    let mut builder = std::fs::OpenOptions::new();
    builder
      .read(options.read)
      .write(options.write)
      .create(options.create)
      .truncate(options.truncate)
      .append(options.append)
      .create_new(options.create_new)
      .open(path)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsOpen<WasmFile> for RealSys {
  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<WasmFile> {
    let s = path_to_str(path.as_ref()).into_owned();
    let js_options = js_sys::Object::new();
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("read"),
      &JsValue::from_bool(options.read),
    );
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("write"),
      &JsValue::from_bool(options.write),
    );
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("create"),
      &JsValue::from_bool(options.create),
    );
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("truncate"),
      &JsValue::from_bool(options.truncate),
    );
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("append"),
      &JsValue::from_bool(options.append),
    );
    js_sys::Reflect::set(
      &js_options,
      &JsValue::from_str("createNew"),
      &JsValue::from_bool(options.create_new),
    );
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
    let s = path_to_str(path.as_ref());
    let v = deno_read_file_sync(&s).map_err(js_value_to_io_error)?;
    let b = js_sys::Uint8Array::new(&v).to_vec();
    Ok(Cow::Owned(b))
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsReadToString for RealSys {
  #[inline]
  fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> Result<Cow<'static, str>> {
    std::fs::read_to_string(path).map(Cow::Owned)
  }
}

#[cfg(target_arch = "wasm32")]
impl FsReadToString for RealSys {
  fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> Result<Cow<'static, str>> {
    let s = path_to_str(path.as_ref());
    let t = deno_read_text_file_sync(&s).map_err(js_value_to_io_error)?;
    Ok(Cow::Owned(t))
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
    let s = path_to_str(path.as_ref());
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
      &options,
      &JsValue::from_str("recursive"),
      &JsValue::from_bool(true),
    );
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
    let s = path_to_str(path.as_ref());
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
    let f = path_to_str(from.as_ref());
    let t = path_to_str(to.as_ref());
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
    let old_path = path_to_str(original.as_ref());
    let new_path = path_to_str(link.as_ref());

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
    let old_path = path_to_str(original.as_ref());
    let new_path = path_to_str(link.as_ref());

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
    let s = path_to_str(path.as_ref());
    deno_write_file_sync(&s, data.as_ref()).map_err(js_value_to_io_error)
  }
}

// ==== File System File ====

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct WasmFile {
  file: DenoFsFile,
  path: String,
}

#[cfg(target_arch = "wasm32")]
impl Drop for WasmFile {
  fn drop(&mut self) {
    self.file.close_internal();
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl FsFileSetPermissions for std::fs::File {
  #[inline]
  fn fs_file_set_permissions(&mut self, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let permissions = std::fs::Permissions::from_mode(mode);
      file.set_permissions(permissions)
    }
    #[cfg(not(unix))]
    {
      let _ = mode;
      Ok(())
    }
  }
}

#[cfg(target_arch = "wasm32")]
impl FsFileSetPermissions for WasmFile {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()> {
    deno_chmod_sync(&self.path, mode).map_err(js_value_to_io_error)
  }
}

#[cfg(target_arch = "wasm32")]
impl std::io::Write for WasmFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    Ok(self.file.write_sync_internal(buf))
  }

  fn flush(&mut self) -> std::io::Result<()> {
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn string_to_path(path: String) -> PathBuf {
  // one day we might have: https://github.com/rust-lang/rust/issues/66621#issuecomment-2561279536
  // but for now, do this hack for windows users
  if is_windows() {
    PathBuf::from(path.replace("\\", "/"))
  } else {
    PathBuf::from(path)
  }
}

#[cfg(target_arch = "wasm32")]
fn path_to_str(path: &Path) -> Cow<str> {
  if is_windows() {
    Cow::Owned(path.to_string_lossy().replace("\\", "/"))
  } else {
    path.to_string_lossy()
  }
}

#[cfg(target_arch = "wasm32")]
fn is_windows() -> bool {
  static IS_WINDOWS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

  *IS_WINDOWS.get_or_init(|| {
    js_sys::Reflect::get(&BUILD, &JsValue::from_str("os")).unwrap() == "windows"
  })
}
