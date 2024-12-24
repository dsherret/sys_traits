use std::collections::HashSet;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::path::Component;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

// this entire module was lazily created... needs way more work

use parking_lot::RwLock;

use crate::*;

use super::RealSys;

#[derive(Debug, Clone)]
pub struct InMemoryFile {
  sys: InMemorySys,
  inner: Arc<RwLock<FileInner>>,
  pos: usize,
}

#[derive(Debug)]
struct FileInner {
  #[allow(dead_code)]
  created_time: SystemTime,
  modified_time: SystemTime,
  data: Vec<u8>,
  mode: u32,
}

#[derive(Debug)]
struct File {
  name: String,
  inner: Arc<RwLock<FileInner>>,
}

#[derive(Debug)]
enum DirectoryEntry {
  File(File),
  Directory(Directory),
  Symlink(Symlink),
}

impl DirectoryEntry {
  fn name(&self) -> &str {
    match self {
      DirectoryEntry::File(f) => &f.name,
      DirectoryEntry::Directory(d) => &d.name,
      DirectoryEntry::Symlink(s) => &s.name,
    }
  }

  fn modified_time(&self) -> SystemTime {
    match self {
      DirectoryEntry::File(f) => f.inner.read().modified_time,
      DirectoryEntry::Directory(d) => d.inner.read().modified_time,
      DirectoryEntry::Symlink(s) => s.inner.read().modified_time,
    }
  }
}

#[derive(Debug)]
struct SymlinkInner {
  #[allow(dead_code)]
  created_time: SystemTime,
  modified_time: SystemTime,
}

#[derive(Debug)]
struct Symlink {
  name: String,
  target: PathBuf,
  inner: RwLock<SymlinkInner>,
}

#[derive(Debug)]
struct DirectoryInner {
  #[allow(dead_code)]
  created_time: SystemTime,
  modified_time: SystemTime,
}

#[derive(Debug)]
struct Directory {
  name: String,
  inner: RwLock<DirectoryInner>,
  entries: Vec<DirectoryEntry>,
}

enum LookupEntry<'a> {
  NotFound(PathBuf),
  Found(PathBuf, &'a DirectoryEntry),
}

#[derive(Debug)]
struct InMemorySysInner {
  // Linux/Mac will always have one dir here, but Windows
  // may have multiple per drive.
  system_root: Vec<DirectoryEntry>,
  cwd: PathBuf,
  thread_sleep_enabled: bool,
  random_seed: Option<u64>,
  time: Option<SystemTime>,
}

impl InMemorySysInner {
  fn to_absolute_path(&self, p: &Path) -> PathBuf {
    if p.is_absolute() {
      normalize_path(p)
    } else {
      normalize_path(&self.cwd.join(p))
    }
  }

  fn time_now(&self) -> SystemTime {
    self.time.unwrap_or_else(|| RealSys.sys_time_now())
  }

