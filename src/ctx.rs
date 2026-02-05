//! Error context wrapper for sys_traits operations.
//!
//! This module provides [`SysCtx`], a wrapper that adds operation and path
//! context to errors returned by sys_traits methods.
//!
//! # Example
//!
//! ```no_run
//! use sys_traits::SysErrorCtx;
//! # #[cfg(feature = "real")]
//! use sys_traits::impls::RealSys;
//!
//! # #[cfg(feature = "real")]
//! # fn example() -> std::io::Result<()> {
//! let sys = RealSys;
//!
//! // Without context:
//! // sys.fs_read("/path/to/file")?;
//! // Error: No such file or directory (os error 2)
//!
//! // With context:
//! sys.with_paths_in_errors().fs_read("/path/to/file")?;
//! // Error: failed to read '/path/to/file': No such file or directory (os error 2)
//! # Ok(())
//! # }
//! ```

use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::BaseFsCanonicalize;
use crate::BaseFsChown;
use crate::BaseFsCloneFile;
use crate::BaseFsCopy;
use crate::BaseFsCreateDir;
use crate::BaseFsCreateJunction;
use crate::BaseFsHardLink;
use crate::BaseFsMetadata;
use crate::BaseFsRead;
use crate::BaseFsReadDir;
use crate::BaseFsReadLink;
use crate::BaseFsRemoveDir;
use crate::BaseFsRemoveDirAll;
use crate::BaseFsRemoveFile;
use crate::BaseFsRename;
use crate::BaseFsSetFileTimes;
use crate::BaseFsSetPermissions;
use crate::BaseFsSetSymlinkFileTimes;
use crate::BaseFsSymlinkChown;
use crate::BaseFsSymlinkDir;
use crate::BaseFsSymlinkFile;
use crate::BaseFsWrite;
use crate::CreateDirOptions;
use crate::FileType;
use crate::FsMetadataValue;
use crate::FsRead;

/// An error that includes context about the operation that failed.
#[derive(Debug)]
pub struct OperationError {
  operation: &'static str,
  kind: OperationErrorKind,
  /// The underlying I/O error.
  pub err: io::Error,
}

impl OperationError {
  /// Returns the operation name (e.g., "read", "write", "copy").
  pub fn operation(&self) -> &'static str {
    self.operation
  }

  /// Returns the error context kind.
  pub fn kind(&self) -> &OperationErrorKind {
    &self.kind
  }
}

impl fmt::Display for OperationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "failed to {}", self.operation)?;
    match &self.kind {
      OperationErrorKind::WithPath(path) => write!(f, " '{}'", path)?,
      OperationErrorKind::WithTwoPaths(from, to) => {
        write!(f, " '{}' to '{}'", from, to)?
      }
    }
    write!(f, ": {}", self.err)
  }
}

impl Error for OperationError {}

/// The kind of context associated with an operation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationErrorKind {
  /// Single path context.
  WithPath(String),
  /// Two path context (e.g., copy, rename).
  WithTwoPaths(String, String),
}

/// A wrapper that adds error context to sys_traits operations.
///
/// Use [`SysErrorCtx::with_ctx`] to create an instance.
#[derive(Debug, Clone, Copy)]
pub struct SysCtx<T>(pub T);

impl<T> SysCtx<T> {
  /// Creates a new `SysCtx` wrapper.
  pub fn new(inner: T) -> Self {
    Self(inner)
  }

  /// Returns a reference to the inner value.
  pub fn inner(&self) -> &T {
    &self.0
  }

  /// Consumes the wrapper and returns the inner value.
  pub fn into_inner(self) -> T {
    self.0
  }
}

/// Extension trait that provides the [`with_ctx`](SysErrorCtx::with_ctx) method.
///
/// Import this trait to use `.with_paths_in_errors()` on any type.
pub trait SysErrorCtx {
  /// Wraps `self` in a [`SysCtx`] that includes paths in error messages.
  fn with_paths_in_errors(&self) -> SysCtx<&Self> {
    SysCtx(self)
  }
}

impl<T> SysErrorCtx for T {}

// helper to create single-path errors wrapped in io::Error
fn err_with_path(
  operation: &'static str,
  path: &Path,
  err: io::Error,
) -> io::Error {
  io::Error::new(
    err.kind(),
    OperationError {
      operation,
      kind: OperationErrorKind::WithPath(path.to_string_lossy().into_owned()),
      err,
    },
  )
}

