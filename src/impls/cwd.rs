use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use std::time::SystemTime;

use crate::*;

/// A system wrapper with an instance-scoped current working directory.
///
/// Relative filesystem paths are resolved against the logical current working
/// directory before being delegated to the inner system. Changing the current
/// directory updates only this wrapper and never calls the inner system's
/// [`BaseEnvSetCurrentDir`] implementation.
///
/// Clones share current working directory changes, while separately
/// constructed wrappers have independent state.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "real")]
/// # fn main() -> std::io::Result<()> {
/// use sys_traits::impls::{CwdSys, RealSys};
/// use sys_traits::{EnvSetCurrentDir, FsRead};
///
/// let sys = CwdSys::new(RealSys)?;
/// sys.env_set_current_dir("/project")?;
/// let config = sys.fs_read("deno.json")?;
/// # let _ = config;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "real"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone)]
pub struct CwdSys<T> {
  inner: T,
  cwd: Arc<RwLock<PathBuf>>,
}

impl<T: EnvCurrentDir> CwdSys<T> {
  /// Creates a wrapper whose logical current working directory starts at the
  /// current directory reported by the inner system.
  pub fn new(inner: T) -> io::Result<Self> {
    let cwd = inner.env_current_dir()?;
    Self::new_with_cwd(inner, cwd)
  }
}

impl<T> CwdSys<T> {
  /// Creates a wrapper whose logical current working directory starts at the
  /// provided path, which should be absolute.
  ///
  /// Unlike changing the current directory, this does not canonicalize the
  /// path or check that it's a directory in the inner system.
  pub fn new_with_cwd(inner: T, cwd: impl Into<PathBuf>) -> io::Result<Self> {
    let cwd = cwd.into();
    if cwd.as_os_str().is_empty() {
      return Err(io::Error::new(
        io::ErrorKind::NotFound,
        "working directory path is empty",
      ));
    }
    Ok(Self {
      inner,
      cwd: Arc::new(RwLock::new(cwd)),
    })
  }

  /// Returns a reference to the inner system.
  pub fn inner(&self) -> &T {
    &self.inner
  }

  /// Consumes the wrapper and returns the inner system.
  pub fn into_inner(self) -> T {
    self.inner
  }

  fn cwd(&self) -> PathBuf {
    self
      .cwd
      .read()
      .unwrap_or_else(|err| err.into_inner())
      .clone()
  }

  fn set_cwd(&self, cwd: PathBuf) {
    *self.cwd.write().unwrap_or_else(|err| err.into_inner()) = cwd;
  }

  fn resolve_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
    Self::resolve_path_from(&self.cwd(), path)
  }

  fn resolve_paths<'a, 'b>(
    &self,
    first: &'a Path,
    second: &'b Path,
  ) -> (Cow<'a, Path>, Cow<'b, Path>) {
    let cwd = self.cwd();
    (
      Self::resolve_path_from(&cwd, first),
      Self::resolve_path_from(&cwd, second),
    )
  }

  fn resolve_path_from<'a>(cwd: &Path, path: &'a Path) -> Cow<'a, Path> {
    if path.is_absolute() || path.as_os_str().is_empty() {
      Cow::Borrowed(path)
    } else {
      Cow::Owned(cwd.join(path))
    }
  }
}

// ==== Environment ====

impl<T> EnvCurrentDir for CwdSys<T> {
  fn env_current_dir(&self) -> io::Result<PathBuf> {
    Ok(self.cwd())
  }
}

impl<T> BaseEnvSetCurrentDir for CwdSys<T>
where
  T: BaseFsCanonicalize + BaseFsMetadata,
{
  fn base_env_set_current_dir(&self, path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty() {
      return Err(io::Error::new(
        io::ErrorKind::NotFound,
        "working directory path is empty",
      ));
    }

    let path = self.resolve_path(path);
    let path = self.inner.base_fs_canonicalize(&path)?;
    let metadata = self.inner.base_fs_metadata(&path)?;
    if !metadata.file_type().is_dir() {
      return Err(io::Error::new(
        io::ErrorKind::NotADirectory,
        format!("'{}' is not a directory", path.display()),
      ));
    }

    self.set_cwd(path);
    Ok(())
  }
}

