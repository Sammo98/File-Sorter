use anyhow::{anyhow, Result};
use clap::Parser;
use notify::event::{CreateKind, EventKind};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fmt::Display;
use std::fs::{create_dir, read_dir, rename};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Parser)]
pub struct FileWatcher {
    target_dir: String,
}

impl Display for FileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.target_dir)
    }
}

impl FileWatcher {
    pub fn backload(&mut self) -> Result<()> {
        // Expand path through HOME if not absolute
        let target = self.target_dir.clone();
        if let Err(e) = self.expand_path(&target) {
            log::error!("Error with provided path: {e:?}. Exiting...");
            std::process::exit(1);
        }

        // Read target directory and backload any missed target files
        let paths = read_dir(&self.target_dir)?;
        for path in paths {
            let path = path?.path();
            if path.is_file() {
                if let Err(e) = self.handle_file(&path) {
                    log::error!("Error handling file {path:?}: {e:?}. Skipping file ...");
                }
            }
        }
        Ok(())
    }

    pub fn run(&self) -> notify::Result<()> {
        // Set up the watch directory and the watcher
        let watch_dir = Path::new(&self.target_dir);
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(watch_dir.as_ref(), RecursiveMode::NonRecursive)?;
        log::info!("Filewatcher successfully initialised!");

        // Handle each event
        for res in rx {
            match res {
                Ok(event) => {
                    if let Err(e) = self.handle_event(event) {
                        log::error!("Error handling event: {e:?}. Skipping event ...")
                    }
                }
                Err(e) => log::error!("Unexpected error receiving event from channel: {:?}", e),
            }
        }
        Ok(())
    }

    fn handle_event(&self, event: Event) -> Result<()> {
        match event.kind {
            EventKind::Create(CreateKind::File) => {
                for src in event.paths.iter() {
                    log::info!("Handling file {src:?} ... ");
                    match self.handle_file(&src) {
                        Ok(_) => log::info!("File moved successfully!"),
                        Err(e) => log::error!("Error handling file {src:?}: {e:?}. Skipping ..."),
                    }
                }
            }
            _ => log::info!("Event {:?} encountered. Skipping ...", event.kind)
        }
        Ok(())
    }

    fn handle_file(&self, handle: &PathBuf) -> Result<()> {
        let file_name = handle.file_name();
        let ext = handle.extension();
        match (file_name, ext) {
            (Some(file_name), Some(ext)) => {
                let dir = Path::new(&self.target_dir).join(ext);
                self.create_dir_if_not_exists(&dir)?;
                let destination = dir.join(file_name);
                self.move_file(handle, destination)?;
                Ok(())
            }
            _ => Err(anyhow!(
                "File Name or Extension does not exist for {:?}",
                handle
            )),
        }
    }

    fn create_dir_if_not_exists(&self, dir: &PathBuf) -> Result<()> {
        match dir.exists() {
            true => Ok(()),
            false => {
                log::info!("Directory {dir:?} does not exist. Creating ..");
                Ok(create_dir(dir.clone())?)
            }
        }
    }

    fn move_file<S: AsRef<Path> + std::fmt::Debug, D: AsRef<Path> + std::fmt::Debug>(
        &self,
        src: S,
        dest: D,
    ) -> Result<()> {
        let src = src.as_ref();
        log::info!("Attempting to move file from {src:?} to {dest:?}");
        match src.is_file() {
            true => rename(src, dest)?,
            false => log::warn!("{src:?} has not been determined to be a file. Skipping ..."),
        }
        Ok(())
    }

    fn expand_path(&mut self, target_directory: &str) -> Result<()> {
        let target_dir_path = PathBuf::from_str(target_directory)?;

        match target_dir_path.is_absolute() {
            true => {}
            false => {
                let home = std::env::var("HOME")?;
                let mut home = PathBuf::from_str(&home)?;
                home.push(self.target_dir.clone());
                self.target_dir = home.display().to_string();
            }
        }

        Ok(())
    }

}

#[cfg(test)]
mod test {
    use super::FileWatcher;
    use anyhow::Result;
    use notify::{
        event::{CreateKind, EventKind},
        Event,
    };
    use std::{fs::File, path::PathBuf};

    fn create_fw_instance(target_dir: &str) -> FileWatcher {
        FileWatcher {
            target_dir: target_dir.into(),
        }
    }

    #[test]
    fn test_move_file() -> Result<()> {
        // Create source directory and file
        let dir = tempfile::tempdir()?;
        let target_dir = dir.path().join("some_file.txt");
        let _ = File::create(&target_dir)?;

        // Create destination directory
        let new_dir = tempfile::tempdir()?;
        let new_path = new_dir.path().join("some_file.txt");

        // Create FileWatcher instance
        let fw = create_fw_instance("bar");

        // Assert source file does not exist in target directory
        assert_eq!(new_path.exists(), false);

        // Move file to target directory and assert it path exists after move
        let _ = fw.move_file(&target_dir, &new_path)?;
        assert_eq!(new_path.exists(), true);
        Ok(())
    }

    #[test]
    fn test_create_dir_if_not_exists() -> Result<()> {
        // Create a temp directory
        let dir = tempfile::tempdir()?;
        let dir = dir.path();

        // Create filewatcher instance watching the temp directory
        let fw = create_fw_instance(&dir.display().to_string());

        // Assert txt directory does not exist as a subdirectory of the filewatcher target
        let non_existent_dir = dir.join("txt");
        assert_eq!(non_existent_dir.exists(), false);

        // Run function and assert that the directory has been created
        let _ = fw.create_dir_if_not_exists(&non_existent_dir);
        assert_eq!(non_existent_dir.exists(), true);

        Ok(())
    }

    #[test]
    fn test_handle_event_executes_on_create_file_event() {
        let mock_event = Event {
            kind: EventKind::Create(CreateKind::File),
            ..Default::default()
        };

        let fw = create_fw_instance("foo");
        let result = fw.handle_event(mock_event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_file_moves_valid_file() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let src_dir = temp_dir.path();
        let src_path = src_dir.join("some_file.txt");
        let _ = File::create(&src_path)?;

        let dest_path = src_dir.join("txt").join("some_file.txt");

        let fw = create_fw_instance(&src_dir.display().to_string());

        assert_eq!(src_path.exists(), true);
        assert_eq!(dest_path.exists(), false);
        let result = fw.handle_file(&src_path);
        assert!(result.is_ok());
        assert_eq!(src_path.exists(), false);
        assert_eq!(dest_path.exists(), true);
        Ok(())
    }

    #[test]
    fn test_handle_file_errors_on_non_existent_extension() {
        let stupid_file = PathBuf::new().join("a silly file");
        let fw = create_fw_instance("foo");
        let result = fw.handle_file(&stupid_file);
        assert!(result.is_err());
    }
}