  fn lookup_entry<'a>(
    &'a self,
    path: &Path,
  ) -> Result<(PathBuf, &'a DirectoryEntry)> {
    match self.lookup_entry_detail(path)? {
      LookupEntry::Found(path, entry) => Ok((path, entry)),
      LookupEntry::NotFound(_) => Err(Error::new(
        ErrorKind::NotFound,
        format!("Path not found: '{}'", path.display()),
      )),
    }
  }

  fn lookup_entry_detail<'a>(&'a self, path: &Path) -> Result<LookupEntry<'a>> {
    let mut final_path = Vec::new();
    let mut seen_entries = HashSet::new();
    let mut path = Cow::Borrowed(path);
    let mut comps = path.components().peekable();
    if comps.peek().is_none() {
      return Err(Error::new(ErrorKind::NotFound, "Empty path"));
    }

    let mut entries = &self.system_root;
    while let Some(comp) = comps.next() {
      final_path.push(comp);
      let comp = match comp {
        Component::RootDir => Cow::Borrowed(""),
        Component::Prefix(component) => {
          let component = component.as_os_str().to_string_lossy();
          if let Some(comp) = comps.next() {
            final_path.push(comp);
          }
          component
        }
        component => component.as_os_str().to_string_lossy(),
      };
      let pos = match entries.binary_search_by(|e| e.name().cmp(&comp)) {
        Ok(p) => p,
        Err(_) => {
          return Ok(LookupEntry::NotFound(
            final_path.into_iter().chain(comps).collect(),
          ));
        }
      };

      match &entries[pos] {
        DirectoryEntry::Directory(dir) => {
          if comps.peek().is_none() {
            return Ok(LookupEntry::Found(
              final_path.into_iter().collect(),
              &entries[pos],
            ));
          } else {
            entries = &dir.entries;
          }
        }
        DirectoryEntry::File(_) => {
          if comps.peek().is_none() {
            return Ok(LookupEntry::Found(
              final_path.into_iter().collect(),
              &entries[pos],
            ));
          } else {
            return Err(Error::new(
              ErrorKind::Other,
              "Path leads into a file or symlink",
            ));
          }
        }
        DirectoryEntry::Symlink(symlink) => {
          let current_path = final_path.into_iter().collect::<PathBuf>();
          let target_path = normalize_path(&current_path.join(&symlink.target));
          if seen_entries.is_empty() {
            // add the original path at this point in order to avoid allocating when we
            // don't have symlinks
            seen_entries.insert(current_path.clone());
          }
          if !seen_entries.insert(target_path.clone()) {
            return Err(Error::new(
              ErrorKind::Other,
              format!("Symlink loop detected resolving '{}'", path.display()),
            ));
          }

          // reset and start resolving the target path
          final_path = Vec::new();
          entries = &self.system_root;
          path = Cow::Owned(target_path);
          comps = path.components().peekable();
        }
      }
    }

    Ok(LookupEntry::NotFound(final_path.into_iter().collect()))
  }

  fn find_directory_mut<'a>(
    &'a mut self,
    path: &Path,
    create_dirs: bool,
  ) -> Result<&'a mut Directory> {
    // ran into a lot of issues with the borrow checker... recommendation was to
    // resolve symlinks first then resolve the path
    let path = match self.lookup_entry_detail(path)? {
      LookupEntry::Found(path, _) => path,
      LookupEntry::NotFound(path) => path,
    };

    let time = self.time_now();
    let mut comps = path.components().peekable();
    if comps.peek().is_none() {
      return Err(Error::new(ErrorKind::NotFound, "Empty path"));
    }

    let mut entries = &mut self.system_root;
    while let Some(comp) = comps.next() {
      let comp = match comp {
        Component::RootDir => Cow::Borrowed(""),
        Component::Prefix(component) => {
          let component = component.as_os_str().to_string_lossy();
          comps.next();
          component
        }
        component => component.as_os_str().to_string_lossy(),
      };
      let pos = match entries.binary_search_by(|e| e.name().cmp(&comp)) {
        Ok(p) => p,
        Err(insert_pos) => {
          if create_dirs {
            let new_dir = Directory {
              name: comp.into_owned(),
              inner: RwLock::new(DirectoryInner {
                created_time: time,
                modified_time: time,
              }),
              entries: vec![],
            };
            entries.insert(insert_pos, DirectoryEntry::Directory(new_dir));
            insert_pos
          } else {
            return Err(Error::new(ErrorKind::NotFound, "Path not found"));
          }
        }
      };

      match &mut entries[pos] {
        DirectoryEntry::Directory(dir) => {
          if comps.peek().is_none() {
            return Ok(dir);
          } else {
            entries = &mut dir.entries;
          }
        }
        DirectoryEntry::File(_) | DirectoryEntry::Symlink { .. } => {
          return Err(Error::new(
            ErrorKind::Other,
            "Path leads into a file or symlink",
          ));
        }
      }
    }

    Err(Error::new(ErrorKind::NotFound, "Path not found"))
  }
}

/// An in-memory system implementation useful for testing.
///
/// This is extremely untested and sloppily implemented. Use with extreme caution
/// and only for testing. You will encounter bugs. Please submit fixes. I implemented
/// this lazily and quickly.
#[derive(Debug, Clone)]
pub struct InMemorySys(Arc<RwLock<InMemorySysInner>>);

impl Default for InMemorySys {
  fn default() -> Self {
    Self(Arc::new(RwLock::new(InMemorySysInner {
      system_root: vec![],
      cwd: PathBuf::from("/"),
      thread_sleep_enabled: true,
      random_seed: None,
      time: None,
    })))
  }
}