impl<T: BaseEnvVar> BaseEnvVar for CwdSys<T> {
  fn base_env_var_os(&self, key: &OsStr) -> Option<OsString> {
    self.inner.base_env_var_os(key)
  }
}

impl<T: EnvVars> EnvVars for CwdSys<T> {
  type EnvVarsOs = T::EnvVarsOs;

  fn env_vars_os(&self) -> Self::EnvVarsOs {
    self.inner.env_vars_os()
  }
}

impl<T: BaseEnvRemoveVar> BaseEnvRemoveVar for CwdSys<T> {
  fn base_env_remove_var(&self, key: &OsStr) {
    self.inner.base_env_remove_var(key);
  }
}

impl<T: BaseEnvSetVar> BaseEnvSetVar for CwdSys<T> {
  fn base_env_set_var(&self, key: &OsStr, value: &OsStr) {
    self.inner.base_env_set_var(key, value);
  }
}

impl<T: EnvUmask> EnvUmask for CwdSys<T> {
  fn env_umask(&self) -> io::Result<u32> {
    self.inner.env_umask()
  }
}

impl<T: EnvSetUmask> EnvSetUmask for CwdSys<T> {
  fn env_set_umask(&self, umask: u32) -> io::Result<u32> {
    self.inner.env_set_umask(umask)
  }
}

impl<T: EnvCacheDir> EnvCacheDir for CwdSys<T> {
  fn env_cache_dir(&self) -> Option<PathBuf> {
    self.inner.env_cache_dir()
  }
}

impl<T: EnvHomeDir> EnvHomeDir for CwdSys<T> {
  fn env_home_dir(&self) -> Option<PathBuf> {
    self.inner.env_home_dir()
  }
}

impl<T: EnvProgramsDir> EnvProgramsDir for CwdSys<T> {
  fn env_programs_dir(&self) -> Option<PathBuf> {
    self.inner.env_programs_dir()
  }
}

impl<T: EnvTempDir> EnvTempDir for CwdSys<T> {
  fn env_temp_dir(&self) -> io::Result<PathBuf> {
    self.inner.env_temp_dir()
  }
}

// ==== File System ====

impl<T: BaseFsCanonicalize> BaseFsCanonicalize for CwdSys<T> {
  fn base_fs_canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
    self.inner.base_fs_canonicalize(&self.resolve_path(path))
  }
}

impl<T: BaseFsChown> BaseFsChown for CwdSys<T> {
  fn base_fs_chown(
    &self,
    path: &Path,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> io::Result<()> {
    self.inner.base_fs_chown(&self.resolve_path(path), uid, gid)
  }
}

impl<T: BaseFsSymlinkChown> BaseFsSymlinkChown for CwdSys<T> {
  fn base_fs_symlink_chown(
    &self,
    path: &Path,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> io::Result<()> {
    self
      .inner
      .base_fs_symlink_chown(&self.resolve_path(path), uid, gid)
  }
}

impl<T: BaseFsCloneFile> BaseFsCloneFile for CwdSys<T> {
  fn base_fs_clone_file(&self, from: &Path, to: &Path) -> io::Result<()> {
    let (from, to) = self.resolve_paths(from, to);
    self.inner.base_fs_clone_file(&from, &to)
  }
}

impl<T: BaseFsCopy> BaseFsCopy for CwdSys<T> {
  fn base_fs_copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
    let (from, to) = self.resolve_paths(from, to);
    self.inner.base_fs_copy(&from, &to)
  }
}

impl<T: BaseFsCreateDir> BaseFsCreateDir for CwdSys<T> {
  fn base_fs_create_dir(
    &self,
    path: &Path,
    options: &CreateDirOptions,
  ) -> io::Result<()> {
    self
      .inner
      .base_fs_create_dir(&self.resolve_path(path), options)
  }
}

impl<T: BaseFsHardLink> BaseFsHardLink for CwdSys<T> {
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> io::Result<()> {
    let (src, dst) = self.resolve_paths(src, dst);
    self.inner.base_fs_hard_link(&src, &dst)
  }
}

impl<T: BaseFsCreateJunction> BaseFsCreateJunction for CwdSys<T> {
  fn base_fs_create_junction(
    &self,
    original: &Path,
    junction: &Path,
  ) -> io::Result<()> {
    let (original, junction) = self.resolve_paths(original, junction);
    self.inner.base_fs_create_junction(&original, &junction)
  }
}

impl<T: BaseFsMetadata> BaseFsMetadata for CwdSys<T> {
  type Metadata = T::Metadata;

