.PHONY: open target/x86_64-pc-windows-gnu/debug/rust-gameboy.exe

ROM := cpu_instrs.gb

open: target/x86_64-pc-windows-gnu/debug/rust-gameboy.exe
	$< --rom=$(ROM) $(if $(BIOS),--bios=$(BIOS),)

target/x86_64-pc-windows-gnu/debug/rust-gameboy.exe:
	cargo build --target x86_64-pc-windows-gnu

debug:
	RUST_BACKTRACE=1 RUST_LOG=info cargo run -- --rom $(ROM) --headless

