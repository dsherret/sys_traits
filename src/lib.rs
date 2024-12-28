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

// #### ENVIRONMENT ####

// == EnvCurrentDir ==

pub trait EnvCurrentDir {
  fn env_current_dir(&self) -> std::io::Result<PathBuf>;
}

// == EnvSetCurrentDir ==

pub trait BaseEnvSetCurrentDir {
  #[doc(hidden)]
  fn base_env_set_current_dir(&self, path: &Path) -> std::io::Result<()>;
}

pub trait EnvSetCurrentDir: BaseEnvSetCurrentDir {
  #[inline]
  fn env_set_current_dir(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.base_env_set_current_dir(path.as_ref())
  }
}

impl<T: BaseEnvSetCurrentDir> EnvSetCurrentDir for T {}

// == EnvVar ==

pub trait BaseEnvVar {
  #[doc(hidden)]
  fn base_env_var_os(&self, key: &OsStr) -> Option<OsString>;
}

pub trait EnvVar: BaseEnvVar {
  #[inline]
  fn env_var_os(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
    self.base_env_var_os(key.as_ref())
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

impl<T: BaseEnvVar> EnvVar for T {}

// == EnvSetVar ==

pub trait BaseEnvSetVar {
  #[doc(hidden)]
  fn base_env_set_var(&self, key: &OsStr, value: &OsStr);
}

pub trait EnvSetVar: BaseEnvSetVar {
  fn env_set_var(&self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
    self.base_env_set_var(key.as_ref(), value.as_ref())
  }
}

impl<T: BaseEnvSetVar> EnvSetVar for T {}

// == EnvUmask ==

pub trait EnvUmask {
  fn env_umask(&self) -> std::io::Result<u32>;
}

// == EnvSetUmask ==

pub trait EnvSetUmask {
  fn env_set_umask(&self, umask: u32) -> std::io::Result<u32>;
}

// == EnvCacheDir ==

pub trait EnvCacheDir {
  fn env_cache_dir(&self) -> Option<PathBuf>;
}

// == EnvHomeDir ==

pub trait EnvHomeDir {
  fn env_home_dir(&self) -> Option<PathBuf>;
}

// == EnvTempDir ==

pub trait EnvTempDir {
  fn env_temp_dir(&self) -> std::io::Result<PathBuf>;
}

// #### FILE SYSTEM ####

#[cfg(windows)]
type CustomFlagsValue = u32;
#[cfg(not(windows))]
type CustomFlagsValue = i32;

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "camelCase"))]
#[non_exhaustive] // so we can add properties without breaking people
pub struct OpenOptions {
  pub read: bool,
  pub write: bool,
  pub create: bool,
  pub truncate: bool,
  pub append: bool,
  pub create_new: bool,
  /// Unix only. Ignored on Windows.
  pub mode: Option<u32>,
  /// Custom flags to set on Unix or Windows.
  ///
  /// On Windows this is a u32, but on Unix it's an i32.
  ///
  /// Note: only provide flags that make sense for the current operating system.
  pub custom_flags: Option<CustomFlagsValue>,
}

impl OpenOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_read() -> Self {
    Self {
      read: true,
      write: false,
      create: false,
      truncate: false,
      append: false,
      create_new: false,
      mode: None,
      custom_flags: None,
    }
  }

  // todo: make this an instance method in the next version
  #[deprecated(note = "use `new_write` instead")]
  pub fn write() -> Self {
    Self::new_write()
  }

  pub fn new_write() -> Self {
    Self {
      read: false,
      write: true,
      create: true,
      truncate: true,
      append: false,
      create_new: false,
      mode: None,
      custom_flags: None,
    }
  }

  pub fn new_append() -> Self {
    Self {
      read: false,
      write: true,
      create: false,
      truncate: false,
      append: true,
      create_new: false,
      mode: None,
      custom_flags: None,
    }
  }

  #[inline]
  pub fn read(&mut self) -> &mut Self {
    self.read = true;
    self
  }

  #[inline]
  pub fn create(&mut self) -> &mut Self {
    self.create = true;
    self
  }

  #[inline]
  pub fn truncate(&mut self) -> &mut Self {
    self.truncate = true;
    self
  }

  #[inline]
  pub fn append(&mut self) -> &mut Self {
    self.append = true;
    self
  }

  #[inline]
  pub fn create_new(&mut self) -> &mut Self {
    self.create_new = true;
    self
  }

  #[inline]
  pub fn mode(&mut self, mode: u32) -> &mut Self {
    self.mode = Some(mode);
    self
  }

  #[inline]
  pub fn custom_flags(&mut self, flags: CustomFlagsValue) -> &mut Self {
    self.custom_flags = Some(flags);
    self
  }
}