  fn base_fs_metadata(&self, path: &Path) -> io::Result<Self::Metadata> {
    self.inner.base_fs_metadata(&self.resolve_path(path))
  }

  fn base_fs_symlink_metadata(
    &self,
    path: &Path,
  ) -> io::Result<Self::Metadata> {
    self
      .inner
      .base_fs_symlink_metadata(&self.resolve_path(path))
  }

  fn base_fs_exists(&self, path: &Path) -> io::Result<bool> {
    self.inner.base_fs_exists(&self.resolve_path(path))
  }

  fn base_fs_exists_no_err(&self, path: &Path) -> bool {
    self.inner.base_fs_exists_no_err(&self.resolve_path(path))
  }
}

impl<T: BaseFsOpen> BaseFsOpen for CwdSys<T> {
  type File = T::File;

  fn base_fs_open(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> io::Result<Self::File> {
    self.inner.base_fs_open(&self.resolve_path(path), options)
  }
}

impl<T: BaseFsRead> BaseFsRead for CwdSys<T> {
  fn base_fs_read(&self, path: &Path) -> io::Result<Cow<'static, [u8]>> {
    self.inner.base_fs_read(&self.resolve_path(path))
  }
}

impl<T: BaseFsReadDir> BaseFsReadDir for CwdSys<T> {
  type ReadDirEntry = T::ReadDirEntry;

  fn base_fs_read_dir(
    &self,
    path: &Path,
  ) -> io::Result<Box<dyn Iterator<Item = io::Result<Self::ReadDirEntry>>>> {
    self.inner.base_fs_read_dir(&self.resolve_path(path))
  }
}

impl<T: BaseFsReadLink> BaseFsReadLink for CwdSys<T> {
  fn base_fs_read_link(&self, path: &Path) -> io::Result<PathBuf> {
    self.inner.base_fs_read_link(&self.resolve_path(path))
  }
}

impl<T: BaseFsRemoveDir> BaseFsRemoveDir for CwdSys<T> {
  fn base_fs_remove_dir(&self, path: &Path) -> io::Result<()> {
    self.inner.base_fs_remove_dir(&self.resolve_path(path))
  }
}

impl<T: BaseFsRemoveDirAll> BaseFsRemoveDirAll for CwdSys<T> {
  fn base_fs_remove_dir_all(&self, path: &Path) -> io::Result<()> {
    self.inner.base_fs_remove_dir_all(&self.resolve_path(path))
  }
}

impl<T: BaseFsRemoveFile> BaseFsRemoveFile for CwdSys<T> {
  fn base_fs_remove_file(&self, path: &Path) -> io::Result<()> {
    self.inner.base_fs_remove_file(&self.resolve_path(path))
  }
}

impl<T: BaseFsRename> BaseFsRename for CwdSys<T> {
  fn base_fs_rename(&self, from: &Path, to: &Path) -> io::Result<()> {
    let (from, to) = self.resolve_paths(from, to);
    self.inner.base_fs_rename(&from, &to)
  }
}

impl<T: BaseFsSetFileTimes> BaseFsSetFileTimes for CwdSys<T> {
  fn base_fs_set_file_times(
    &self,
    path: &Path,
    atime: SystemTime,
    mtime: SystemTime,
  ) -> io::Result<()> {
    self
      .inner
      .base_fs_set_file_times(&self.resolve_path(path), atime, mtime)
  }
}

impl<T: BaseFsSetSymlinkFileTimes> BaseFsSetSymlinkFileTimes for CwdSys<T> {
  fn base_fs_set_symlink_file_times(
    &self,
    path: &Path,
    atime: SystemTime,
    mtime: SystemTime,
  ) -> io::Result<()> {
    self.inner.base_fs_set_symlink_file_times(
      &self.resolve_path(path),
      atime,
      mtime,
    )
  }
}

impl<T: BaseFsSetPermissions> BaseFsSetPermissions for CwdSys<T> {
  fn base_fs_set_permissions(&self, path: &Path, mode: u32) -> io::Result<()> {
    self
      .inner
      .base_fs_set_permissions(&self.resolve_path(path), mode)
  }
}

impl<T: BaseFsSymlinkDir> BaseFsSymlinkDir for CwdSys<T> {
  fn base_fs_symlink_dir(
    &self,
    original: &Path,
    link: &Path,
  ) -> io::Result<()> {
    self
      .inner
      .base_fs_symlink_dir(original, &self.resolve_path(link))
  }
}

impl<T: BaseFsSymlinkFile> BaseFsSymlinkFile for CwdSys<T> {
  fn base_fs_symlink_file(
    &self,
    original: &Path,
    link: &Path,
  ) -> io::Result<()> {
    self
      .inner
      .base_fs_symlink_file(original, &self.resolve_path(link))
  }
}

impl<T: BaseFsWrite> BaseFsWrite for CwdSys<T> {
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
    self.inner.base_fs_write(&self.resolve_path(path), data)
  }
}