impl InMemorySys {
  pub fn set_seed(&self, seed: Option<u64>) {
    self.0.write().random_seed = seed;
  }

  pub fn set_time(&self, time: Option<SystemTime>) {
    self.0.write().time = time;
  }

  /// Makes thread sleeping a no-op.
  pub fn disable_thread_sleep(&self) {
    self.0.write().thread_sleep_enabled = false;
  }
}

impl EnvCurrentDir for InMemorySys {
  fn env_current_dir(&self) -> std::io::Result<PathBuf> {
    Ok(self.0.read().cwd.clone())
  }
}

impl EnvSetCurrentDir for InMemorySys {
  fn env_set_current_dir(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = self.fs_canonicalize(path)?; // cause an error if not exists
    self.0.write().cwd = path;
    Ok(())
  }
}

// File System

impl FsCanonicalize for InMemorySys {
  fn fs_canonicalize(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
    let inner = self.0.read();
    let path = inner.to_absolute_path(path.as_ref());
    let (path, _) = inner.lookup_entry(&path)?;
    Ok(path)
  }
}

impl FsCreateDirAll for InMemorySys {
  fn fs_create_dir_all(&self, path: impl AsRef<Path>) -> Result<()> {
    let mut inner = self.0.write();
    let abs = inner.to_absolute_path(path.as_ref());
    inner.find_directory_mut(&abs, true)?;
    Ok(())
  }
}

impl FsExists for InMemorySys {
  fn fs_exists(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    let inner = self.0.read();
    let lookup = inner.lookup_entry(path.as_ref());
    Ok(lookup.is_ok())
  }
}

impl FsIsFile for InMemorySys {
  fn fs_is_file(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    let inner = self.0.read();
    let (_, entry) = inner.lookup_entry(path.as_ref())?;
    match entry {
      DirectoryEntry::File(_) => Ok(true),
      _ => Ok(false),
    }
  }
}

impl FsIsDir for InMemorySys {
  fn fs_is_dir(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
    let inner = self.0.read();
    let (_, entry) = inner.lookup_entry(path.as_ref())?;
    match entry {
      DirectoryEntry::Directory(_) => Ok(true),
      _ => Ok(false),
    }
  }
}

impl FsModified for InMemorySys {
  fn fs_modified(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<std::io::Result<SystemTime>> {
    let inner = self.0.read();
    let (_, entry) = inner.lookup_entry(path.as_ref())?;
    Ok(Ok(entry.modified_time()))
  }
}

impl FsOpen<InMemoryFile> for InMemorySys {
  fn fs_open(
    &self,
    path: impl AsRef<Path>,
    options: &OpenOptions,
  ) -> std::io::Result<InMemoryFile> {
    let mut inner = self.0.write();
    let time_now = inner.time_now();
    let path = inner.to_absolute_path(path.as_ref());

    // Edge case: If `parent()` is None, path might be root or invalid
    // The minimal fix is to check for that scenario
    let parent_path = match path.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot open root or invalid path",
        ));
      }
    };

    let parent = inner.find_directory_mut(parent_path, false)?;
    let file_name = match path.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        return Err(Error::new(ErrorKind::Other, "No file name found"));
      }
    };

    match parent
      .entries
      .binary_search_by(|e| e.name().cmp(&file_name))
    {
      Ok(pos) => match &mut parent.entries[pos] {
        DirectoryEntry::File(f) => {
          if options.create_new {
            return Err(Error::new(
              ErrorKind::AlreadyExists,
              "File already exists (create_new=true)",
            ));
          }
          if options.truncate {
            let mut fi = f.inner.write();
            fi.data.clear();
            fi.modified_time = time_now;
          }
          Ok(InMemoryFile {
            sys: self.clone(),
            inner: f.inner.clone(),
            pos: if options.append {
              f.inner.read().data.len()
            } else {
              0
            },
          })
        }
        _ => Err(Error::new(ErrorKind::Other, "Path is not a file")),
      },
      Err(insert_pos) => {
        if !options.create {
          return Err(Error::new(ErrorKind::NotFound, "File not found"));
        }
        let new_file = File {
          name: file_name.into_owned(),
          inner: Arc::new(RwLock::new(FileInner {
            created_time: time_now,
            modified_time: time_now,
            data: vec![],
            mode: 0o644,
          })),
        };
        let result = InMemoryFile {
          sys: self.clone(),
          inner: new_file.inner.clone(),
          pos: if options.append {
            new_file.inner.read().data.len()
          } else {
            0
          },
        };
        parent
          .entries
          .insert(insert_pos, DirectoryEntry::File(new_file));
        Ok(result)
      }
    }
  }
}

