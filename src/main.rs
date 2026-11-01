use std::println;

const FLAG_NEGATIVE: u8 = 0b0000_1000; // Bit 3 (N): Set if the result has bit 7 set (negative)
const FLAG_ZERO: u8 = 0b0001_0000; // Bit 4 (Z): Set if the result is zero
const FLAG_CARRY: u8 = 0b0010_0000; // Bit 5 (C): Set if the result overflowed 8 bits (carry out)
const FLAG_HALT: u8 = 0b1000_0000; // Bit 7 (H): Set if the CPU should halt
const RAM_SIZE: usize = 4_096;
const NUMBER_OF_REGS: usize = 4;

#[derive(Debug)]
struct CPU {
    ram: [u8; RAM_SIZE],
    accumulator: u8,
    registers: [u8; NUMBER_OF_REGS],
    pc: u16,
    _sp: u16,
    flags: u8,
}

fn overflowing_add(a: u8, b: u8) -> (u8, bool) {
    let res = a as u16 + b as u16;
    (res as u8, res > u8::MAX as u16)
}

impl CPU {
    fn new() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            accumulator: 0,
            registers: [0; NUMBER_OF_REGS],
            pc: 0,
            _sp: RAM_SIZE as u16 - 1,
            flags: 0,
        }
    }

    fn decode(&mut self) -> Instruction {
        let opcode = self.ram[self.pc as usize];
        self.pc += 1; // Move past the opcode

        match opcode {
            1 => {
                let reg = self.ram[self.pc as usize];
                self.pc += 1;
                let value = self.ram[self.pc as usize];
                self.pc += 1; //next instruction
                Instruction::LDI {
                    reg: reg,
                    value: value,
                }
            }
            2 => {
                let reg = self.ram[self.pc as usize];
                self.pc += 1; //next instruction
                Instruction::ADD { reg: reg }
            }
            3 => {
                let reg = self.ram[self.pc as usize];
                self.pc += 1; //next instruction
                Instruction::PRINT { reg: reg }
            }
            4 => Instruction::HALT,
            _ => Instruction::NOP,
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::LDI { reg, value } => {
                self.registers[reg as usize] = value;
            }
            Instruction::ADD { reg } => {
                self.flags &= !(FLAG_CARRY | FLAG_ZERO | FLAG_NEGATIVE);
                let overflow;
                (self.accumulator, overflow) =
                    overflowing_add(self.accumulator, self.registers[reg as usize]);
                if self.accumulator == 0 {
                    self.flags |= FLAG_ZERO;
                }
                if self.accumulator & 0b1000_0000 != 0 {
                    self.flags |= FLAG_NEGATIVE;
                }
                if overflow {
                    self.flags |= FLAG_CARRY;
                }
            }
            Instruction::PRINT { reg } => {
                println!("R{reg} = [{:#X}]", self.registers[reg as usize]);
            }
            Instruction::HALT => {
                self.flags |= FLAG_HALT;
            }
            Instruction::NOP => {}
        }
    }
}

#[derive(Debug)]
enum Instruction {
    LDI { reg: u8, value: u8 }, // Load immediate value into register
    ADD { reg: u8 },            // Add register to accumulator
    PRINT { reg: u8 },          // Print register value
    HALT,                       // Stop execution
    NOP,
}

fn main() {
    let mut cpu = CPU::new();
    println!("CPU init successful..");

    let program = [
        1, 0, 100, // LDI R0, 15
        1, 1, 200, // LDI R1, 20
        2, 0, // ADD R0
        2, 1, // ADD R1
        3, 0, // PRINT R0
        4, // HALT
    ];

    cpu.ram[..program.len()].copy_from_slice(&program);

    loop {
        if cpu.flags & FLAG_HALT != 0 {
            break;
        }
        let instruction = cpu.decode();
        cpu.execute(instruction);
    }

    println!("Final Accumulator value: {}", cpu.accumulator);
    println!("Flags: {:08b}", cpu.flags);
}