// ==== System ====

impl<T: SystemTimeNow> SystemTimeNow for CwdSys<T> {
  fn sys_time_now(&self) -> SystemTime {
    self.inner.sys_time_now()
  }
}

impl<T: SystemRandom> SystemRandom for CwdSys<T> {
  fn sys_random(&self, buf: &mut [u8]) -> io::Result<()> {
    self.inner.sys_random(buf)
  }
}

impl<T: ProcessExit> ProcessExit for CwdSys<T> {
  fn process_exit(&self, code: i32) -> ! {
    self.inner.process_exit(code)
  }
}

impl<T: ThreadSleep> ThreadSleep for CwdSys<T> {
  fn thread_sleep(&self, duration: Duration) {
    self.inner.thread_sleep(duration);
  }
}

#[cfg(all(test, feature = "memory"))]
mod memory_tests {
  use super::*;
  use crate::impls::InMemorySys;

  #[derive(Debug, Clone)]
  struct RecordingSymlinkSys {
    recorded: Arc<RwLock<Option<(PathBuf, PathBuf)>>>,
  }

  fn recording_cwd() -> PathBuf {
    if cfg!(windows) {
      PathBuf::from(r"C:\work")
    } else {
      PathBuf::from("/work")
    }
  }

  impl EnvCurrentDir for RecordingSymlinkSys {
    fn env_current_dir(&self) -> io::Result<PathBuf> {
      Ok(recording_cwd())
    }
  }

  impl BaseFsSymlinkFile for RecordingSymlinkSys {
    fn base_fs_symlink_file(
      &self,
      original: &Path,
      link: &Path,
    ) -> io::Result<()> {
      *self.recorded.write().unwrap_or_else(|err| err.into_inner()) =
        Some((original.to_path_buf(), link.to_path_buf()));
      Ok(())
    }
  }

  #[test]
  fn validates_cwd_with_inner_system() {
    let inner = InMemorySys::new_with_cwd("/");
    inner.fs_create_dir_all("/logical/sub").unwrap();
    inner.fs_write("/logical/file.txt", "data").unwrap();
    let sys = CwdSys::new(inner).unwrap();

    sys.env_set_current_dir("/logical").unwrap();
    assert_eq!(sys.env_current_dir().unwrap(), PathBuf::from("/logical"));

    sys.env_set_current_dir("sub").unwrap();
    assert_eq!(
      sys.env_current_dir().unwrap(),
      PathBuf::from("/logical/sub")
    );

    sys.env_set_current_dir("..").unwrap();
    let cwd = sys.env_current_dir().unwrap();
    let err = sys.env_set_current_dir("file.txt").unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotADirectory);
    assert_eq!(sys.env_current_dir().unwrap(), cwd);