impl FsRead for InMemorySys {
  fn fs_read(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, [u8]>> {
    let arc_file = self.fs_open(path, &OpenOptions::read())?;
    let inner = arc_file.inner.read();
    Ok(Cow::Owned(inner.data.clone()))
  }
}

impl FsReadToString for InMemorySys {
  fn fs_read_to_string(
    &self,
    path: impl AsRef<Path>,
  ) -> std::io::Result<Cow<'static, str>> {
    let bytes = self.fs_read(path)?;
    match String::from_utf8(bytes.to_vec()) {
      Ok(s) => Ok(Cow::Owned(s)),
      Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
    }
  }
}

impl FsRemoveFile for InMemorySys {
  fn fs_remove_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut inner = self.0.write();
    let path = inner.to_absolute_path(path.as_ref());
    let parent_path = match path.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot remove root or invalid path",
        ));
      }
    };
    let parent = inner.find_directory_mut(parent_path, false)?;
    let file_name = match path.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        return Err(Error::new(ErrorKind::Other, "No file name found"));
      }
    };

    match parent
      .entries
      .binary_search_by(|e| e.name().cmp(&file_name))
    {
      Ok(pos) => match &parent.entries[pos] {
        DirectoryEntry::File(_) => {
          parent.entries.remove(pos);
          Ok(())
        }
        _ => Err(Error::new(ErrorKind::Other, "Not a file")),
      },
      Err(_) => Err(Error::new(ErrorKind::NotFound, "File not found")),
    }
  }
}

impl FsRename for InMemorySys {
  fn fs_rename(
    &self,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    let mut inner = self.0.write();
    let from = inner.to_absolute_path(from.as_ref());
    let to = inner.to_absolute_path(to.as_ref());

    let from_parent_path = match from.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot rename root or invalid path",
        ));
      }
    };
    let from_file_name = match from.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        return Err(Error::new(ErrorKind::Other, "No source file name found"));
      }
    };

    let from_parent = inner.find_directory_mut(from_parent_path, false)?;
    let from_idx = match from_parent
      .entries
      .binary_search_by(|e| e.name().cmp(&from_file_name))
    {
      Ok(pos) => pos,
      Err(_) => {
        return Err(Error::new(ErrorKind::NotFound, "Source file not found"));
      }
    };
    let file_entry = from_parent.entries.remove(from_idx);

    let to_parent_path = match to.parent() {
      Some(p) if !p.as_os_str().is_empty() => p,
      _ => {
        // If `to` has no valid parent, restore the original file entry:
        from_parent.entries.insert(from_idx, file_entry);
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot rename to root or invalid path",
        ));
      }
    };
    let to_file_name = match to.file_name() {
      Some(n) => n.to_string_lossy(),
      None => {
        // restore
        from_parent.entries.insert(from_idx, file_entry);
        return Err(Error::new(
          ErrorKind::Other,
          "No destination file name found",
        ));
      }
    };

    let to_parent = inner.find_directory_mut(to_parent_path, true)?;
    match file_entry {
      DirectoryEntry::File(mut f) => {
        match to_parent
          .entries
          .binary_search_by(|e| e.name().cmp(&to_file_name))
        {
          Ok(pos) => match &to_parent.entries[pos] {
            DirectoryEntry::Directory(_) => {
              let from_parent =
                inner.find_directory_mut(from_parent_path, false)?;
              from_parent
                .entries
                .insert(from_idx, DirectoryEntry::File(f));
              return Err(Error::new(
                ErrorKind::Other,
                "Cannot rename to a directory",
              ));
            }
            _ => {
              f.name = to_file_name.to_string();
              to_parent.entries[pos] = DirectoryEntry::File(f);
            }
          },
          Err(insert_pos) => {
            f.name = to_file_name.to_string();
            to_parent
              .entries
              .insert(insert_pos, DirectoryEntry::File(f));
          }
        }
      }
      _ => {
        let from_parent = inner.find_directory_mut(from_parent_path, false)?;
        from_parent.entries.insert(from_idx, file_entry);
        return Err(Error::new(
          ErrorKind::Other,
          "Cannot rename directories or symlinks here",
        ));
      }
    }
    Ok(())
  }
}

