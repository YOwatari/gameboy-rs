.PHONY: open target/x86_64-pc-windows-gnu/debug/gameboy-rs.exe

#BIOS := DMG_ROM.bin
ROM := cpu_instrs.gb

open: target/x86_64-pc-windows-gnu/debug/gameboy-rs.exe
	$< --rom=$(ROM) $(if $(BIOS),--bios=$(BIOS),)

target/x86_64-pc-windows-gnu/debug/gameboy-rs.exe:
	cargo build --target x86_64-pc-windows-gnu -p gameboy-rs
