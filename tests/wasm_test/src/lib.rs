use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use sys_traits::impls::RealSys;
use sys_traits::CreateDirOptions;
use sys_traits::EnvCacheDir;
use sys_traits::EnvCurrentDir;
use sys_traits::EnvHomeDir;
use sys_traits::EnvSetCurrentDir;
use sys_traits::EnvSetUmask;
use sys_traits::EnvSetVar;
use sys_traits::EnvTempDir;
use sys_traits::EnvUmask;
use sys_traits::EnvVar;
use sys_traits::FileType;
use sys_traits::FsCanonicalize;
use sys_traits::FsChown;
use sys_traits::FsCopy;
use sys_traits::FsCreateDir;
use sys_traits::FsCreateDirAll;
use sys_traits::FsDirEntry;
use sys_traits::FsFileIsTerminal;
use sys_traits::FsFileLock;
use sys_traits::FsFileLockMode;
use sys_traits::FsFileSetLen;
use sys_traits::FsHardLink;
use sys_traits::FsMetadata;
use sys_traits::FsMetadataValue;
use sys_traits::FsOpen;
use sys_traits::FsRead;
use sys_traits::FsReadDir;
use sys_traits::FsReadLink;
use sys_traits::FsRemoveDir;
use sys_traits::FsRemoveDirAll;
use sys_traits::FsRemoveFile;
use sys_traits::FsSetFileTimes;
use sys_traits::FsSetPermissions;
use sys_traits::FsSetSymlinkFileTimes;
use sys_traits::FsSymlinkChown;
use sys_traits::FsSymlinkFile;
use sys_traits::FsWrite;
use sys_traits::OpenOptions;
use sys_traits::SystemRandom;
use sys_traits::SystemTimeNow;
use sys_traits::ThreadSleep;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console, js_name = error)]
  fn log(s: &str);
}

