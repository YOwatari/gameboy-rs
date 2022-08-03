.PHONY: open main/target/x86_64-pc-windows-gnu/debug/gameboy-rs.exe

#BIOS := DMG_ROM.bin
ROM := cpu_instrs.gb

open: main/target/x86_64-pc-windows-gnu/debug/gameboy-rs.exe
	$< --rom=$(ROM) $(if $(BIOS),--bios=$(BIOS),)

main/target/x86_64-pc-windows-gnu/debug/gameboy-rs.exe:
	cargo build --target x86_64-pc-windows-gnu

debug:
	RUST_BACKTRACE=1 RUST_LOG=info cargo run -- --rom $(ROM) $(if $(BIOS),--bios=$(BIOS),) --headless
