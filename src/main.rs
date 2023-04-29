pub mod watch;
pub mod utils;
use clap::Parser;
use watch::FileWatcher;
use utils::init_logger;

fn main() {
    init_logger();
    let mut fw = FileWatcher::parse();

    log::info!("Filewatcher Backload Commencing...");
    if let Err(e) = fw.backload() {
        println!("Error initiating filewatcher {}, exiting...", e.to_string());
        std::process::exit(1);
    }
    log::info!("Filewatcher Backload Complete!");

    log::info!("Filewatcher Beginning watch at {}", fw);
    if let Err(e) = fw.run() {
        println!("Error initiating filewatcher {}, exiting...", e.to_string());
        std::process::exit(1);
    }
}
