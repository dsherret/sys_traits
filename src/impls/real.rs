use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::SystemTime;

use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;

use super::strip_unc_prefix;
use super::RealSys;

use crate::*;

// ==== Environment ====

impl EnvCurrentDir for RealSys {
  #[inline]
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    env::current_dir()
  }
}

impl BaseEnvSetCurrentDir for RealSys {
  #[inline]
  fn base_env_set_current_dir(&self, path: &Path) -> std::io::Result<()> {
    env::set_current_dir(path)
  }
}

impl BaseEnvVar for RealSys {
  #[inline]
  fn base_env_var_os(&self, key: &OsStr) -> Option<OsString> {
    env::var_os(key)
  }
}

impl BaseEnvSetVar for RealSys {
  #[inline]
  fn base_env_set_var(&self, key: &OsStr, value: &OsStr) {
    env::set_var(key, value);
  }
}

#[cfg(all(unix, feature = "libc"))]
impl EnvUmask for RealSys {
  fn env_umask(&self) -> std::io::Result<u32> {
    use libc::mode_t;
    use libc::umask;

    // SAFETY: libc calls
    unsafe {
      // unfortuantely there's no way to get the umask without setting it
      // temporarily... so we set the value then restore it after
      let current_umask = umask(0o000 as mode_t);
      umask(current_umask);
      Ok(current_umask as u32)
    }
  }
}

#[cfg(not(unix))]
impl EnvUmask for RealSys {
  fn env_umask(&self) -> std::io::Result<u32> {
    Err(std::io::Error::new(
      ErrorKind::Unsupported,
      "umask is not supported on this platform",
    ))
  }
}

#[cfg(all(unix, feature = "libc"))]
impl EnvSetUmask for RealSys {
  fn env_set_umask(&self, value: u32) -> std::io::Result<u32> {
    // SAFETY: libc calls
    unsafe {
      use libc::mode_t;
      use libc::umask;

      let current_umask = umask(value as mode_t);
      Ok(current_umask as u32)
    }
  }
}

