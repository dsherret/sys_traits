use core::str;
use std::borrow::Cow;
use std::env::VarError;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

pub mod impls;

// Environment

pub trait EnvCurrentDir {
  fn env_current_dir(&self) -> std::io::Result<PathBuf>;
}

pub trait EnvSetCurrentDir {
  fn env_set_current_dir(&self, path: impl AsRef<Path>) -> std::io::Result<()>;
}

pub trait EnvVar {
  fn env_var_os(&self, key: impl AsRef<OsStr>) -> Option<OsString>;

  fn env_var(&self, key: impl AsRef<OsStr>) -> Result<String, VarError> {
    match self.env_var_os(key) {
      Some(val) => val.into_string().map_err(VarError::NotUnicode),
      None => Err(VarError::NotPresent),
    }
  }

  /// Helper to get a path from an environment variable.
  fn env_var_path(&self, key: impl AsRef<OsStr>) -> Option<PathBuf> {
    self
      .env_var_os(key)
      .and_then(|h| if h.is_empty() { None } else { Some(h) })
      .map(|value| {
        #[cfg(target_arch = "wasm32")]
        {
          impls::wasm_string_to_path(value.to_string_lossy().to_string())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
          PathBuf::from(value)
        }
      })
  }
}

pub trait EnvSetVar {
  fn env_set_var(&self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>);
}

pub trait EnvCacheDir {
  fn env_cache_dir(&self) -> Option<PathBuf>;
}

pub trait EnvHomeDir {
  fn env_home_dir(&self) -> Option<PathBuf>;
}

// File System

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "camelCase"))]
pub struct OpenOptions {
  pub read: bool,
  pub write: bool,
  pub create: bool,
  pub truncate: bool,
  pub append: bool,
  pub create_new: bool,
}

impl OpenOptions {
  pub fn read() -> Self {
    Self {
      read: true,
      write: false,
      create: false,
      truncate: false,
      append: false,
      create_new: false,
    }
  }

  pub fn write() -> Self {
    Self {
      read: false,
      write: true,
      create: true,
      truncate: true,
      append: false,
      create_new: false,
    }
  }
}

pub trait FsCanonicalize {
  fn fs_canonicalize(&self, path: impl AsRef<Path>)
    -> std::io::Result<PathBuf>;
}

pub trait FsCreateDirAll {
  fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
  File,
  Dir,
  Symlink,
  Unknown,
}

impl FileType {
  pub fn is_dir(&self) -> bool {
    *self == Self::Dir
  }

  pub fn is_file(&self) -> bool {
    *self == Self::File
  }

  pub fn is_symlink(&self) -> bool {
    *self == Self::Symlink
  }
}

impl From<std::fs::FileType> for FileType {
  fn from(file_type: std::fs::FileType) -> Self {
    if file_type.is_file() {
      FileType::File
    } else if file_type.is_dir() {
      FileType::Dir
    } else if file_type.is_symlink() {
      FileType::Symlink
    } else {
      FileType::Unknown
    }
  }
}

pub trait FsMetadataValue: std::fmt::Debug {
  fn file_type(&self) -> FileType;
  fn modified(&self) -> std::io::Result<SystemTime>;
}

/// These two functions are so cloesly related that it becomes verbose to
/// separate them out into two traits.
pub trait FsMetadata {
  type Metadata: FsMetadataValue;

  fn fs_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::Metadata>;

  fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::Metadata>;

  fn fs_is_file(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    Ok(self.fs_metadata(path)?.file_type() == FileType::File)
  }