// == FsCanonicalize ==

pub trait BaseFsCanonicalize {
  #[doc(hidden)]
  fn base_fs_canonicalize(&self, path: &Path) -> std::io::Result<PathBuf>;
}

pub trait FsCanonicalize: BaseFsCanonicalize {
  #[inline]
  fn fs_canonicalize(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<PathBuf> {
    self.base_fs_canonicalize(path.as_ref())
  }
}

impl<T: BaseFsCanonicalize> FsCanonicalize for T {}

// == FsCopy ==

pub trait BaseFsCopy {
  #[doc(hidden)]
  fn base_fs_copy(&self, from: &Path, to: &Path) -> std::io::Result<u64>;
}

pub trait FsCopy: BaseFsCopy {
  #[inline]
  fn fs_copy(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<u64> {
    self.base_fs_copy(from.as_ref(), to.as_ref())
  }
}

impl<T: BaseFsCopy> FsCopy for T {}

// == FsCreateDir ==

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "camelCase"))]
#[non_exhaustive] // so we can add properties without breaking people
pub struct CreateDirOptions {
  pub recursive: bool,
  /// Unix only. Ignored on Windows.
  pub mode: Option<u32>,
}

impl CreateDirOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_recursive() -> Self {
    Self {
      recursive: true,
      ..Default::default()
    }
  }

  #[inline]
  pub fn recursive(&mut self) -> &mut Self {
    self.recursive = true;
    self
  }

  #[inline]
  pub fn mode(&mut self, mode: u32) -> &mut Self {
    self.mode = Some(mode);
    self
  }
}

pub trait BaseFsCreateDir {
  #[doc(hidden)]
  fn base_fs_create_dir(
    &self,
    path: &Path,
    options: &CreateDirOptions,
  ) -> std::io::Result<()>;
}

pub trait FsCreateDir: BaseFsCreateDir {
  fn fs_create_dir(
    &self,
    path: impl AsRef<Path>,
    options: &CreateDirOptions,
  ) -> std::io::Result<()> {
    self.base_fs_create_dir(path.as_ref(), options)
  }
}

impl<T: BaseFsCreateDir> FsCreateDir for T {}

// == FsCreateDirAll ==

pub trait FsCreateDirAll: BaseFsCreateDir {
  fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.base_fs_create_dir(
      path.as_ref(),
      &CreateDirOptions {
        recursive: true,
        mode: None,
      },
    )
  }
}

impl<T: BaseFsCreateDir> FsCreateDirAll for T {}

// == FsHardLink ==

pub trait BaseFsHardLink {
  #[doc(hidden)]
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> std::io::Result<()>;
}

pub trait FsHardLink: BaseFsHardLink {
  fn fs_hard_link(
    &self,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.base_fs_hard_link(src.as_ref(), dst.as_ref())
  }
}

impl<T: BaseFsHardLink> FsHardLink for T {}

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

pub trait BaseFsMetadata {
  type Metadata: FsMetadataValue;

  #[doc(hidden)]
  fn base_fs_metadata(&self, path: &Path) -> std::io::Result<Self::Metadata>;

  #[doc(hidden)]
  fn base_fs_symlink_metadata(
    &self,
    path: &Path,
  ) -> std::io::Result<Self::Metadata>;
}

/// These two functions are so cloesly related that it becomes verbose to
/// separate them out into two traits.
pub trait FsMetadata: BaseFsMetadata {
  #[inline]
  fn fs_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::Metadata> {
    self.base_fs_metadata(path.as_ref())
  }