#[wasm_bindgen]
pub fn run_tests(is_windows: bool) -> Result<(), JsValue> {
  console_error_panic_hook::set_once();
  run(is_windows).map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

fn run(is_windows: bool) -> std::io::Result<()> {
  let sys = RealSys::default();

  // create dir all
  let _ = sys.fs_remove_dir_all("tests/wasm_test/temp");
  sys.fs_create_dir_all("tests/wasm_test/temp/sub")?;

  // create dir
  let err = sys
    .fs_create_dir(
      "tests/wasm_test/temp/sub/sub/sub",
      &CreateDirOptions::default(),
    )
    .unwrap_err();
  assert_eq!(err.kind(), ErrorKind::NotFound); // because not recursive
  let mut options = CreateDirOptions::default();
  options.recursive().mode(0o755);
  sys.fs_create_dir("tests/wasm_test/temp/sub/sub/sub", &options)?;

  // random
  let mut data = [0; 10];
  sys.sys_random(&mut data)?;
  assert!(data.iter().any(|&x| x != 0));

  // env
  let cwd = sys.env_current_dir()?;
  sys.env_set_current_dir(cwd.join("tests/wasm_test"))?;
  let test_dir = sys.env_current_dir()?;
  assert!(test_dir.ends_with("wasm_test"));

  sys.env_set_var("SYS_TRAITS_TEST", "Value");
  assert_eq!(sys.env_var("SYS_TRAITS_TEST").unwrap(), "Value");

  // file system
  assert!(sys.fs_exists_no_err(test_dir.join("src")));
  assert!(!sys.fs_is_file_no_err(test_dir.join("src")));
  assert!(sys.fs_is_dir_no_err(test_dir.join("src")));
  assert!(sys.fs_is_file_no_err(test_dir.join("Cargo.toml")));
  assert!(!sys.fs_is_dir_no_err(test_dir.join("Cargo.toml")));

  let temp_dir = test_dir.join("temp");
  sys.env_set_current_dir(&temp_dir)?;

  let start_time = sys.sys_time_now();
  sys.fs_write("file.txt", "hello")?;
  assert_eq!(sys.fs_read_to_string("file.txt")?, "hello");
  assert_eq!(sys.fs_read("file.txt")?.into_owned(), b"hello");
  let modified_time = sys.fs_metadata("file.txt")?.modified()?;
  let end_time = sys.sys_time_now();
  assert!(start_time <= end_time);
  // some file systems have less precision than the system clock,
  // so just check that it's within a second
  assert!(
    modified_time
      .duration_since(start_time)
      .unwrap_or_else(|_| start_time.duration_since(modified_time).unwrap())
      < Duration::from_secs(1)
  );

  sys.fs_symlink_file("file.txt", "link.txt")?;
  assert!(sys.fs_is_symlink_no_err("link.txt"));
  assert_eq!(sys.fs_read_to_string("link.txt")?, "hello");
  assert_eq!(sys.fs_canonicalize("link.txt")?, temp_dir.join("file.txt"));
  assert_eq!(sys.fs_read_link("link.txt")?, PathBuf::from("file.txt"));
  sys.fs_remove_file("link.txt")?;
  assert!(!sys.fs_exists_no_err("link.txt"));
  assert!(sys.fs_exists_no_err("file.txt"));

  // open an existing file with create_new
  {
    let mut open_options = OpenOptions::default();
    open_options.create_new = true;
    open_options.create = true;
    open_options.write = true;
    let err = sys.fs_open("file.txt", &open_options).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
  }

  // open existing file with truncate off
  {
    let mut open_options = OpenOptions::default();
    open_options.write = true;
    open_options.truncate = false;
    open_options.append = false;
    let mut file = sys.fs_open("file.txt", &open_options)?;
    file.write(b"t")?;
  }
  // now open for reading
  {
    let mut file = sys.fs_open("file.txt", &OpenOptions::new_read())?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    assert_eq!(text, "tello");
  }

  // now append with truncate off
  {
    let mut open_options = OpenOptions::default();
    open_options.write = true;
    open_options.truncate = false;
    open_options.append = true;
    let mut file = sys.fs_open("file.txt", &open_options)?;
    file.write(b" there")?;
  }

  // now with append off and seeking
  {
    let mut open_options = OpenOptions::default();
    open_options.write = true;
    open_options.truncate = false;
    open_options.append = false;
    let mut file = sys.fs_open("file.txt", &open_options)?;
    assert_eq!(file.seek(std::io::SeekFrom::End(0))?, 11);
    assert_eq!(file.write(b"?")?, 1);
    assert_eq!(file.seek(std::io::SeekFrom::Current(-1))?, 11);
    assert_eq!(file.write(b"!")?, 1);
    assert_eq!(file.seek(std::io::SeekFrom::Start(0))?, 0);
    assert_eq!(file.write(b"H")?, 1);
  }

  assert_eq!(sys.fs_read_to_string("file.txt")?, "Hello there!");

  // system
  let start_time = sys.sys_time_now();
  sys.thread_sleep(Duration::from_millis(20));
  let end_time = sys.sys_time_now();
  assert!(
    end_time.duration_since(start_time).unwrap() >= Duration::from_millis(20)
  );

  let err = sys.fs_read_to_string("non_existent.txt").unwrap_err();
  assert_eq!(err.kind(), std::io::ErrorKind::NotFound);

  // just ensure these don't panic
  assert!(sys.env_home_dir().is_some());
  assert!(sys.env_cache_dir().is_some());
  assert!(sys.env_temp_dir().is_ok());

  let entries = sys.fs_read_dir(".")?;
  let mut entries = entries.into_iter().map(|e| e.unwrap()).collect::<Vec<_>>();
  entries.sort_by_key(|e| e.file_name().to_string_lossy().to_string());
  assert_eq!(entries.len(), 2);
  assert_eq!(entries[0].file_name().to_string_lossy(), "file.txt");
  assert_eq!(entries[0].file_type().unwrap(), FileType::File);
  assert_eq!(entries[0].path().to_path_buf(), PathBuf::from("./file.txt")); // because . was provided
  assert_eq!(entries[0].metadata().unwrap().file_type(), FileType::File);
  assert_eq!(entries[1].file_name().to_string_lossy(), "sub");
  assert_eq!(entries[1].file_type().unwrap(), FileType::Dir);

  // try writing and reading hard links
  sys.fs_hard_link("file.txt", "hardlink.txt")?;
  assert_eq!(sys.fs_read_to_string("hardlink.txt")?, "Hello there!");

  // umask
  if is_windows {
    let err = sys.env_umask().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Unsupported);
    let err = sys.env_set_umask(0o777).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Unsupported);
  } else {
    let original = sys.env_umask().unwrap();
    let value = sys.env_set_umask(0o777).unwrap();
    assert_eq!(value, original);
    let value = sys.env_set_umask(original).unwrap();
    assert_eq!(value, 0o0777);
  }

  // permissions
  if is_windows {
    let err = sys.fs_set_permissions("file.txt", 0o0777).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Unsupported);
  } else {
    sys.fs_set_permissions("file.txt", 0o0777).unwrap();
  }

  // copy file
  sys.fs_copy("file.txt", "copy.txt").unwrap();
  assert_eq!(sys.fs_read_to_string("copy.txt").unwrap(), "Hello there!");

  // open and set length below
  {
    let mut options = OpenOptions::new_write();
    options.truncate = false;
    let mut fs_file = sys.fs_open("copy.txt", &options)?;
    fs_file.fs_file_set_len(5)?;
    drop(fs_file);
    assert_eq!(sys.fs_read_to_string("copy.txt").unwrap(), "Hello");
  }
  // open and set length above
  {
    let mut options = OpenOptions::new_write();
    options.truncate = false;
    let mut fs_file = sys.fs_open("copy.txt", &options)?;
    fs_file.fs_file_set_len(10)?;
    drop(fs_file);
    assert_eq!(
      sys.fs_read_to_string("copy.txt").unwrap(),
      format!("Hello{}", "\0".repeat(5))
    );
  }
  // metadata
  {
    let metadata = sys.fs_metadata("copy.txt")?;
    assert_eq!(metadata.len(), 10);
    assert_eq!(metadata.file_type(), FileType::File);
    assert!(metadata.accessed().is_ok());
    assert!(metadata.changed().is_ok());
    assert!(metadata.created().is_ok());
    assert!(metadata.modified().is_ok());
    assert!(metadata.dev().is_ok());
    assert!(metadata.mode().is_ok());

    if is_windows {
      assert_eq!(metadata.ino().unwrap_err().kind(), ErrorKind::Unsupported);
      assert_eq!(metadata.nlink().unwrap_err().kind(), ErrorKind::Unsupported);
      assert_eq!(metadata.uid().unwrap_err().kind(), ErrorKind::Unsupported);
      assert_eq!(metadata.gid().unwrap_err().kind(), ErrorKind::Unsupported);
      assert_eq!(metadata.rdev().unwrap_err().kind(), ErrorKind::Unsupported);
      assert_eq!(
        metadata.blksize().unwrap_err().kind(),
        ErrorKind::Unsupported
      );
      assert_eq!(
        metadata.blocks().unwrap_err().kind(),
        ErrorKind::Unsupported
      );
      assert_eq!(
        metadata.is_block_device().unwrap_err().kind(),
        ErrorKind::Unsupported
      );
      assert_eq!(
        metadata.is_char_device().unwrap_err().kind(),
        ErrorKind::Unsupported
      );
      assert_eq!(
        metadata.is_fifo().unwrap_err().kind(),
        ErrorKind::Unsupported
      );
      assert_eq!(
        metadata.is_socket().unwrap_err().kind(),
        ErrorKind::Unsupported
      );
    } else {
      assert!(metadata.ino().is_ok());
      assert!(metadata.nlink().is_ok());
      assert!(metadata.uid().is_ok());
      assert!(metadata.gid().is_ok());
      assert!(metadata.rdev().is_ok());
      assert!(metadata.blksize().is_ok());
      assert!(metadata.blocks().is_ok());
      assert!(metadata.is_block_device().is_ok());
      assert!(metadata.is_char_device().is_ok());
      assert!(metadata.is_fifo().is_ok());
      assert!(metadata.is_socket().is_ok());
    }
    assert_eq!(
      metadata.file_attributes().unwrap_err().kind(),
      ErrorKind::Unsupported
    );
  }

  // system time
  {
    let accessed_time = SystemTime::UNIX_EPOCH
      .checked_add(Duration::from_secs(100))
      .unwrap();
    let modified_time = SystemTime::UNIX_EPOCH
      .checked_add(Duration::from_secs(10))
      .unwrap();
    sys.fs_set_file_times("copy.txt", accessed_time, modified_time)?;
    let metadata = sys.fs_metadata("copy.txt")?;
    assert_eq!(metadata.accessed()?, accessed_time);
    assert_eq!(metadata.modified()?, modified_time);
    assert_eq!(
      sys
        .fs_set_symlink_file_times("copy.txt", accessed_time, modified_time)
        .unwrap_err()
        .kind(),
      ErrorKind::Unsupported
    );
  }

  // chown
  if is_windows {
    assert_eq!(
      sys.fs_chown("copy.txt", None, None).unwrap_err().kind(),
      ErrorKind::Unsupported
    );
  } else {
    assert!(sys.fs_chown("copy.txt", None, None).is_ok());
  }
  assert_eq!(
    sys
      .fs_symlink_chown("copy.txt", None, None)
      .unwrap_err()
      .kind(),
    ErrorKind::Unsupported
  );

  // remove dir
  {
    sys.fs_create_dir_all("my_dir/to_remove")?;
    assert!(sys.fs_remove_dir("my_dir").is_err());
    sys.fs_remove_dir("my_dir/to_remove")?;
    sys.fs_remove_dir("my_dir")?;
    assert!(!sys.fs_exists_no_err("my_dir"));
  }

  // is-terminal
  {
    let file = sys.fs_open("copy.txt", &OpenOptions::new_read())?;
    assert!(!file.fs_file_is_terminal());
    if !is_windows {
      let file = sys.fs_open("/dev/tty6", &OpenOptions::new_write())?;
      assert!(file.fs_file_is_terminal());
    }
  }

  // file lock
  {
    let file = sys.fs_open("copy.txt", &OpenOptions::new_read())?;
    file.fs_file_lock(FsFileLockMode::Shared)?;
    file.fs_file_unlock()?;
    file.fs_file_lock(FsFileLockMode::Exclusive)?;
    file.fs_file_unlock()?;
    assert_eq!(
      file
        .fs_file_try_lock(FsFileLockMode::Shared)
        .unwrap_err()
        .kind(),
      ErrorKind::Unsupported
    );
  }

  log("Success!");

  Ok(())
}