  fn fs_is_file_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_is_file(path).unwrap_or(false)
  }

  fn fs_is_dir(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    Ok(self.fs_metadata(path)?.file_type() == FileType::Dir)
  }

  fn fs_is_dir_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_is_dir(path).unwrap_or(false)
  }

  fn fs_exists(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    match self.fs_symlink_metadata(path) {
      Ok(_) => Ok(true),
      Err(err) => {
        if err.kind() == ErrorKind::NotFound {
          Ok(false)
        } else {
          Err(err)
        }
      }
    }
  }

  fn fs_exists_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_exists(path).unwrap_or(false)
  }

  fn fs_is_symlink(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    Ok(self.fs_symlink_metadata(path)?.file_type() == FileType::Symlink)
  }

  fn fs_is_symlink_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_is_symlink(path).unwrap_or(false)
  }
}

pub trait FsFile:
  std::io::Read + std::io::Write + FsFileSetPermissions
{
}

pub trait FsOpen {
  type File: FsFile;

  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File>;
}

pub trait FsRead {
  fn fs_read(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, [u8]>>;

  fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, str>> {
    let bytes = self.fs_read(path)?;
    match bytes {
      Cow::Borrowed(bytes) => str::from_utf8(bytes)
        .map(Cow::Borrowed)
        .map_err(|e| e.to_string()),
      Cow::Owned(bytes) => String::from_utf8(bytes)
        .map(Cow::Owned)
        .map_err(|e| e.to_string()),
    }
    .map_err(|error_text| Error::new(ErrorKind::InvalidData, error_text))
  }

  fn fs_read_to_string_lossy(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, str>> {
    // Like String::from_utf8_lossy but operates on owned values
    #[inline(always)]
    fn string_from_utf8_lossy(buf: Vec<u8>) -> String {
      match String::from_utf8_lossy(&buf) {
        // buf contained non-utf8 chars than have been patched
        Cow::Owned(s) => s,
        // SAFETY: if Borrowed then the buf only contains utf8 chars,
        // we do this instead of .into_owned() to avoid copying the input buf
        Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(buf) },
      }
    }

    let bytes = self.fs_read(path)?;
    match bytes {
      Cow::Borrowed(bytes) => Ok(String::from_utf8_lossy(bytes)),
      Cow::Owned(bytes) => Ok(Cow::Owned(string_from_utf8_lossy(bytes))),
    }
  }
}

pub trait FsDirEntry: std::fmt::Debug {
  type Metadata: FsMetadataValue;

  fn file_name(&self) -> Cow<OsStr>;
  fn file_type(&self) -> std::io::Result<FileType>;
  fn metadata(&self) -> std::io::Result<Self::Metadata>;
  fn path(&self) -> Cow<Path>;
}

pub trait FsReadDir {
  type ReadDirEntry: FsDirEntry;

  fn fs_read_dir(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<impl Iterator<Item = std::io::Result<Self::ReadDirEntry>>>;
}

pub trait FsRemoveDirAll {
  fn fs_remove_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()>;
}

pub trait FsRemoveFile {
  fn fs_remove_file(&self, path: impl AsRef<Path>) -> std::io::Result<()>;
}

pub trait FsRename {
  fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<()>;
}

pub trait FsSymlinkDir {
  fn fs_symlink_dir(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()>;
}

pub trait FsSymlinkFile {
  fn fs_symlink_file(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()>;
}

pub trait FsWrite {
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()>;
}

// File System File

pub trait FsFileSetPermissions {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()>;
}

// System

pub trait SystemTimeNow {
  fn sys_time_now(&self) -> std::time::SystemTime;
}

pub trait SystemRandom {
  fn sys_random(&self, buf: &mut [u8]) -> std::io::Result<()>;

  fn sys_random_u8(&self) -> std::io::Result<u8> {
    let mut buf = [0; 1];
    self.sys_random(&mut buf)?;
    Ok(buf[0])
  }

  fn sys_random_u32(&self) -> std::io::Result<u32> {
    let mut buf = [0; 4];
    self.sys_random(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
  }

  fn sys_random_u64(&self) -> std::io::Result<u64> {
    let mut buf = [0; 8];
    self.sys_random(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
  }
}

pub trait ThreadSleep {
  fn thread_sleep(&self, duration: std::time::Duration);
}
