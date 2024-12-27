use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use sys_traits::impls::RealSys;
use sys_traits::EnvCacheDir;
use sys_traits::EnvCurrentDir;
use sys_traits::EnvHomeDir;
use sys_traits::EnvSetCurrentDir;
use sys_traits::EnvSetVar;
use sys_traits::EnvTempDir;
use sys_traits::EnvVar;
use sys_traits::FileType;
use sys_traits::FsCanonicalize;
use sys_traits::FsCreateDirAll;
use sys_traits::FsDirEntry;
use sys_traits::FsHardLink;
use sys_traits::FsMetadata;
use sys_traits::FsMetadataValue;
use sys_traits::FsOpen;
use sys_traits::FsRead;
use sys_traits::FsReadDir;
use sys_traits::FsRemoveDirAll;
use sys_traits::FsRemoveFile;
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
pub fn run_tests() -> Result<(), JsValue> {
  console_error_panic_hook::set_once();
  run().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

fn run() -> std::io::Result<()> {
  let sys = RealSys::default();

  let _ = sys.fs_remove_dir_all("tests/wasm_test/temp");
  sys.fs_create_dir_all("tests/wasm_test/temp/sub")?;

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
  sys.fs_remove_file("link.txt")?;
  assert!(!sys.fs_exists_no_err("link.txt"));
  assert!(sys.fs_exists_no_err("file.txt"));

  // open an existing file with create_new
  let err = sys
    .fs_open(
      "file.txt",
      &OpenOptions {
        create_new: true,
        create: true,
        write: true,
        ..Default::default()
      },
    )
    .unwrap_err();
  assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);

  // open existing file with truncate off
  {
    let mut file = sys.fs_open(
      "file.txt",
      &OpenOptions {
        write: true,
        truncate: false,
        append: false,
        ..Default::default()
      },
    )?;
    file.write(b"t")?;
  }
  // now open for reading
  {
    let mut file = sys.fs_open("file.txt", &OpenOptions::read())?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    assert_eq!(text, "tello");
  }

  // now append with truncate off
  {
    let mut file = sys.fs_open(
      "file.txt",
      &OpenOptions {
        write: true,
        truncate: false,
        append: true,
        ..Default::default()
      },
    )?;
    file.write(b" there")?;
  }

  // now with append off and seeking
  {
    let mut file = sys.fs_open(
      "file.txt",
      &OpenOptions {
        write: true,
        truncate: false,
        append: false,
        ..Default::default()
      },
    )?;
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

  log("Success!");

  Ok(())
}
