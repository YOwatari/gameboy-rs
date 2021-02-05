.PHONY: open target/x86_64-pc-windows-gnu/debug/rust-gameboy.exe

open: target/x86_64-pc-windows-gnu/debug/rust-gameboy.exe
	$< $(BIOS) $(ROM)

target/x86_64-pc-windows-gnu/debug/rust-gameboy.exe:
	cargo build --target x86_64-pc-windows-gnu
