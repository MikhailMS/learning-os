# RADIUS OS

Following (this great tutorial)[https://os.phil-opp.com/freestanding-rust-binary] in an effort to learn a bit more about OS and how to build one from scratch


## Pre-reqs:
1. Install package to create bootable images `cargo install bootimage`
2. Install QEMU: for macOS run `brew install qemu`


## Build commands
0. (For completeness only)  `cargo rustc -- -C link-args="-e __start -static -nostartfiles"` to build on macOS
1. To create bootable image `cargo bootimage`
2. To run image             `qemu-system-x86_64 -drive format=raw,file=target/x86_64-radius_os/debug/bootimage-radius_os.bin` or `cargo run`


## Notes