  #[inline]
  fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Self::Metadata> {
    self.base_fs_symlink_metadata(path.as_ref())
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

impl<T: BaseFsMetadata> FsMetadata for T {}

// == FsOpen ==

pub trait FsFile:
  std::io::Read
  + std::io::Write
  + std::io::Seek
  + FsFileSetPermissions
  + FsFileSetLen
{
}

pub trait BaseFsOpen {
  // ideally this wouldn't be constrained, but by not doing
  // this then the type parameters get really out of hand
  type File: FsFile;

  #[doc(hidden)]
  fn base_fs_open(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File>;
}

pub trait FsOpen: BaseFsOpen {
  #[inline]
  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File> {
    self.base_fs_open(path.as_ref(), options)
  }
}

impl<T: BaseFsOpen> FsOpen for T {}

// == FsRead ==

pub trait BaseFsRead {
  #[doc(hidden)]
  fn base_fs_read(&self, path: &Path) -> std::io::Result<Cow<'static, [u8]>>;
}

pub trait FsRead: BaseFsRead {
  #[inline]
  fn fs_read(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, [u8]>> {
    self.base_fs_read(path.as_ref())
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

impl<T: BaseFsRead> FsRead for T {}

// == FsReadDir ==

pub trait FsDirEntry: std::fmt::Debug {
  type Metadata: FsMetadataValue;

  fn file_name(&self) -> Cow<OsStr>;
  fn file_type(&self) -> std::io::Result<FileType>;
  fn metadata(&self) -> std::io::Result<Self::Metadata>;
  fn path(&self) -> Cow<Path>;
}

pub trait BaseFsReadDir {
  type ReadDirEntry: FsDirEntry + 'static;

  #[doc(hidden)]
  fn base_fs_read_dir(
    &self,
    path: &Path,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  >;
}

pub trait FsReadDir: BaseFsReadDir {
  #[inline]
  fn fs_read_dir(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  > {
    self.base_fs_read_dir(path.as_ref())
  }
}

impl<T: BaseFsReadDir> FsReadDir for T {}

// == FsRemoveDirAll ==

pub trait BaseFsRemoveDirAll {
  #[doc(hidden)]
  fn base_fs_remove_dir_all(&self, path: &Path) -> std::io::Result<()>;
}

pub trait FsRemoveDirAll: BaseFsRemoveDirAll {
  #[inline]
  fn fs_remove_dir_all(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.base_fs_remove_dir_all(path.as_ref())
  }
}

impl<T: BaseFsRemoveDirAll> FsRemoveDirAll for T {}

// == FsRemoveFile ==

pub trait BaseFsRemoveFile {
  #[doc(hidden)]
  fn base_fs_remove_file(&self, path: &Path) -> std::io::Result<()>;
}

pub trait FsRemoveFile: BaseFsRemoveFile {
  #[inline]
  fn fs_remove_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    self.base_fs_remove_file(path.as_ref())
  }
}

impl<T: BaseFsRemoveFile> FsRemoveFile for T {}

// == FsRename ==

pub trait BaseFsRename {
  #[doc(hidden)]
  fn base_fs_rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
}

pub trait FsRename: BaseFsRename {
  #[inline]
  fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.base_fs_rename(from.as_ref(), to.as_ref())
  }
}

impl<T: BaseFsRename> FsRename for T {}

// == FsSetPermissions ==

pub trait BaseFsSetPermissions {
  #[doc(hidden)]
  fn base_fs_set_permissions(
    &self,
    path: &Path,
    mode: u32,
  ) -> std::io::Result<()>;
}

pub trait FsSetPermissions: BaseFsSetPermissions {
  fn fs_set_permissions(
    &self,
    path: impl AsRef<Path>,
    mode: u32,
  ) -> std::io::Result<()> {
    self.base_fs_set_permissions(path.as_ref(), mode)
  }
}

impl<T: BaseFsSetPermissions> FsSetPermissions for T {}

// == FsSymlinkDir ==

pub trait BaseFsSymlinkDir {
  #[doc(hidden)]
  fn base_fs_symlink_dir(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()>;
}

pub trait FsSymlinkDir: BaseFsSymlinkDir {
  #[inline]
  fn fs_symlink_dir(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.base_fs_symlink_dir(original.as_ref(), link.as_ref())
  }
}

impl<T: BaseFsSymlinkDir> FsSymlinkDir for T {}

// == FsSymlinkFile ==

pub trait BaseFsSymlinkFile {
  #[doc(hidden)]
  fn base_fs_symlink_file(
    &self,
    original: &Path,
    link: &Path,
  ) -> std::io::Result<()>;
}

pub trait FsSymlinkFile: BaseFsSymlinkFile {
  #[inline]
  fn fs_symlink_file(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.base_fs_symlink_file(original.as_ref(), link.as_ref())
  }
}

impl<T: BaseFsSymlinkFile> FsSymlinkFile for T {}

// == FsWrite ==

pub trait BaseFsWrite {
  #[doc(hidden)]
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()>;
}

pub trait FsWrite: BaseFsWrite {
  #[inline]
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()> {
    self.base_fs_write(path.as_ref(), data.as_ref())
  }
}

impl<T: BaseFsWrite> FsWrite for T {}

// #### FILE SYSTEM FILE ####

pub trait FsFileSetLen {
  fn fs_file_set_len(&mut self, size: u64) -> std::io::Result<()>;
}

pub trait FsFileSetPermissions {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()>;
}

// #### SYSTEM ####

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