    let err = sys.env_set_current_dir("missing").unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    assert_eq!(sys.env_current_dir().unwrap(), cwd);
  }

  #[test]
  fn new_with_cwd_sets_initial_cwd_without_touching_inner_system() {
    let inner = InMemorySys::new_with_cwd("/");
    inner.fs_create_dir_all("/initial").unwrap();
    inner.fs_write("/initial/file.txt", "data").unwrap();

    let sys = CwdSys::new_with_cwd(inner.clone(), "/initial").unwrap();
    assert_eq!(sys.env_current_dir().unwrap(), PathBuf::from("/initial"));
    assert_eq!(sys.fs_read_to_string("file.txt").unwrap(), "data");
    assert_eq!(inner.env_current_dir().unwrap(), PathBuf::from("/"));

    // not validated against the inner system
    let sys = CwdSys::new_with_cwd(inner.clone(), "/missing").unwrap();
    assert_eq!(sys.env_current_dir().unwrap(), PathBuf::from("/missing"));
    assert_eq!(
      sys.fs_read("file.txt").unwrap_err().kind(),
      io::ErrorKind::NotFound
    );

    let err = CwdSys::new_with_cwd(inner, "").unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
  }

  #[test]
  fn clones_share_cwd_and_independent_wrappers_are_isolated() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CwdSys<InMemorySys>>();

    let inner = InMemorySys::new_with_cwd("/");
    inner.fs_create_dir_all("/one").unwrap();
    inner.fs_create_dir_all("/two").unwrap();
    inner.fs_write("/one/file.txt", "one").unwrap();
    inner.fs_write("/two/file.txt", "two").unwrap();

    let sys = CwdSys::new(inner.clone()).unwrap();
    let cloned = sys.clone();
    std::thread::spawn(move || cloned.env_set_current_dir("/one").unwrap())
      .join()
      .unwrap();
    assert_eq!(sys.env_current_dir().unwrap(), PathBuf::from("/one"));
    assert_eq!(sys.fs_read_to_string("file.txt").unwrap(), "one");

    let independent = CwdSys::new(inner).unwrap();
    independent.env_set_current_dir("/two").unwrap();
    assert_eq!(
      independent.env_current_dir().unwrap(),
      PathBuf::from("/two")
    );
    assert_eq!(independent.fs_read_to_string("file.txt").unwrap(), "two");
    assert_eq!(sys.env_current_dir().unwrap(), PathBuf::from("/one"));
    assert_eq!(sys.fs_read_to_string("file.txt").unwrap(), "one");
  }

  #[test]
  fn preserves_relative_symlink_targets() {
    let recorded = Arc::new(RwLock::new(None));
    let inner = RecordingSymlinkSys {
      recorded: recorded.clone(),
    };
    let sys = CwdSys::new(inner.clone()).unwrap();

    sys.fs_symlink_file("target.txt", "link.txt").unwrap();

    assert_eq!(
      recorded
        .read()
        .unwrap_or_else(|err| err.into_inner())
        .clone(),
      Some((
        PathBuf::from("target.txt"),
        recording_cwd().join("link.txt"),
      ))
    );
  }

  #[test]
  fn empty_and_absolute_paths_preserve_their_semantics() {
    let inner = InMemorySys::new_with_cwd("/");
    inner.fs_create_dir_all("/cwd").unwrap();
    inner.fs_create_dir_all("/absolute").unwrap();
    inner.fs_write("/absolute/file.txt", "absolute").unwrap();
    let sys = CwdSys::new(inner).unwrap();
    sys.env_set_current_dir("/cwd").unwrap();

    assert_eq!(
      sys.fs_read_to_string("/absolute/file.txt").unwrap(),
      "absolute"
    );
    assert_eq!(
      sys.fs_canonicalize("/absolute/file.txt").unwrap(),
      PathBuf::from("/absolute/file.txt")
    );

    let cwd = sys.env_current_dir().unwrap();
    assert_eq!(
      sys.env_set_current_dir("").unwrap_err().kind(),
      io::ErrorKind::NotFound
    );
    assert_eq!(
      sys.fs_canonicalize("").unwrap_err().kind(),
      io::ErrorKind::NotFound
    );
    assert_eq!(sys.env_current_dir().unwrap(), cwd);
    assert!(sys.fs_is_dir("/cwd").unwrap());
  }

  #[test]
  fn delegates_unrelated_traits() {
    let inner = InMemorySys::new_with_cwd("/");
    inner.env_set_var("KEY", "before");
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(123);
    inner.set_time(Some(time));
    let sys = CwdSys::new(inner.clone()).unwrap();

    assert_eq!(sys.env_var("KEY").unwrap(), "before");
    sys.env_set_var("KEY", "after");
    assert_eq!(inner.env_var("KEY").unwrap(), "after");
    assert_eq!(sys.sys_time_now(), time);
  }
}

#[cfg(all(test, feature = "real", not(target_arch = "wasm32")))]
mod real_tests {
  use super::*;
  use crate::impls::RealSys;
  use std::io::Write;