#[cfg(not(unix))]
impl EnvSetUmask for RealSys {
  fn env_set_umask(&self, _umask: u32) -> std::io::Result<u32> {
    Err(std::io::Error::new(
      ErrorKind::Unsupported,
      "umask is not supported on this platform",
    ))
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
  #[inline]
  fn env_cache_dir(&self) -> Option<PathBuf> {
    known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData)
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
  #[inline]
  fn env_home_dir(&self) -> Option<PathBuf> {
    self.env_var_path("USERPROFILE").or_else(|| {
      known_folder(&windows_sys::Win32::UI::Shell::FOLDERID_Profile)
    })
  }
}

impl EnvTempDir for RealSys {
  #[inline]
  fn env_temp_dir(&self) -> std::io::Result<PathBuf> {
    Ok(env::temp_dir())
  }
}

// ==== File System ====

impl BaseFsCanonicalize for RealSys {
  #[inline]
  fn base_fs_canonicalize(&self, path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map(strip_unc_prefix)
  }
}

#[cfg(unix)]
impl BaseFsChown for RealSys {
  #[inline]
  fn base_fs_chown(
    &self,
    path: &Path,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> io::Result<()> {
    std::os::unix::fs::chown(path, uid, gid)
  }
}

#[cfg(not(unix))]
impl BaseFsChown for RealSys {
  #[inline]
  fn base_fs_chown(
    &self,
    _path: &Path,
    _uid: Option<u32>,
    _gid: Option<u32>,
  ) -> io::Result<()> {
    Err(Error::new(
      ErrorKind::Unsupported,
      "chown is not supported on this platform",
    ))
  }
}

#[cfg(unix)]
impl BaseFsSymlinkChown for RealSys {
  #[inline]
  fn base_fs_symlink_chown(
    &self,
    path: &Path,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> io::Result<()> {
    std::os::unix::fs::lchown(path, uid, gid)
  }
}

#[cfg(not(unix))]
impl BaseFsSymlinkChown for RealSys {
  #[inline]
  fn base_fs_symlink_chown(
    &self,
    _path: &Path,
    _uid: Option<u32>,
    _gid: Option<u32>,
  ) -> io::Result<()> {
    Err(Error::new(
      ErrorKind::Unsupported,
      "lchown is not supported on this platform",
    ))
  }
}

impl BaseFsCopy for RealSys {
  #[inline]
  fn base_fs_copy(&self, from: &Path, to: &Path) -> std::io::Result<u64> {
    fs::copy(from, to)
  }
}

impl BaseFsCreateDir for RealSys {
  fn base_fs_create_dir(
    &self,
    path: &Path,
    options: &CreateDirOptions,
  ) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(options.recursive);
    #[cfg(unix)]
    {
      use std::os::unix::fs::DirBuilderExt;
      if let Some(mode) = options.mode {
        builder.mode(mode);
      }
    }
    builder.create(path)
  }
}

impl BaseFsHardLink for RealSys {
  #[inline]
  fn base_fs_hard_link(&self, src: &Path, dst: &Path) -> Result<()> {
    fs::hard_link(src, dst)
  }
}

macro_rules! unix_metadata_prop {
  ($id:ident, $type:ident) => {
    #[inline]
    fn $id(&self) -> Result<$type> {
      #[cfg(unix)]
      {
        use std::os::unix::fs::MetadataExt;
        Ok(self.inner.$id())
      }
      #[cfg(not(unix))]
      {
        Err(Error::new(
          ErrorKind::Unsupported,
          concat!(stringify!($id), " is not supported on this platform"),
        ))
      }
    }
  };
}

macro_rules! unix_win_extra_metadata_prop {
  ($id:ident, $type:ident) => {
    #[inline]
    fn $id(&self) -> Result<$type> {
      #[cfg(unix)]
      {
        use std::os::unix::fs::MetadataExt;
        Ok(self.inner.$id())
      }
      #[cfg(not(unix))]
      {
        let win_extra = self.get_or_init_stat_info()?;
        Ok(win_extra.$id)
      }
    }
  };
}

macro_rules! unix_metadata_file_type_prop {
  ($id:ident, $type:ident) => {
    #[inline]
    fn $id(&self) -> Result<$type> {
      #[cfg(unix)]
      {
        use std::os::unix::fs::FileTypeExt;
        Ok(self.inner.file_type().$id())
      }
      #[cfg(not(unix))]
      {
        Err(Error::new(
          ErrorKind::Unsupported,
          concat!(stringify!($id), " is not supported on this platform"),
        ))
      }
    }
  };
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
enum StatKind {
  Stat,
  SymlinkStat,
}

// this information requires opening a file handle, so it's only done once
#[cfg(windows)]
#[derive(Debug, Clone, Default)]
struct WinStatInfo {
  dev: u64,
  ctime: Option<SystemTime>,
  mode: u32,
}

/// A wrapper type is used in order to force usages to
/// `use sys_traits::FsMetadataValue` so that the code
/// compiles under Wasm.
#[derive(Debug, Clone)]
pub struct RealFsMetadata {
  inner: fs::Metadata,
  #[cfg(windows)]
  path: PathBuf,
  #[cfg(windows)]
  kind: StatKind,
  #[cfg(windows)]
  stat_info: OnceLock<std::result::Result<WinStatInfo, (ErrorKind, String)>>,
}

#[cfg(windows)]
impl RealFsMetadata {
  pub fn get_or_init_stat_info(&self) -> Result<&WinStatInfo> {
    let result = self
      .stat_info
      .get_or_init(|| {
        let mut info = WinStatInfo::default();
        stat_extra(&mut info, &self.path, FILE_FLAG_BACKUP_SEMANTICS)
          .map(|_| info)
          .map_err(|err| (err.kind(), err.to_string()))
      })
      .as_ref();
    result.map_err(|(kind, msg)| Error::new(kind.clone(), msg.clone()))
  }
}

#[cfg(windows)]
fn stat_extra(
  fsstat: &mut WinStatInfo,
  path: &Path,
  file_flags: windows_sys::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
) -> Result<()> {
  use std::ffi::c_int;
  use std::os::windows::prelude::OsStrExt;
  use std::time::Duration;
  use windows_sys::Wdk::Storage::FileSystem::FileAllInformation;
  use windows_sys::Wdk::Storage::FileSystem::NtQueryInformationFile;
  use windows_sys::Wdk::Storage::FileSystem::FILE_ALL_INFORMATION;
  use windows_sys::Win32::Foundation::CloseHandle;
  use windows_sys::Win32::Foundation::RtlNtStatusToDosError;
  use windows_sys::Win32::Foundation::ERROR_MORE_DATA;
  use windows_sys::Win32::Foundation::FALSE;
  use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
  use windows_sys::Win32::Foundation::NTSTATUS;
  use windows_sys::Win32::Storage::FileSystem::CreateFileW;
  use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandle;
  use windows_sys::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION;
  use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;
  use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_READONLY;
  use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
  use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
  use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
  use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
  use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
  use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;

  struct WinHandle(*mut std::ffi::c_void);

  impl Drop for WinHandle {
    fn drop(&mut self) {
      // SAFETY: winapi call
      unsafe {
        CloseHandle(self.0);
      }
    }
  }

  unsafe fn get_dev(handle: *mut std::ffi::c_void) -> std::io::Result<u64> {
    let info = {
      let mut info =
        std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
      if GetFileInformationByHandle(handle, info.as_mut_ptr()) == FALSE {
        return Err(std::io::Error::last_os_error());
      }

      info.assume_init()
    };

    Ok(info.dwVolumeSerialNumber as u64)
  }

  const WINDOWS_TO_UNIX_EPOCH_SECS: i64 = 11_644_473_600; // Seconds between Windows epoch and Unix epoch

  fn windows_time_to_system_time(windows_time: i64) -> SystemTime {
    // windows_time is in 100ns intervals since 1601-01-01
    let secs = windows_time / 10_000_000;
    let nanos = ((windows_time % 10_000_000) * 100) as u32;

    if secs >= WINDOWS_TO_UNIX_EPOCH_SECS {
      // Time is after the Unix epoch
      SystemTime::UNIX_EPOCH
        + Duration::new((secs - WINDOWS_TO_UNIX_EPOCH_SECS) as u64, nanos)
    } else {
      // Time is before the Unix epoch
      let duration_since_unix =
        Duration::new((WINDOWS_TO_UNIX_EPOCH_SECS - secs) as u64, nanos);
      SystemTime::UNIX_EPOCH - duration_since_unix
    }
  }

  unsafe fn query_file_information(
    handle: *mut std::ffi::c_void,
  ) -> std::result::Result<FILE_ALL_INFORMATION, NTSTATUS> {
    let mut info = std::mem::MaybeUninit::<FILE_ALL_INFORMATION>::zeroed();
    let mut io_status_block =
      std::mem::MaybeUninit::<IO_STATUS_BLOCK>::zeroed();
    let status = NtQueryInformationFile(
      handle as _,
      io_status_block.as_mut_ptr(),
      info.as_mut_ptr() as *mut _,
      std::mem::size_of::<FILE_ALL_INFORMATION>() as _,
      FileAllInformation,
    );

    if status < 0 {
      let converted_status = RtlNtStatusToDosError(status);

      // If error more data is returned, then it means that the buffer is too small to get full filename information
      // to have that we should retry. However, since we only use BasicInformation and StandardInformation, it is fine to ignore it
      // since struct is populated with other data anyway.
      // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntqueryinformationfile#remarksdd
      if converted_status != ERROR_MORE_DATA {
        return Err(converted_status as NTSTATUS);
      }
    }

    Ok(info.assume_init())
  }

  // SAFETY: winapi calls
  unsafe {
    let mut path: Vec<_> = path.as_os_str().encode_wide().collect();
    path.push(0);
    let file_handle = CreateFileW(
      path.as_ptr(),
      0,
      FILE_SHARE_READ | FILE_SHARE_DELETE | FILE_SHARE_WRITE,
      std::ptr::null_mut(),
      OPEN_EXISTING,
      file_flags,
      std::ptr::null_mut(),
    );
    if file_handle == INVALID_HANDLE_VALUE {
      return Err(std::io::Error::last_os_error().into());
    }
    let file_handle = WinHandle(file_handle);

    fsstat.dev = get_dev(file_handle.0)?;

    if let Ok(file_info) = query_file_information(file_handle.0) {
      fsstat.ctime = Some(windows_time_to_system_time(
        file_info.BasicInformation.ChangeTime,
      ));

      if file_info.BasicInformation.FileAttributes
        & FILE_ATTRIBUTE_REPARSE_POINT
        != 0
      {
        // fsstat.is_symlink = true;
      }

      const S_IFDIR: c_int = 0o4_0000;
      const S_IFREG: c_int = 0o10_0000;
      const S_IREAD: c_int = 0o0400;
      const S_IWRITE: c_int = 0o0200;

      if file_info.BasicInformation.FileAttributes & FILE_ATTRIBUTE_DIRECTORY
        != 0
      {
        fsstat.mode |= S_IFDIR as u32;
        // fsstat.size = 0;
      } else {
        fsstat.mode |= S_IFREG as u32;
        // fsstat.size = file_info.StandardInformation.EndOfFile as u64;
      }

      if file_info.BasicInformation.FileAttributes & FILE_ATTRIBUTE_READONLY
        != 0
      {
        fsstat.mode |= (S_IREAD | (S_IREAD >> 3) | (S_IREAD >> 6)) as u32;
      } else {
        fsstat.mode |= ((S_IREAD | S_IWRITE)
          | ((S_IREAD | S_IWRITE) >> 3)
          | ((S_IREAD | S_IWRITE) >> 6)) as u32;
      }
    }

    Ok(())
  }
}

impl FsMetadataValue for RealFsMetadata {
  #[inline]
  fn file_type(&self) -> FileType {
    self.inner.file_type().into()
  }

  #[inline]
  fn len(&self) -> u64 {
    self.inner.len()
  }

  #[inline]
  fn accessed(&self) -> Result<SystemTime> {
    self.inner.accessed()
  }

  #[inline]
  fn changed(&self) -> Result<SystemTime> {
    #[cfg(unix)]
    {
      use std::os::unix::fs::MetadataExt;
      let changed = self.inner.ctime();
      Ok(
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(changed as u64),
      )
    }
    #[cfg(not(unix))]
    {
      let win_extra = self.get_or_init_stat_info()?;
      win_extra
        .ctime
        .ok_or_else(|| Error::new(ErrorKind::Other, "failed to get ctime"))
    }
  }

  #[inline]
  fn created(&self) -> Result<SystemTime> {
    self.inner.created()
  }

  #[inline]
  fn modified(&self) -> Result<SystemTime> {
    self.inner.modified()
  }

  unix_win_extra_metadata_prop!(dev, u64);
  unix_metadata_prop!(ino, u64);
  unix_win_extra_metadata_prop!(mode, u32);
  unix_metadata_prop!(nlink, u64);
  unix_metadata_prop!(uid, u32);
  unix_metadata_prop!(gid, u32);
  unix_metadata_prop!(rdev, u64);
  unix_metadata_prop!(blksize, u64);
  unix_metadata_prop!(blocks, u64);
  unix_metadata_file_type_prop!(is_block_device, bool);
  unix_metadata_file_type_prop!(is_char_device, bool);
  unix_metadata_file_type_prop!(is_fifo, bool);
  unix_metadata_file_type_prop!(is_socket, bool);

  fn file_attributes(&self) -> io::Result<u32> {
    #[cfg(windows)]
    {
      use std::os::windows::prelude::MetadataExt;
      Ok(self.inner.file_attributes())
    }
    #[cfg(not(windows))]
    {
      Err(Error::new(
        ErrorKind::Unsupported,
        "file_attributes is not supported on this platform",
      ))
    }
  }
}

impl BaseFsMetadata for RealSys {
  type Metadata = RealFsMetadata;

  #[inline]
  fn base_fs_metadata(&self, path: &Path) -> Result<Self::Metadata> {
    fs::metadata(path).map(|inner| RealFsMetadata {
      inner,
      #[cfg(windows)]
      path: path.to_path_buf(),
      #[cfg(windows)]
      kind: StatKind::Stat,
    })
  }

  #[inline]
  fn base_fs_symlink_metadata(&self, path: &Path) -> Result<Self::Metadata> {
    fs::symlink_metadata(path).map(|inner| RealFsMetadata {
      inner,
      #[cfg(windows)]
      kind: StatKind::SymlinkStat,
    })
  }
}

impl BaseFsOpen for RealSys {
  type File = RealFsFile;

  fn base_fs_open(
    &self,
    path: &Path,
    options: &OpenOptions,
  ) -> std::io::Result<Self::File> {
    let mut builder = fs::OpenOptions::new();
    if let Some(mode) = options.mode {
      #[cfg(unix)]
      {
        use std::os::unix::fs::OpenOptionsExt;
        builder.mode(mode);
      }
      #[cfg(not(unix))]
      let _ = mode;
    }
    if let Some(flags) = options.custom_flags {
      #[cfg(unix)]
      {
        use std::os::unix::fs::OpenOptionsExt;
        builder.custom_flags(flags);
      }
      #[cfg(windows)]
      {
        use std::os::windows::fs::OpenOptionsExt;
        builder.custom_flags(flags);
      }
      #[cfg(all(not(windows), not(unix)))]
      let _ = flags;
    }
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

impl BaseFsRead for RealSys {
  #[inline]
  fn base_fs_read(&self, path: &Path) -> Result<Cow<'static, [u8]>> {
    fs::read(path).map(Cow::Owned)
  }
}

#[derive(Debug)]
pub struct RealFsDirEntry(fs::DirEntry);

impl FsDirEntry for RealFsDirEntry {
  type Metadata = RealFsMetadata;

  #[inline]
  fn file_name(&self) -> Cow<OsStr> {
    Cow::Owned(self.0.file_name())
  }

  #[inline]
  fn file_type(&self) -> std::io::Result<FileType> {
    self.0.file_type().map(FileType::from)
  }

  #[inline]
  fn metadata(&self) -> std::io::Result<Self::Metadata> {
    self.0.metadata().map(|inner| RealFsMetadata {
      inner,
      #[cfg(windows)]
      kind: StatKind::Stat,
    })
  }

  #[inline]
  fn path(&self) -> Cow<Path> {
    Cow::Owned(self.0.path())
  }
}

impl BaseFsReadDir for RealSys {
  type ReadDirEntry = RealFsDirEntry;

  #[inline]
  fn base_fs_read_dir(
    &self,
    path: &Path,
  ) -> std::io::Result<
    Box<dyn Iterator<Item = std::io::Result<Self::ReadDirEntry>>>,
  > {
    let iterator = fs::read_dir(path)?;
    Ok(Box::new(iterator.map(|result| result.map(RealFsDirEntry))))
  }
}

impl BaseFsReadLink for RealSys {
  fn base_fs_read_link(&self, path: &Path) -> io::Result<PathBuf> {
    fs::read_link(path)
  }
}

impl BaseFsRemoveDir for RealSys {
  #[inline]
  fn base_fs_remove_dir(&self, path: &Path) -> std::io::Result<()> {
    fs::remove_dir(path)
  }
}

impl BaseFsRemoveDirAll for RealSys {
  #[inline]
  fn base_fs_remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
    fs::remove_dir_all(path)
  }
}

impl BaseFsRemoveFile for RealSys {
  #[inline]
  fn base_fs_remove_file(&self, path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)
  }
}

impl BaseFsRename for RealSys {
  #[inline]
  fn base_fs_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
    fs::rename(from, to)
  }
}

#[cfg(feature = "filetime")]
impl BaseFsSetFileTimes for RealSys {
  #[inline]
  fn base_fs_set_file_times(
    &self,
    path: &Path,
    atime: SystemTime,
    mtime: SystemTime,
  ) -> Result<()> {
    let atime = filetime::FileTime::from_system_time(atime);
    let mtime = filetime::FileTime::from_system_time(mtime);
    filetime::set_file_times(path, atime, mtime)
  }
}

#[cfg(feature = "filetime")]
impl BaseFsSetSymlinkFileTimes for RealSys {
  #[inline]
  fn base_fs_set_symlink_file_times(
    &self,
    path: &Path,
    atime: SystemTime,
    mtime: SystemTime,
  ) -> Result<()> {
    let atime = filetime::FileTime::from_system_time(atime);
    let mtime = filetime::FileTime::from_system_time(mtime);
    filetime::set_symlink_file_times(path, atime, mtime)
  }
}

#[cfg(unix)]
impl BaseFsSetPermissions for RealSys {
  #[inline]
  fn base_fs_set_permissions(
    &self,
    path: &Path,
    mode: u32,
  ) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions)
  }
}

