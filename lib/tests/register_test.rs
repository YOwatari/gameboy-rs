#[cfg(test)]
extern crate rstest;
extern crate speculate;

use register::Flags;
use register::Register;
use register::Registers16;
use rstest::*;
use speculate::speculate;

use gameboy_rs_lib::cpu::register;

speculate! {
    describe "レジスタ操作" {
        #[fixture(b=0b_10101111, c=0b_11001100)]
        fn fixture(b: u8, c: u8) -> Registers {
            let mut register = Registers::new();
            register.b = b;
            register.c = c;
            return register;
        }

        describe "16bit読み取り" {
            #[rstest(reg, expected,
                case(Registers16::BC, 0b_10101111_11001100),
            )]
            fn read_wordは対象16bitレジスタを読み取れる(fixture: Registers, reg: Registers16, expected: u16) {
                assert_eq!(expected, fixture.read_word(reg));
            }
        }
    }
}
