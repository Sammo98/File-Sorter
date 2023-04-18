pub mod watch;

fn main() {
    if let Err(e) = watch::FileWatcher::backload() {
        println!("Error initiating filewatcher {}, exiting...", e.to_string());
        std::process::exit(1);
    }
    if let Err(e) = watch::FileWatcher::run(){
        println!("Error initiating filewatcher {}, exiting...", e.to_string());
        std::process::exit(1);
    }
}