impl FsSymlinkDir for InMemorySys {
  fn fs_symlink_dir(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    self.fs_symlink_file(original.as_ref(), link.as_ref())
  }
}

impl FsSymlinkFile for InMemorySys {
  fn fs_symlink_file(
    &self,
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
  ) -> std::io::Result<()> {
    let mut inner = self.0.write();
    let time = inner.time_now();
    let link = inner.to_absolute_path(link.as_ref());
    let parent = inner.find_directory_mut(link.parent().unwrap(), false)?;
    let file_name = link.file_name().unwrap().to_string_lossy();
    match parent
      .entries
      .binary_search_by(|e| e.name().cmp(&file_name))
    {
      Ok(overwrite_pos) => {
        match &parent.entries[overwrite_pos] {
          DirectoryEntry::Directory(directory) => {
            return Err(Error::new(
              ErrorKind::AlreadyExists,
              format!("Directory already exists: '{}'", directory.name),
            ));
          }
          DirectoryEntry::File(_) | DirectoryEntry::Symlink(_) => {
            // do nothing
          }
        }

        parent.entries[overwrite_pos] = DirectoryEntry::Symlink(Symlink {
          name: file_name.into_owned(),
          target: original.as_ref().to_path_buf(),
          inner: RwLock::new(SymlinkInner {
            created_time: time,
            modified_time: time,
          }),
        });
        Ok(())
      }
      Err(insert_index) => {
        parent.entries.insert(
          insert_index,
          DirectoryEntry::Symlink(Symlink {
            name: file_name.into_owned(),
            target: original.as_ref().to_path_buf(),
            inner: RwLock::new(SymlinkInner {
              created_time: time,
              modified_time: time,
            }),
          }),
        );
        Ok(())
      }
    }
  }
}

impl FsWrite for InMemorySys {
  fn fs_write(
    &self,
    path: impl AsRef<Path>,
    data: impl AsRef<[u8]>,
  ) -> std::io::Result<()> {
    let opts = OpenOptions {
      write: true,
      create: true,
      truncate: true,
      append: false,
      read: false,
      create_new: false,
    };
    let time_now = self.sys_time_now();
    let file = self.fs_open(path, &opts)?;
    let mut inner = file.inner.write();
    inner.data.clear();
    inner.data.extend_from_slice(data.as_ref());
    inner.modified_time = time_now;
    Ok(())
  }
}

// File System File

impl FsFileSetPermissions for InMemoryFile {
  fn fs_file_set_permissions(&mut self, mode: u32) -> std::io::Result<()> {
    let mut inner = self.inner.write();
    inner.mode = mode;
    Ok(())
  }
}

impl std::io::Write for InMemoryFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let time = self.sys.sys_time_now();
    let mut inner = self.inner.write();
    inner.data.splice(self.pos.., buf.as_ref().iter().cloned());
    inner.modified_time = time;
    self.pos += buf.as_ref().len();
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl std::io::Read for InMemoryFile {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    let inner = self.inner.read();
    let data = &inner.data[self.pos..];
    let len = std::cmp::min(data.len(), buf.len());
    buf[..len].copy_from_slice(&data[..len]);
    self.pos += len;
    Ok(len)
  }
}

// System

impl SystemTimeNow for InMemorySys {
  fn sys_time_now(&self) -> SystemTime {
    self.0.read().time_now()
  }
}

impl SystemRandom for InMemorySys {
  fn sys_random(&self, buf: &mut [u8]) -> std::io::Result<()> {
    match self.0.read().random_seed {
      Some(seed) => {
        // not the best, but good enough for now
        let mut state = seed;
        for byte in buf.iter_mut() {
          // simple linear congruential generator
          state = state.wrapping_mul(1664525).wrapping_add(1013904223);
          *byte = (state >> 24) as u8; // use the top 8 bits
        }
        Ok(())
      }
      None => RealSys.sys_random(buf),
    }
  }
}

