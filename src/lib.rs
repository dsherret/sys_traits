use std::borrow::Cow;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
  File,
  Dir,
  Symlink,
  Unknown,
}

pub trait FsMetadataValue {
  fn file_type(&self) -> FileType;
  fn modified(&self) -> std::io::Result<SystemTime>;
}

pub trait FsMetadata {
  type MetadataValue: FsMetadataValue;

  fn fs_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::MetadataValue>;

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
}

pub trait FsSymlinkMetadata {
  type MetadataValue: FsMetadataValue;

  fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::MetadataValue>;

  fn fs_exists(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    match self.fs_symlink_metadata(path) {
      Ok(_) => Ok(true),
      Err(err) => {
        if err.kind() == std::io::ErrorKind::NotFound {
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
}

pub trait FsReadToString {
  fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, str>>;
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