#[cfg(not(unix))]
impl BaseFsSetPermissions for RealSys {
  fn base_fs_set_permissions(
    &self,
    _path: &Path,
    _mode: u32,
  ) -> std::io::Result<()> {
    Err(std::io::Error::new(
      ErrorKind::Unsupported,
      "cannot set path permissions on this platform",
    ))
  }
}

impl BaseFsSymlinkDir for RealSys {
  fn base_fs_symlink_dir(
    &self,
    original: &Path,
    link: &Path,
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

impl BaseFsSymlinkFile for RealSys {
  fn base_fs_symlink_file(
    &self,
    original: &Path,
    link: &Path,
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

impl BaseFsWrite for RealSys {
  #[inline]
  fn base_fs_write(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
    fs::write(path, data)
  }
}

// ==== File System File ====

/// A wrapper type is used in order to force usages to
/// `use sys_traits::FsFile` so that the code
/// compiles under Wasm.
#[derive(Debug)]
pub struct RealFsFile(fs::File);

impl FsFile for RealFsFile {}

impl FsFileSetLen for RealFsFile {
  #[inline]
  fn fs_file_set_len(&mut self, size: u64) -> std::io::Result<()> {
    self.0.set_len(size)
  }
}

impl FsFileSetPermissions for RealFsFile {
  #[inline]
  fn fs_file_set_permissions(&mut self, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let permissions = fs::Permissions::from_mode(mode);
      self.0.set_permissions(permissions)
    }
    #[cfg(not(unix))]
    {
      let _ = mode;
      Ok(())
    }
  }
}

impl std::io::Seek for RealFsFile {
  #[inline]
  fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64> {
    self.0.seek(pos)
  }
}

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

impl std::io::Read for RealFsFile {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    self.0.read(buf)
  }
}

// ==== System ====

impl SystemTimeNow for RealSys {
  #[inline]
  fn sys_time_now(&self) -> SystemTime {
    SystemTime::now()
  }
}

#[cfg(feature = "getrandom")]
impl crate::SystemRandom for RealSys {
  #[inline]
  fn sys_random(&self, buf: &mut [u8]) -> Result<()> {
    getrandom::getrandom(buf)
      .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
  }
}

impl crate::ThreadSleep for RealSys {
  #[inline]
  fn thread_sleep(&self, duration: std::time::Duration) {
    std::thread::sleep(duration);
  }
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

  #[cfg(all(unix, feature = "libc"))]
  #[test]
  fn test_umask() {
    let original_umask = RealSys.env_umask().unwrap();
    assert_eq!(RealSys.env_set_umask(0o777).unwrap(), original_umask);
    assert_eq!(RealSys.env_set_umask(original_umask).unwrap(), 0o777);
  }

  #[cfg(target_os = "windows")]
  #[test]
  fn test_umask() {
    let err = RealSys.env_umask().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Unsupported);
    let err = RealSys.env_set_umask(0o000).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Unsupported);
  }

  #[test]
  fn test_general() {
    assert!(RealSys.sys_time_now().elapsed().is_ok());
  }
}
