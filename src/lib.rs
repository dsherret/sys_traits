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

pub mod boxed;
pub mod impls;

// Reasonings:
// 1. Why separate trait for implementation and use?
//    - This is to allow boxing an Impl trait because stuff like `impl AsRef<Path>`
//      can't be boxed.

// #### ENVIRONMENT ####

// == EnvCurrentDir ==

pub trait EnvCurrentDir {
  fn env_current_dir(&self) -> std::io::Result<PathBuf>;
}

// == EnvSetCurrentDir ==

pub trait EnvSetCurrentDirImpl {
  fn env_set_current_dir_impl(&self, path: &Path) -> std::io::Result<()>;
}

pub trait EnvSetCurrentDir: EnvSetCurrentDirImpl {
  #[inline]
  fn env_set_current_dir(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.env_set_current_dir_impl(path.as_ref())
  }
}

impl<T: EnvSetCurrentDirImpl> EnvSetCurrentDir for T {}

// == EnvVar ==

pub trait EnvVarImpl {
  fn env_var_os_impl(&self, key: &OsStr) -> Option<OsString>;
}

pub trait EnvVar: EnvVarImpl {
  #[inline]
  fn env_var_os(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
    self.env_var_os_impl(key.as_ref())
  }

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
        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
        {
          impls::wasm_string_to_path(value.to_string_lossy().to_string())
        }
        #[cfg(any(not(target_arch = "wasm32"), not(feature = "wasm")))]
        {
          PathBuf::from(value)
        }
      })
  }
}

impl<T: EnvVarImpl> EnvVar for T {}

// == EnvSetVar ==

pub trait EnvSetVarImpl {
  fn env_set_var_impl(&self, key: &OsStr, value: &OsStr);
}

pub trait EnvSetVar: EnvSetVarImpl {
  fn env_set_var(&self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
    self.env_set_var_impl(key.as_ref(), value.as_ref())
  }
}

impl<T: EnvSetVarImpl> EnvSetVar for T {}

// == EnvCacheDir ==

pub trait EnvCacheDir {
  fn env_cache_dir(&self) -> Option<PathBuf>;
}

// == EnvHomeDir ==

pub trait EnvHomeDir {
  fn env_home_dir(&self) -> Option<PathBuf>;
}

// #### FILE SYSTEM ####

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
  /// Unix only. Ignored on Windows.
  pub mode: Option<u32>,
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
      mode: None,
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
      mode: None,
    }
  }
}

// == FsCanonicalize ==

pub trait FsCanonicalizeImpl {
  fn fs_canonicalize_impl(&self, path: &Path) -> std::io::Result<PathBuf>;
}

pub trait FsCanonicalize: FsCanonicalizeImpl {
  #[inline]
  fn fs_canonicalize(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<PathBuf> {
    self.fs_canonicalize_impl(path.as_ref())
  }
}

impl<T: FsCanonicalizeImpl> FsCanonicalize for T {}

// == FsCreateDirAll ==

pub trait FsCreateDirAllImpl {
  fn fs_create_dir_all_impl(&self, path: &Path) -> std::io::Result<()>;
}

pub trait FsCreateDirAll: FsCreateDirAllImpl {
  fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.fs_create_dir_all_impl(path.as_ref())
  }
}

impl<T: FsCreateDirAllImpl> FsCreateDirAll for T {}

// == FsHardLink ==

pub trait FsHardLinkImpl {
  fn fs_hard_link_impl(&self, src: &Path, dst: &Path) -> std::io::Result<()>;
}

pub trait FsHardLink: FsHardLinkImpl {
  fn fs_hard_link(
    &self,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.fs_hard_link_impl(src.as_ref(), dst.as_ref())
  }
}

impl<T: FsHardLinkImpl> FsHardLink for T {}

// == FsMetadata ==

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

pub trait FsMetadataImpl {
  type Metadata: FsMetadataValue;

  fn fs_metadata_impl(&self, path: &Path) -> std::io::Result<Self::Metadata>;

  fn fs_symlink_metadata_impl(
    &self,
    path: &Path,
  ) -> std::io::Result<Self::Metadata>;
}

/// These two functions are so cloesly related that it becomes verbose to
/// separate them out into two traits.
pub trait FsMetadata: FsMetadataImpl {
  #[inline]
  fn fs_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::Metadata> {
    self.fs_metadata_impl(path.as_ref())
  }

