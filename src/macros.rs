// macro_export => makes the macros defined here to be available in the root of the project
#[macro_export]
macro_rules! loop_through_files_in_dir {
  ($directory_name:expr, $identifier:ident ) => {{
    let mut counter: u32 = 0;
    for entry in fs::read_dir(($directory_name)).unwrap() {
      let dir_entry = entry.unwrap();
      let filename = dir_entry.file_name().into_string().unwrap();
      $identifier.insert(&filename);
      counter += 1;
    }
    counter
  }};

  ($directory_name:expr, $filename:ident, $block:block) => {{
    let mut counter: u32 = 0;
    for entry in fs::read_dir(($directory_name)).unwrap() {
      let dir_entry = entry.unwrap();
      let $filename = dir_entry.file_name().into_string().unwrap();
      $block
      counter += 1;
    }
    counter
  }};
}

#[macro_export]
macro_rules! handle_err {
  ($err:expr, $message:expr) => {
    if $err.is_err() {
      println!("{}. Error: {}", $message, $err.unwrap_err());
      panic!($message);
    }
  };

  ($err:expr, $message:expr, $custom_execution:block) => {
    if $err.is_err() {
      println!("{}. Error: {}", $message, $err.unwrap_err());
      $custom_execution
    }
  };
}