// helper to create two-path errors wrapped in io::Error
fn err_with_two_paths(
  operation: &'static str,
  from: &Path,
  to: &Path,
  err: io::Error,
) -> io::Error {
  io::Error::new(
    err.kind(),
    OperationError {
      operation,
      kind: OperationErrorKind::WithTwoPaths(
        from.to_string_lossy().into_owned(),
        to.to_string_lossy().into_owned(),
      ),
      err,
    },
  )
}

// == FsCanonicalize ==

impl<T: BaseFsCanonicalize> SysCtx<&T> {
  pub fn fs_canonicalize(&self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();
    self
      .0
      .base_fs_canonicalize(path)
      .map_err(|e| err_with_path("canonicalize", path, e))
  }
}

// == FsChown ==

impl<T: BaseFsChown> SysCtx<&T> {
  pub fn fs_chown(
    &self,
    path: impl AsRef<Path>,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_chown(path, uid, gid)
      .map_err(|e| err_with_path("chown", path, e))
  }
}

// == FsSymlinkChown ==

impl<T: BaseFsSymlinkChown> SysCtx<&T> {
  pub fn fs_symlink_chown(
    &self,
    path: impl AsRef<Path>,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_symlink_chown(path, uid, gid)
      .map_err(|e| err_with_path("chown symlink", path, e))
  }
}

// == FsCloneFile ==

impl<T: BaseFsCloneFile> SysCtx<&T> {
  pub fn fs_clone_file(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    self
      .0
      .base_fs_clone_file(from, to)
      .map_err(|e| err_with_two_paths("clone", from, to, e))
  }
}

// == FsCopy ==

impl<T: BaseFsCopy> SysCtx<&T> {
  pub fn fs_copy(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> io::Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();
    self
      .0
      .base_fs_copy(from, to)
      .map_err(|e| err_with_two_paths("copy", from, to, e))
  }
}

// == FsCreateDir ==

impl<T: BaseFsCreateDir> SysCtx<&T> {
  pub fn fs_create_dir(
    &self,
    path: impl AsRef<Path>,
    options: &CreateDirOptions,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_create_dir(path, options)
      .map_err(|e| err_with_path("create directory", path, e))
  }

  pub fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_create_dir(
        path,
        &CreateDirOptions {
          recursive: true,
          mode: None,
        },
      )
      .map_err(|e| err_with_path("create directory", path, e))
  }
}

// == FsHardLink ==

impl<T: BaseFsHardLink> SysCtx<&T> {
  pub fn fs_hard_link(
    &self,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
  ) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    self
      .0
      .base_fs_hard_link(src, dst)
      .map_err(|e| err_with_two_paths("hard link", src, dst, e))
  }
}

// == FsCreateJunction ==

impl<T: BaseFsCreateJunction> SysCtx<&T> {
  pub fn fs_create_junction(
    &self,
    original: impl AsRef<Path>,
    junction: impl AsRef<Path>,
  ) -> io::Result<()> {
    let original = original.as_ref();
    let junction = junction.as_ref();
    self
      .0
      .base_fs_create_junction(original, junction)
      .map_err(|e| err_with_two_paths("create junction", original, junction, e))
  }
}

// == FsMetadata ==

impl<T: BaseFsMetadata> SysCtx<&T> {
  pub fn fs_metadata(&self, path: impl AsRef<Path>) -> io::Result<T::Metadata> {
    let path = path.as_ref();
    self
      .0
      .base_fs_metadata(path)
      .map_err(|e| err_with_path("stat", path, e))
  }

  pub fn fs_symlink_metadata(
    &self,
    path: impl AsRef<Path>,
  ) -> io::Result<T::Metadata> {
    let path = path.as_ref();
    self
      .0
      .base_fs_symlink_metadata(path)
      .map_err(|e| err_with_path("lstat", path, e))
  }

  pub fn fs_is_file(&self, path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(self.fs_metadata(path)?.file_type() == FileType::File)
  }

  pub fn fs_is_dir(&self, path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(self.fs_metadata(path)?.file_type() == FileType::Dir)
  }