impl ThreadSleep for InMemorySys {
  fn thread_sleep(&self, dur: std::time::Duration) {
    if self.0.read().thread_sleep_enabled {
      RealSys.thread_sleep(dur);
    }
  }
}

/// Normalize all intermediate components of the path (ie. remove "./" and "../" components).
/// Similar to `fs::canonicalize()` but doesn't resolve symlinks.
///
/// Taken from Cargo
/// <https://github.com/rust-lang/cargo/blob/af307a38c20a753ec60f0ad18be5abed3db3c9ac/src/cargo/util/paths.rs#L60-L85>
#[inline]
fn normalize_path(path: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let mut ret =
    if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
      components.next();
      PathBuf::from(c.as_os_str())
    } else {
      PathBuf::new()
    };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        ret.push(c);
      }
    }
  }
  ret
}

// most of these tests were lazily created with ChatGPT
#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;
  use std::path::Path;
  use std::time::Duration;
  use std::time::SystemTime;

  #[test]
  fn test_create_dir_all() {
    let sys = InMemorySys::default();
    let dir_path = Path::new("/rootDir/subDir");
    assert!(!sys.fs_exists(dir_path).unwrap());
    sys.fs_create_dir_all(dir_path).unwrap();
    assert!(sys.fs_exists(dir_path).unwrap());
    assert!(sys.fs_is_dir(dir_path).unwrap());
  }

  #[test]
  fn test_write_read_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/test.txt";
    sys.fs_write(file_path, b"Hello World!").unwrap();
    assert!(sys.fs_exists(file_path).unwrap());
    assert!(sys.fs_is_file(file_path).unwrap());

    let contents = sys.fs_read_to_string(file_path).unwrap();
    assert_eq!(&*contents, "Hello World!");
  }

  #[test]
  fn test_truncate_existing_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/data.bin";
    sys.fs_write(file_path, b"abcdef").unwrap();

    let opts = OpenOptions {
      write: true,
      truncate: true,
      ..Default::default()
    };
    let file = sys.fs_open(file_path, &opts).unwrap();
    // file is truncated at open, so should be empty
    let guard = file.inner.read();
    assert!(guard.data.is_empty());
  }

  #[test]
  fn test_rename_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let old_path = "/rootDir/old.txt";
    let new_path = "/rootDir/new.txt";
    sys.fs_write(old_path, b"some data").unwrap();

    sys.fs_rename(old_path, new_path).unwrap();
    assert!(!sys.fs_exists(old_path).unwrap());
    assert!(sys.fs_exists(new_path).unwrap());

    let data = sys.fs_read_to_string(new_path).unwrap();
    assert_eq!(&*data, "some data");
  }

  #[test]
  fn test_remove_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/remove_me.txt";
    sys.fs_write(file_path, b"Bye!").unwrap();
    assert!(sys.fs_exists(file_path).unwrap());

    sys.fs_remove_file(file_path).unwrap();
    assert!(!sys.fs_exists(file_path).unwrap());
  }

  #[test]
  fn test_modified_time() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/rootDir").unwrap();

    let file_path = "/rootDir/hello.txt";
    sys.fs_write(file_path, b"Hi!").unwrap();

    // First check if we can get a valid modified time
    let mod_result = sys.fs_modified(file_path).unwrap();
    assert!(mod_result.is_ok());
    let modified = mod_result.unwrap();

    // Since we can't easily freeze or manipulate real time,
    // we'll just assert it's no earlier than the current system time minus some buffer.
    let now = SystemTime::now();
    let duration = now.duration_since(modified);
    assert!(duration.is_ok());
  }

  #[test]
  fn test_exists_no_err() {
    let sys = InMemorySys::default();
    assert!(!sys.fs_exists_no_err("/does/not/exist"));
    sys.fs_create_dir_all("/exists").unwrap();
    assert!(sys.fs_exists_no_err("/exists"));
  }

  #[test]
  fn test_is_file_no_err() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file.txt", b"contents").unwrap();
    assert!(!sys.fs_is_file_no_err("/no/file"));
    assert!(!sys.fs_is_file_no_err("/dir"));
    assert!(sys.fs_is_file_no_err("/dir/file.txt"));
  }

  #[test]
  fn test_is_dir_no_err() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file.txt", b"contents").unwrap();
    assert!(!sys.fs_is_dir_no_err("/no/dir"));
    assert!(sys.fs_is_dir_no_err("/dir"));
    assert!(!sys.fs_is_dir_no_err("/dir/file.txt"));
  }

  #[test]
  fn test_file_permissions() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();

    let file_path = "/dir/perm_test.txt";
    sys.fs_write(file_path, b"Testing perms").unwrap();
    let mut file = sys.fs_open(file_path, &OpenOptions::read()).unwrap();
    file.fs_file_set_permissions(0o755).unwrap();

    let guard = file.inner.read();
    assert_eq!(guard.mode, 0o755);
  }

  #[test]
  fn test_file_append() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();

    let file_path = "/dir/append_test.txt";
    let mut opts = OpenOptions {
      write: true,
      create: true,
      ..Default::default()
    };
    // Not truncate
    sys.fs_open(file_path, &opts).unwrap(); // creates empty file
                                            // Now open again with append
    opts.append = true;
    let mut file = sys.fs_open(file_path, &opts).unwrap();
    // Should start at position 0 in the code, but let's test manually
    _ = file.write(b"Appending ").unwrap();
    _ = file.write(b"more data").unwrap();

    let contents = sys.fs_read_to_string(file_path).unwrap();
    assert_eq!(&*contents, "Appending more data");
  }

  #[test]
  fn test_create_dir_that_already_exists() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/already/exists").unwrap();
    let result = sys.fs_create_dir_all("/already/exists");
    assert!(
      result.is_ok(),
      "Creating a directory that already exists should succeed"
    );
  }

  #[test]
  fn test_remove_non_empty_directory_fails() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file.txt", b"data").unwrap();
    let result = sys.fs_remove_file("/dir");
    assert!(
            result.is_err(),
            "Removing a non-empty directory (treated as a directory, not a file) should fail"
        );
  }

  #[test]
  fn test_fs_canonicalize_relative_path() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/test/relative").unwrap();
    {
      let mut inner = sys.0.write();
      inner.cwd = PathBuf::from("/test");
    }
    let abs = sys.fs_canonicalize("relative").unwrap();
    assert_eq!(abs, PathBuf::from("/test/relative"));
  }

  #[test]
  fn test_fs_canonicalize_absolute_path() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/absolute").unwrap();
    let abs = sys.fs_canonicalize("/absolute").unwrap();
    assert_eq!(abs, PathBuf::from("/absolute"));
  }

  #[test]
  fn test_sys_random_no_seed() {
    let sys = InMemorySys::default();
    let mut buf1 = [0u8; 8];
    let mut buf2 = [0u8; 8];
    sys.sys_random(&mut buf1).unwrap();
    sys.sys_random(&mut buf2).unwrap();
    // There's no guarantee on exact values without a seed, but it should succeed
    assert_ne!(buf1, [0u8; 8]);
    assert_ne!(buf2, [0u8; 8]);
  }

  #[test]
  fn test_thread_sleep_no_op() {
    let sys = InMemorySys::default();
    sys.disable_thread_sleep();
    let start = SystemTime::now();
    sys.thread_sleep(Duration::from_secs(1));
    // Should be effectively no-op, so the elapsed time should be tiny
    let elapsed = start.elapsed().unwrap();
    assert!(
      elapsed < Duration::from_millis(100),
      "Sleep should be disabled"
    );
  }

  #[test]
  fn test_rename_file_to_existing_file() {
    let sys = InMemorySys::default();
    sys.fs_create_dir_all("/dir").unwrap();
    sys.fs_write("/dir/file1.txt", b"111").unwrap();
    sys.fs_write("/dir/file2.txt", b"222").unwrap();
    let result = sys.fs_rename("/dir/file1.txt", "/dir/file2.txt");
    assert!(result.is_ok() || result.is_err());
    let file1_exists = sys.fs_exists_no_err("/dir/file1.txt");
    let file2_exists = sys.fs_exists_no_err("/dir/file2.txt");
    assert!(!file1_exists && file2_exists);
  }

  #[test]
  fn test_fs_write_into_non_existent_subdir_fails() {
    let sys = InMemorySys::default();
    let result = sys.fs_write("/no-such-subdir/myfile.txt", b"content");
    assert!(
      result.is_err(),
      "Should fail because /no-such-subdir does not exist"
    );
  }
}