  #[inline]
  fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::Metadata> {
    self.fs_symlink_metadata_impl(path.as_ref())
  }

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

impl<T: FsMetadataImpl> FsMetadata for T {}

// == FsOpen ==

pub trait FsFile:
  std::io::Read + std::io::Write + std::io::Seek + FsFileSetPermissions
{
}

pub trait FsOpenImpl {
  // ideally this wouldn't be constrained, but by not doing
  // this then the type parameters get really out of hand
  type File: FsFile;

  fn fs_open_impl(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File>;
}

pub trait FsOpen: FsOpenImpl {
  #[inline]
  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File> {
    self.fs_open_impl(path.as_ref(), options)
  }
}

impl<T: FsOpenImpl> FsOpen for T {}

// == FsRead ==

pub trait FsReadImpl {
  fn fs_read_impl(&self, path: &Path) -> std::io::Result<Cow<'static, [u8]>>;
}

pub trait FsRead: FsReadImpl {
  #[inline]
  fn fs_read(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, [u8]>> {
    self.fs_read_impl(path.as_ref())
  }

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

impl<T: FsReadImpl> FsRead for T {}

// == FsReadDir ==

pub trait FsDirEntry: std::fmt::Debug {
  type Metadata: FsMetadataValue;

  fn file_name(&self) -> Cow<OsStr>;
  fn file_type(&self) -> std::io::Result<FileType>;
  fn metadata(&self) -> std::io::Result<Self::Metadata>;
  fn path(&self) -> Cow<Path>;
}

pub trait FsReadDirImpl {
  type ReadDirEntry: FsDirEntry + 'static;

  fn fs_read_dir_impl(
    &self,
    path: &Path,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  >;
}

pub trait FsReadDir: FsReadDirImpl {
  #[inline]
  fn fs_read_dir(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  > {
    self.fs_read_dir_impl(path.as_ref())
  }
}

impl<T: FsReadDirImpl> FsReadDir for T {}

// == FsRemoveDirAll ==

pub trait FsRemoveDirAllImpl {
  fn fs_remove_dir_all_impl(&self, path: &Path) -> std::io::Result<()>;
}

pub trait FsRemoveDirAll: FsRemoveDirAllImpl {
  #[inline]
  fn fs_remove_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.fs_remove_dir_all_impl(path.as_ref())
  }
}

impl<T: FsRemoveDirAllImpl> FsRemoveDirAll for T {}

// == FsRemoveFile ==

pub trait FsRemoveFileImpl {
  fn fs_remove_file_impl(&self, path: &Path) -> std::io::Result<()>;
}

pub trait FsRemoveFile: FsRemoveFileImpl {
  #[inline]
  fn fs_remove_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.fs_remove_file_impl(path.as_ref())
  }
}

impl<T: FsRemoveFileImpl> FsRemoveFile for T {}

// == FsRename ==

pub trait FsRenameImpl {
  fn fs_rename_impl(&self, from: &Path, to: &Path) -> std::io::Result<()>;
}

pub trait FsRename: FsRenameImpl {
  #[inline]
  fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.fs_rename_impl(from.as_ref(), to.as_ref())
  }
}

impl<T: FsRenameImpl> FsRename for T {}

// == FsSymlinkDir ==

pub trait FsSymlinkDirImpl {
  fn fs_symlink_dir_impl(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()>;
}

pub trait FsSymlinkDir: FsSymlinkDirImpl {
  #[inline]
  fn fs_symlink_dir(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.fs_symlink_dir_impl(original.as_ref(), link.as_ref())
  }
}

impl<T: FsSymlinkDirImpl> FsSymlinkDir for T {}

// == FsSymlinkFile ==

pub trait FsSymlinkFileImpl {
  fn fs_symlink_file_impl(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()>;
}

pub trait FsSymlinkFile: FsSymlinkFileImpl {
  #[inline]
  fn fs_symlink_file(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.fs_symlink_file_impl(original.as_ref(), link.as_ref())
  }
}

impl<T: FsSymlinkFileImpl> FsSymlinkFile for T {}

// == FsWrite ==

pub trait FsWriteImpl {
  fn fs_write_impl(&self, path: &Path, data: &[u8]) -> std::io::Result<()>;
}

pub trait FsWrite: FsWriteImpl {
  #[inline]
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()> {
    self.fs_write_impl(path.as_ref(), data.as_ref())
  }
}

impl<T: FsWriteImpl> FsWrite for T {}

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
