use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;
use std::time::SystemTime;

use crate::FileType;
use crate::FsDirEntry;
use crate::FsFile;
use crate::FsFileSetPermissions;
use crate::FsMetadataValue;
use crate::FsOpenImpl;
use crate::FsReadDirImpl;
use crate::OpenOptions;

// == FsOpenBoxed ==

pub struct BoxedFsFile(pub Box<dyn FsFile + 'static>);

impl std::io::Read for BoxedFsFile {
  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.0.read(buf)
  }
}

impl std::io::Seek for BoxedFsFile {
  #[inline]
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    self.0.seek(pos)
  }
}

impl std::io::Write for BoxedFsFile {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.0.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.0.flush()
  }
}

impl FsFileSetPermissions for BoxedFsFile {
  #[inline]
  fn fs_file_set_permissions(&mut self, perm: u32) -> std::io::Result<()> {
    self.0.fs_file_set_permissions(perm)
  }
}

impl FsFile for BoxedFsFile {}

pub trait FsOpenBoxed {
  fn fs_open_boxed(
    &self,
    path: &Path,
    open_options: &OpenOptions,
  ) -> std::io::Result<BoxedFsFile>;
}

impl<T: FsOpenImpl> FsOpenBoxed for T {
  fn fs_open_boxed(
    &self,
    path: &Path,
    open_options: &OpenOptions,
  ) -> std::io::Result<BoxedFsFile> {
    self
      .fs_open_impl(path, open_options)
      .map(|file| BoxedFsFile(Box::new(file)))
  }
}

// == FsReadDirBoxed ==

#[derive(Debug)]
pub struct BoxedFsMetadataValue(pub Box<dyn FsMetadataValue>);

impl FsMetadataValue for BoxedFsMetadataValue {
  #[inline]
  fn file_type(&self) -> FileType {
    self.0.file_type()
  }

  #[inline]
  fn modified(&self) -> std::io::Result<SystemTime> {
    self.0.modified()
  }
}

#[derive(Debug)]
struct MappedMetadataFsDirEntry<T: FsDirEntry + 'static>(T);

impl<T: FsDirEntry + 'static> FsDirEntry for MappedMetadataFsDirEntry<T> {
  type Metadata = BoxedFsMetadataValue;

  #[inline]
  fn file_name(&self) -> Cow<OsStr> {
    self.0.file_name()
  }

  #[inline]
  fn file_type(&self) -> std::io::Result<FileType> {
    self.0.file_type()
  }

  #[inline]
  fn metadata(&self) -> std::io::Result<Self::Metadata> {
    self
      .0
      .metadata()
      .map(|metadata| BoxedFsMetadataValue(Box::new(metadata)))
  }

  #[inline]
  fn path(&self) -> Cow<Path> {
    self.0.path()
  }
}

#[derive(Debug)]
pub struct BoxedFsDirEntry(
  Box<dyn FsDirEntry<Metadata = BoxedFsMetadataValue>>,
);

impl BoxedFsDirEntry {
  pub fn new<T: FsDirEntry + 'static>(entry: T) -> Self {
    Self(Box::new(MappedMetadataFsDirEntry(entry)))
  }
}

impl FsDirEntry for BoxedFsDirEntry {
  type Metadata = BoxedFsMetadataValue;

  #[inline]
  fn file_name(&self) -> Cow<OsStr> {
    self.0.file_name()
  }

  #[inline]
  fn file_type(&self) -> std::io::Result<FileType> {
    self.0.file_type()
  }

  #[inline]
  fn metadata(&self) -> std::io::Result<Self::Metadata> {
    self.0.metadata()
  }

  #[inline]
  fn path(&self) -> Cow<Path> {
    self.0.path()
  }
}

pub trait FsReadDirBoxed {
  fn fs_read_dir_boxed(
    &self,
    path: &Path,
  ) -> std::io::Result<Box<dyn Iterator<Item = std::io::Result<BoxedFsDirEntry>>>>;
}

impl<T: FsReadDirImpl> FsReadDirBoxed for T {
  fn fs_read_dir_boxed(
    &self,
    path: &Path,
  ) -> std::io::Result<Box<dyn Iterator<Item = std::io::Result<BoxedFsDirEntry>>>>
  {
    let iter = self.fs_read_dir_impl(path)?;
    Ok(Box::new(
      iter.map(|result| result.map(BoxedFsDirEntry::new)),
    ))
  }
}