  pub fn fs_is_symlink(&self, path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(self.fs_symlink_metadata(path)?.file_type() == FileType::Symlink)
  }
}

// == FsRead ==

impl<T: BaseFsRead> SysCtx<&T> {
  pub fn fs_read(
    &self,
    path: impl AsRef<Path>,
  ) -> io::Result<Cow<'static, [u8]>> {
    let path = path.as_ref();
    self
      .0
      .base_fs_read(path)
      .map_err(|e| err_with_path("read", path, e))
  }

  pub fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> io::Result<Cow<'static, str>> {
    let path = path.as_ref();
    self
      .0
      .fs_read_to_string(path)
      .map_err(|e| err_with_path("read", path, e))
  }

  pub fn fs_read_to_string_lossy(
    &self,
    path: impl AsRef<Path>,
  ) -> io::Result<Cow<'static, str>> {
    let path = path.as_ref();
    self
      .0
      .fs_read_to_string_lossy(path)
      .map_err(|e| err_with_path("read", path, e))
  }
}

// == FsReadDir ==

impl<T: BaseFsReadDir> SysCtx<&T> {
  pub fn fs_read_dir(
    &self,
    path: impl AsRef<Path>,
  ) -> io::Result<Box<dyn Iterator<Item = io::Result<T::ReadDirEntry>>>> {
    let path = path.as_ref();
    self
      .0
      .base_fs_read_dir(path)
      .map_err(|e| err_with_path("read directory", path, e))
  }
}

// == FsReadLink ==

impl<T: BaseFsReadLink> SysCtx<&T> {
  pub fn fs_read_link(&self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();
    self
      .0
      .base_fs_read_link(path)
      .map_err(|e| err_with_path("read link", path, e))
  }
}

// == FsRemoveDir ==

impl<T: BaseFsRemoveDir> SysCtx<&T> {
  pub fn fs_remove_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_remove_dir(path)
      .map_err(|e| err_with_path("remove directory", path, e))
  }
}

// == FsRemoveDirAll ==

impl<T: BaseFsRemoveDirAll> SysCtx<&T> {
  pub fn fs_remove_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_remove_dir_all(path)
      .map_err(|e| err_with_path("remove directory", path, e))
  }
}

// == FsRemoveFile ==

impl<T: BaseFsRemoveFile> SysCtx<&T> {
  pub fn fs_remove_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_remove_file(path)
      .map_err(|e| err_with_path("remove", path, e))
  }
}

// == FsRename ==

impl<T: BaseFsRename> SysCtx<&T> {
  pub fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    self
      .0
      .base_fs_rename(from, to)
      .map_err(|e| err_with_two_paths("rename", from, to, e))
  }
}

// == FsSetFileTimes ==

impl<T: BaseFsSetFileTimes> SysCtx<&T> {
  pub fn fs_set_file_times(
    &self,
    path: impl AsRef<Path>,
    atime: SystemTime,
    mtime: SystemTime,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_set_file_times(path, atime, mtime)
      .map_err(|e| err_with_path("set file times", path, e))
  }
}

// == FsSetSymlinkFileTimes ==

impl<T: BaseFsSetSymlinkFileTimes> SysCtx<&T> {
  pub fn fs_set_symlink_file_times(
    &self,
    path: impl AsRef<Path>,
    atime: SystemTime,
    mtime: SystemTime,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_set_symlink_file_times(path, atime, mtime)
      .map_err(|e| err_with_path("set symlink file times", path, e))
  }
}

// == FsSetPermissions ==

impl<T: BaseFsSetPermissions> SysCtx<&T> {
  pub fn fs_set_permissions(
    &self,
    path: impl AsRef<Path>,
    mode: u32,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_set_permissions(path, mode)
      .map_err(|e| err_with_path("set permissions", path, e))
  }
}

// == FsSymlinkDir ==

impl<T: BaseFsSymlinkDir> SysCtx<&T> {
  pub fn fs_symlink_dir(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> io::Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();
    self
      .0
      .base_fs_symlink_dir(original, link)
      .map_err(|e| err_with_two_paths("symlink directory", original, link, e))
  }
}

// == FsSymlinkFile ==

impl<T: BaseFsSymlinkFile> SysCtx<&T> {
  pub fn fs_symlink_file(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> io::Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();
    self
      .0
      .base_fs_symlink_file(original, link)
      .map_err(|e| err_with_two_paths("symlink", original, link, e))
  }
}

