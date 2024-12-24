use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

pub mod impls;

// Environment

pub trait EnvCurrentDir {
  fn env_current_dir(&self) -> std::io::Result<PathBuf>;
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

pub trait FsExists {
  fn fs_exists(&self, path: impl AsRef<Path>) -> std::io::Result<bool>;

  fn fs_exists_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_exists(path).unwrap_or(false)
  }
}

pub trait FsIsFile {
  fn fs_is_file(&self, path: impl AsRef<Path>) -> std::io::Result<bool>;

  fn fs_is_file_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_is_file(path).unwrap_or(false)
  }
}

pub trait FsIsDir {
  fn fs_is_dir(&self, path: impl AsRef<Path>) -> std::io::Result<bool>;

  fn fs_is_dir_no_err(&self, path: impl AsRef<Path>) -> bool {
    self.fs_is_dir(path).unwrap_or(false)
  }
}

pub trait FsModified {
  /// First result is the metadata result, second result is the `metadata.modified()` result.
  fn fs_modified(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<std::io::Result<std::time::SystemTime>>;
}

pub trait FsOpen<TFile> {
  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<TFile>;
}

pub trait FsRead {
  fn fs_read(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, [u8]>>;
}

pub trait FsReadToString {
  fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, str>>;
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

pub trait FsFileWrite {
  fn fs_file_write_all(
    &mut self,
    write: impl AsRef<[u8]>,
  ) -> std::io::Result<()>;
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