  #[test]
  fn relative_file_operations_use_logical_cwd_without_changing_process_cwd() {
    let process_cwd = std::env::current_dir().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    RealSys.fs_create_dir_all(&workspace).unwrap();
    let workspace = RealSys.fs_canonicalize(&workspace).unwrap();

    let sys = CwdSys::new(RealSys).unwrap();
    sys.env_set_current_dir(&workspace).unwrap();
    assert_eq!(sys.env_current_dir().unwrap(), workspace);
    assert_eq!(std::env::current_dir().unwrap(), process_cwd);

    sys.fs_create_dir_all("sub").unwrap();
    sys.env_set_current_dir("sub").unwrap();
    assert_eq!(
      sys.env_current_dir().unwrap(),
      RealSys.fs_canonicalize(workspace.join("sub")).unwrap()
    );
    assert_eq!(std::env::current_dir().unwrap(), process_cwd);
    sys.env_set_current_dir("..").unwrap();

    let mut file = sys
      .fs_open("opened.txt", &OpenOptions::new_write())
      .unwrap();
    file.write_all(b"opened").unwrap();
    drop(file);
    sys.fs_write("written.txt", "written").unwrap();
    assert_eq!(sys.fs_read_to_string("opened.txt").unwrap(), "opened");
    assert_eq!(sys.fs_read_to_string("written.txt").unwrap(), "written");

    let metadata = sys.fs_metadata("opened.txt").unwrap();
    assert!(metadata.file_type().is_file());
    assert_eq!(metadata.len(), 6);

    let mut names = sys
      .fs_read_dir(".")
      .unwrap()
      .map(|entry| entry.unwrap().file_name().into_owned())
      .collect::<Vec<_>>();
    names.sort();
    assert!(names.contains(&OsString::from("opened.txt")));
    assert!(names.contains(&OsString::from("sub")));
    assert!(names.contains(&OsString::from("written.txt")));

    sys.fs_copy("opened.txt", "copied.txt").unwrap();
    sys.fs_rename("copied.txt", "renamed.txt").unwrap();
    assert_eq!(sys.fs_read_to_string("renamed.txt").unwrap(), "opened");
    assert_eq!(
      sys.fs_canonicalize("renamed.txt").unwrap(),
      RealSys
        .fs_canonicalize(workspace.join("renamed.txt"))
        .unwrap()
    );

    sys.fs_remove_file("opened.txt").unwrap();
    sys.fs_remove_file("written.txt").unwrap();
    sys.fs_remove_file("renamed.txt").unwrap();
    sys.fs_remove_dir("sub").unwrap();
    sys.fs_create_dir_all("nested/child").unwrap();
    sys.fs_remove_dir_all("nested").unwrap();
    assert!(sys.fs_read_dir(".").unwrap().next().is_none());

    let absolute_file = temp_dir.path().join("absolute.txt");
    sys.fs_write(&absolute_file, "absolute").unwrap();
    assert_eq!(sys.fs_read_to_string(&absolute_file).unwrap(), "absolute");
    assert!(!workspace.join("absolute.txt").exists());

    assert_eq!(std::env::current_dir().unwrap(), process_cwd);
  }

  #[test]
  fn empty_paths_never_resolve_to_logical_cwd() {
    let process_cwd = std::env::current_dir().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let logical_cwd = RealSys.fs_canonicalize(temp_dir.path()).unwrap();
    let sys = CwdSys::new(RealSys).unwrap();
    sys.env_set_current_dir(&logical_cwd).unwrap();

    assert_eq!(
      sys.env_set_current_dir("").unwrap_err().kind(),
      io::ErrorKind::NotFound
    );
    assert_eq!(
      sys.fs_canonicalize("").unwrap_err().kind(),
      io::ErrorKind::NotFound
    );
    assert!(sys.fs_read("").is_err());
    assert!(sys.fs_remove_dir_all("").is_err());
    assert_eq!(sys.env_current_dir().unwrap(), logical_cwd);
    assert!(temp_dir.path().is_dir());
    assert_eq!(std::env::current_dir().unwrap(), process_cwd);
  }