// == FsWrite ==

impl<T: BaseFsWrite> SysCtx<&T> {
  pub fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> io::Result<()> {
    let path = path.as_ref();
    self
      .0
      .base_fs_write(path, data.as_ref())
      .map_err(|e| err_with_path("write", path, e))
  }
}

#[cfg(all(test, feature = "memory"))]
mod tests {
  use super::*;
  use crate::impls::InMemorySys;
  use crate::FsCreateDir;
  use crate::FsMetadata;
  use crate::FsRead;
  use crate::FsWrite;

  #[test]
  fn test_error_display_single_path() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_read("/nonexistent")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "read");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithPath("/nonexistent".to_string())
    );
    assert_eq!(
      err.to_string(),
      format!("failed to read '/nonexistent': {}", op_err.err)
    );
  }

  #[test]
  fn test_error_display_two_paths() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_copy("/src", "/dst")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "copy");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithTwoPaths("/src".to_string(), "/dst".to_string())
    );
    assert_eq!(
      err.to_string(),
      format!("failed to copy '/src' to '/dst': {}", op_err.err)
    );
  }

  #[test]
  fn test_error_preserves_kind() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_read("/nonexistent")
      .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
  }

  #[test]
  fn test_error_downcast_to_operation_error() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_read("/nonexistent")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "read");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithPath("/nonexistent".to_string())
    );
  }

  #[test]
  fn test_fs_read_success() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/test.txt", b"hello").unwrap();
    let data = sys.with_paths_in_errors().fs_read("/test.txt").unwrap();
    assert_eq!(&*data, b"hello");
  }

  #[test]
  fn test_fs_read_to_string_success() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/test.txt", b"hello").unwrap();
    let data = sys
      .with_paths_in_errors()
      .fs_read_to_string("/test.txt")
      .unwrap();
    assert_eq!(&*data, "hello");
  }

  #[test]
  fn test_fs_read_to_string_lossy_success() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/test.txt", b"hello").unwrap();
    let data = sys
      .with_paths_in_errors()
      .fs_read_to_string_lossy("/test.txt")
      .unwrap();
    assert_eq!(&*data, "hello");
  }

  #[test]
  fn test_fs_write_success() {
    let sys = InMemorySys::new_with_cwd("/");
    sys
      .with_paths_in_errors()
      .fs_write("/test.txt", b"hello")
      .unwrap();
    let data = sys.fs_read("/test.txt").unwrap();
    assert_eq!(&*data, b"hello");
  }

  #[test]
  fn test_fs_write_error() {
    let sys = InMemorySys::default();
    // writing to a path in a non-existent directory should fail
    let err = sys
      .with_paths_in_errors()
      .fs_write("/nonexistent/test.txt", b"hello")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "write");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithPath("/nonexistent/test.txt".to_string())
    );
  }

  #[test]
  fn test_fs_create_dir() {
    let sys = InMemorySys::default();
    sys
      .with_paths_in_errors()
      .fs_create_dir("/newdir", &CreateDirOptions::default())
      .unwrap();
    assert!(sys.fs_is_dir("/newdir").unwrap());
  }

  #[test]
  fn test_fs_create_dir_all() {
    let sys = InMemorySys::default();
    sys
      .with_paths_in_errors()
      .fs_create_dir_all("/a/b/c")
      .unwrap();
    assert!(sys.fs_is_dir("/a/b/c").unwrap());
  }

  #[test]
  fn test_fs_remove_file() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/test.txt", b"hello").unwrap();
    sys
      .with_paths_in_errors()
      .fs_remove_file("/test.txt")
      .unwrap();
    assert!(!sys.fs_exists("/test.txt").unwrap());
  }

  #[test]
  fn test_fs_remove_file_error() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_remove_file("/nonexistent")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "remove");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithPath("/nonexistent".to_string())
    );
  }

  #[test]
  fn test_fs_remove_dir() {
    let sys = InMemorySys::default();
    sys
      .fs_create_dir("/testdir", &CreateDirOptions::default())
      .unwrap();
    sys
      .with_paths_in_errors()
      .fs_remove_dir("/testdir")
      .unwrap();
    assert!(!sys.fs_exists("/testdir").unwrap());
  }

  #[test]
  fn test_fs_rename() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/old.txt", b"hello").unwrap();
    sys
      .with_paths_in_errors()
      .fs_rename("/old.txt", "/new.txt")
      .unwrap();
    assert!(!sys.fs_exists("/old.txt").unwrap());
    assert!(sys.fs_exists("/new.txt").unwrap());
  }

  #[test]
  fn test_fs_rename_error() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_rename("/nonexistent", "/new.txt")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "rename");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithTwoPaths(
        "/nonexistent".to_string(),
        "/new.txt".to_string()
      )
    );
  }

  #[test]
  fn test_fs_copy() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/src.txt", b"hello").unwrap();
    let bytes = sys
      .with_paths_in_errors()
      .fs_copy("/src.txt", "/dst.txt")
      .unwrap();
    assert_eq!(bytes, 5);
    assert_eq!(&*sys.fs_read("/dst.txt").unwrap(), b"hello");
  }

  #[test]
  fn test_fs_metadata() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/test.txt", b"hello").unwrap();
    let meta = sys.with_paths_in_errors().fs_metadata("/test.txt").unwrap();
    assert_eq!(meta.file_type(), FileType::File);
    assert_eq!(meta.len(), 5);
  }

  #[test]
  fn test_fs_metadata_error() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_metadata("/nonexistent")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "stat");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithPath("/nonexistent".to_string())
    );
  }

  #[test]
  fn test_fs_is_file() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/test.txt", b"hello").unwrap();
    assert!(sys.with_paths_in_errors().fs_is_file("/test.txt").unwrap());
    sys
      .fs_create_dir("/testdir", &CreateDirOptions::default())
      .unwrap();
    assert!(!sys.with_paths_in_errors().fs_is_file("/testdir").unwrap());
  }

  #[test]
  fn test_fs_is_dir() {
    let sys = InMemorySys::new_with_cwd("/");
    sys
      .fs_create_dir("/testdir", &CreateDirOptions::default())
      .unwrap();
    assert!(sys.with_paths_in_errors().fs_is_dir("/testdir").unwrap());
    sys.fs_write("/test.txt", b"hello").unwrap();
    assert!(!sys.with_paths_in_errors().fs_is_dir("/test.txt").unwrap());
  }

  #[test]
  fn test_fs_read_dir() {
    let sys = InMemorySys::new_with_cwd("/");
    sys
      .fs_create_dir("/testdir", &CreateDirOptions::default())
      .unwrap();
    sys.fs_write("/testdir/a.txt", b"a").unwrap();
    sys.fs_write("/testdir/b.txt", b"b").unwrap();
    let entries: Vec<_> = sys
      .with_paths_in_errors()
      .fs_read_dir("/testdir")
      .unwrap()
      .collect::<Result<_, _>>()
      .unwrap();
    assert_eq!(entries.len(), 2);
  }

  #[test]
  fn test_fs_read_dir_error() {
    let sys = InMemorySys::default();
    let result = sys.with_paths_in_errors().fs_read_dir("/nonexistent");
    let err = match result {
      Ok(_) => panic!("expected error"),
      Err(e) => e,
    };
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "read directory");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithPath("/nonexistent".to_string())
    );
  }

  #[test]
  fn test_fs_hard_link() {
    let sys = InMemorySys::new_with_cwd("/");
    sys.fs_write("/original.txt", b"hello").unwrap();
    sys
      .with_paths_in_errors()
      .fs_hard_link("/original.txt", "/link.txt")
      .unwrap();
    assert_eq!(&*sys.fs_read("/link.txt").unwrap(), b"hello");
  }

  #[test]
  fn test_fs_hard_link_error() {
    let sys = InMemorySys::default();
    let err = sys
      .with_paths_in_errors()
      .fs_hard_link("/nonexistent", "/link.txt")
      .unwrap_err();
    let inner = err.get_ref().unwrap();
    let op_err = inner.downcast_ref::<OperationError>().unwrap();
    assert_eq!(op_err.operation(), "hard link");
    assert_eq!(
      op_err.kind(),
      &OperationErrorKind::WithTwoPaths(
        "/nonexistent".to_string(),
        "/link.txt".to_string()
      )
    );
  }
}
