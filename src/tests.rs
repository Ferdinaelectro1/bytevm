use std::assert_eq;

use super::*;

// Opcode table (byte 0 of each instruction)
//
// | Opcode | Mnemonic | Operands     | Size    | Effect                                  |
// |--------|----------|--------------|---------|-----------------------------------------|
// | 1      | LDI      | reg, value   | 3 bytes | registers[reg] = value                  |
// | 2      | ADD      | reg          | 2 bytes | acc += registers[reg]  (sets Z/N/C)     |
// | 3      | PRINT    | reg          | 2 bytes | print registers[reg]                    |
// | 4      | HALT     | -            | 1 byte  | set FLAG_HALT                           |
// | 5      | SUB      | reg          | 2 bytes | acc -= registers[reg]  (sets Z/N/C)     |
// | other  | NOP      | -            | 1 byte  | does nothing                            |

// Load the program and run it
fn run(program: &[u8]) -> CPU {
    let mut cpu = CPU::new();
    cpu.ram[..program.len()].copy_from_slice(program);
    while cpu.flags & FLAG_HALT == 0 {
        let instruction = cpu.decode();
        cpu.execute(instruction);
    }
    cpu
}

#[test]
fn overflowing_add_wraps_and_sets_carry() {
    assert_eq!(overflowing_add(10, 23), (33, false));
    assert_eq!(overflowing_add(200, 100), (44, true));
    assert_eq!(overflowing_add(0, 0), (0, false));
    assert_eq!(overflowing_add(255, 0), (255, false));
    assert_eq!(overflowing_add(255, 1), (0, true));
}

#[test]
fn overflowing_sub_wraps_and_sets_carry() {
    assert_eq!(overflowing_sub(10, 5), (5, false));
    assert_eq!(overflowing_sub(5, 10), (251, true));
    assert_eq!(overflowing_sub(0, 0), (0, false));
    assert_eq!(overflowing_sub(0, 1), (255, true));
    assert_eq!(overflowing_sub(255, 255), (0, false));
}

#[test]
fn move_immediate_into_register() {
    // LDI R0,32 / HALT
    let cpu = run(&[1, 0, 32, 4]);
    assert_eq!(cpu.registers[0], 32);
}

#[test]
fn sub_borrow_sets_carry_and_negative() {
    // LDI R0,5 / LDI R1,10 / ADD R0 / SUB R1 / HALT
    let cpu = run(&[1, 0, 5, 1, 1, 10, 2, 0, 5, 1, 4]);

    assert_eq!(cpu.accumulator, 251);
    assert_eq!(
        cpu.flags & (FLAG_ZERO | FLAG_NEGATIVE | FLAG_CARRY),
        FLAG_CARRY | FLAG_NEGATIVE
    );
}

#[test]
fn sub_zero_result_sets_zero_only() {
    // LDI R0,5 / LDI R1,5 / ADD R0 / SUB R1 / HALT
    let cpu = run(&[1, 0, 5, 1, 1, 5, 2, 0, 5, 1, 4]);

    assert_eq!(cpu.accumulator, 0);
    assert_eq!(
        cpu.flags & (FLAG_ZERO | FLAG_NEGATIVE | FLAG_CARRY),
        FLAG_ZERO
    );
}

#[test]
fn sub_normal_result_sets_no_flags() {
    // LDI R0,20 / LDI R1,5 / ADD R0 / SUB R1 / HALT
    let cpu = run(&[1, 0, 20, 1, 1, 5, 2, 0, 5, 1, 4]);

    assert_eq!(cpu.accumulator, 15);
    assert_eq!(cpu.flags & (FLAG_ZERO | FLAG_NEGATIVE | FLAG_CARRY), 0);
}

#[test]
fn add_overflow_to_zero_sets_zero_and_carry() {
    // LDI R0,255 / LDI R1,1 / ADD R0 / ADD R1 / HALT
    let cpu = run(&[1, 0, 255, 1, 1, 1, 2, 0, 2, 1, 4]);

    assert_eq!(cpu.accumulator, 0);
    assert_eq!(
        cpu.flags & (FLAG_ZERO | FLAG_NEGATIVE | FLAG_CARRY),
        FLAG_ZERO | FLAG_CARRY
    );
}

#[test]
fn unknown_opcode_is_nop_and_advances_one_byte() {
    // 99 (unknown) / HALT
    let cpu = run(&[99, 4]);

    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.accumulator, 0);
}