  #[cfg(all(windows, feature = "strip_unc"))]
  #[test]
  fn windows_paths_use_real_sys_presentation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let expected_cwd = RealSys.fs_canonicalize(temp_dir.path()).unwrap();
    let sys = CwdSys::new(RealSys).unwrap();
    sys.env_set_current_dir(temp_dir.path()).unwrap();

    assert_eq!(sys.env_current_dir().unwrap(), expected_cwd);
    assert!(!sys
      .env_current_dir()
      .unwrap()
      .as_os_str()
      .to_string_lossy()
      .starts_with(r"\\?\"));

    sys.fs_write("file.txt", "data").unwrap();
    assert_eq!(
      sys.fs_canonicalize("file.txt").unwrap(),
      RealSys
        .fs_canonicalize(temp_dir.path().join("file.txt"))
        .unwrap()
    );
  }

  #[cfg(windows)]
  #[test]
  fn windows_verbatim_cwd_resolves_relative_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sys = CwdSys::new(VerbatimCanonicalizeSys).unwrap();
    sys.env_set_current_dir(temp_dir.path()).unwrap();

    let cwd = sys.env_current_dir().unwrap();
    // Windows doesn't normalize verbatim paths, so this relies on
    // `Path::join` handling `/`, `.`, and `..` for a verbatim base
    assert!(cwd.as_os_str().to_string_lossy().starts_with(r"\\?\"));

    sys.fs_create_dir_all("sub/child").unwrap();
    sys.fs_write("sub/child/../file.txt", "data").unwrap();
    assert_eq!(sys.fs_read_to_string("./sub/file.txt").unwrap(), "data");
    assert_eq!(sys.fs_read_dir(".").unwrap().count(), 1);

    sys.env_set_current_dir("sub/child").unwrap();
    sys.env_set_current_dir("..").unwrap();
    assert_eq!(sys.env_current_dir().unwrap(), cwd.join("sub"));
    assert_eq!(sys.fs_read_to_string("file.txt").unwrap(), "data");

    /// A real system that always returns verbatim paths (ex. `\\?\C:\dir`)
    /// when canonicalizing, regardless of the `strip_unc` feature.
    #[derive(Debug, Clone)]
    struct VerbatimCanonicalizeSys;

    impl EnvCurrentDir for VerbatimCanonicalizeSys {
      fn env_current_dir(&self) -> io::Result<PathBuf> {
        std::fs::canonicalize(RealSys.env_current_dir()?)
      }
    }

    impl BaseFsCanonicalize for VerbatimCanonicalizeSys {
      fn base_fs_canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
      }
    }

    impl BaseFsMetadata for VerbatimCanonicalizeSys {
      type Metadata = <RealSys as BaseFsMetadata>::Metadata;

      fn base_fs_metadata(&self, path: &Path) -> io::Result<Self::Metadata> {
        RealSys.base_fs_metadata(path)
      }

      fn base_fs_symlink_metadata(
        &self,
        path: &Path,
      ) -> io::Result<Self::Metadata> {
        RealSys.base_fs_symlink_metadata(path)
      }
    }

    impl BaseFsCreateDir for VerbatimCanonicalizeSys {
      fn base_fs_create_dir(
        &self,
        path: &Path,
        options: &CreateDirOptions,
      ) -> io::Result<()> {
        RealSys.base_fs_create_dir(path, options)
      }
    }

    impl BaseFsRead for VerbatimCanonicalizeSys {
      fn base_fs_read(&self, path: &Path) -> io::Result<Cow<'static, [u8]>> {
        RealSys.base_fs_read(path)
      }
    }

    impl BaseFsReadDir for VerbatimCanonicalizeSys {
      type ReadDirEntry = <RealSys as BaseFsReadDir>::ReadDirEntry;

      fn base_fs_read_dir(
        &self,
        path: &Path,
      ) -> io::Result<Box<dyn Iterator<Item = io::Result<Self::ReadDirEntry>>>>
      {
        RealSys.base_fs_read_dir(path)
      }
    }

    impl BaseFsWrite for VerbatimCanonicalizeSys {
      fn base_fs_write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        RealSys.base_fs_write(path, data)
      }
    }
  }

  #[test]
  fn constructor_and_accessors_wrap_the_inner_system() {
    let sys = CwdSys::new(RealSys).unwrap();
    let _: &RealSys = sys.inner();
    let _: RealSys = sys.into_inner();
  }
}
