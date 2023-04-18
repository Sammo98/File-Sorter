use std::fs::{rename, create_dir, read_dir};
use std::path::{Path, PathBuf};
use notify::event::{EventKind, CreateKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config, Event};
use anyhow::{Result, anyhow};

const HOME:&str = std::env!("HOME");

pub struct FileWatcher;

impl FileWatcher {

    pub fn run() -> notify::Result<()> {

        // Set up the watch directory and the watcher
        let watch_dir = Path::new(HOME).join("Downloads");
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(watch_dir.as_ref(), RecursiveMode::NonRecursive)?;


        // Handle each event
        for res in rx {
            match res {
                Ok(event) => {
                    if let Err(e) = FileWatcher::handle_event(event) {
                        println!("Error handling event: {:?}", e)
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    Ok(())
    }

    fn handle_event(e:Event) -> Result<()> {
        match e.kind {
            EventKind::Create(CreateKind::File) => {
                for src in e.paths.iter() {
                    if let Err(_) = FileWatcher::handle_file(&src) {
                        println!("Error handling file {src:?}");
                    }

                }
            },
            _ => println!("Eventkind: {:?} ignored.", {e.kind})
        }
        Ok(())
    }

    pub fn backload() -> Result<()> {
        let directory = Path::new(HOME).join("Downloads");
        let paths = read_dir(directory)?;
        for path in paths {
            let path = path?.path();
            if path.is_file(){
                if let Err(_) = FileWatcher::handle_file(&path) {
                    println!("Error handling file {path:?}");
                }
            }

        }
        Ok(())
    }


    fn handle_file(handle:&PathBuf) -> Result<()>{
        let file_name = handle.file_name();
        let ext = handle.extension();
        match (file_name, ext) {
            (Some(file_name), Some(ext)) => {
                let destination = Path::new(HOME).join("Downloads").join(ext).join(file_name);
                FileWatcher::create_dir_if_not_exists(ext)?;
                FileWatcher::move_file(handle, destination)?;
                Ok(())
            },
            _ => Err(anyhow!("File Name or Extension does not exist for {:?}", handle))
        }

    }
    
    fn create_dir_if_not_exists<P:AsRef<Path>>(extension_type:P) -> Result<()> {
        let directory = Path::new(HOME).join("Downloads").join(extension_type);
        match directory.exists() {
            true => Ok(()),
            false => {
                Ok(create_dir(directory.clone())?)
            }
        }
    }

    fn move_file<S:AsRef<Path>, D:AsRef<Path>>(src:S, dest:D) -> Result<()> {
        let src = src.as_ref();
        if let true = src.is_file() {
            rename(src, dest)?;
        }
        Ok(())
    }

}
