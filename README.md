# File-Sorter

A solution to my laziness when it comes to cleaning up my downloads folder.

Filewatcher wrapped in CLAP as a CLI tool.

Usage:

`cargo run -- <directory_to_watch>`

Currently only support file creation events.

Note: Path is expanded to absolute path via HOME environment variable. For Windows users, absolute path should be supplied to the binary
