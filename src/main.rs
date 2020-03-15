#[derive(Debug, PartialEq, Clone, Copy, Default)]
struct Byte(pub u8);

impl Byte {
    fn new(b: u8) -> Byte {
        assert!(b < 64, "Byte value should be smaller than 64");
        Byte(b)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Sign {
    Plus,
    Minus,
}
impl Default for Sign {
    fn default() -> Self {
        Sign::Plus
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
struct Word {
    sign: Sign,
    bytes: [Byte; 5],
}
impl Word {
    fn slice(self, field_spec: FieldSpecification) -> Self {
        let sign = if field_spec.l > 0 {
            Sign::default()
        } else {
            self.sign
        };
        let mut bytes = [Byte::default(); 5];
        let len = (field_spec.r - field_spec.l) + 1;
        for i in 0..len {
            if field_spec.l + i == 0 {
                continue;
            };
            let index_from = (field_spec.l + i - 1) as usize;
            let index_to = (5 + i - len) as usize;
            bytes[index_to] = self.bytes[index_from];
        }
        Self { sign, bytes }
    }
}

// Registers
#[derive(Default)]
struct Extension {
    sign: Sign,
    bytes: [Byte; 5],
}
#[derive(Default)]
struct Index {
    sign: Sign,
    bytes: [Byte; 2],
}
#[derive(Default)]
struct Jump {
    //assume sign is always Plus
    bytes: [Byte; 2],
}

enum Toggle {
    On,
    Off,
}
impl Default for Toggle {
    fn default() -> Self {
        Toggle::Off
    }
}
enum Comparison {
    Less,
    Equal,
    Greater,
}
impl Default for Comparison {
    fn default() -> Self {
        Comparison::Equal
    }
}

struct Mix {
    a: Word,
    x: Extension,
    i1: Index,
    i2: Index,
    i3: Index,
    i4: Index,
    i5: Index,
    i6: Index,
    j: Jump,
    overflow: Toggle,
    comparison_indicator: Comparison,
    memory: [Word; 4000],
}
impl Default for Mix {
    fn default() -> Self {
        Mix {
            a: Default::default(),
            x: Default::default(),
            i1: Default::default(),
            i2: Default::default(),
            i3: Default::default(),
            i4: Default::default(),
            i5: Default::default(),
            i6: Default::default(),
            j: Default::default(),
            overflow: Default::default(),
            comparison_indicator: Default::default(),
            memory: [Default::default(); 4000],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct FieldSpecification {
    l: u8,
    r: u8,
}
impl From<Byte> for FieldSpecification {
    fn from(b: Byte) -> Self {
        let l = b.0 / 8;
        let r = b.0 % 8;
        Self { l, r }
    }
}
impl Into<Byte> for FieldSpecification {
    fn into(self) -> Byte {
        Byte::new(self.l * 8 + self.r)
    }
}
impl Default for FieldSpecification {
    fn default() -> Self {
        Self { l: 0, r: 5 }
    }
}
impl FieldSpecification {
    fn new(l: u8, r: u8) -> Self {
        Self { l, r }
    }
}

enum IndexNumber {
    I1,
    I2,
    I3,
    I4,
    I5,
    I6,
}
struct Address {
    sign: Sign,
    bytes: [Byte; 2],
}
impl Address {
    fn new(address: i16) -> Self {
        let sign = if address >= 0 {
            Sign::Plus
        } else {
            Sign::Minus
        };
        let b0 = Byte::new((address.abs() / 64) as u8);
        let b1 = Byte::new((address.abs() % 64) as u8);
        Self {
            sign,
            bytes: [b0, b1],
        }
    }
}
enum Operation {
    LDA,
}
struct Instruction {
    operation: Operation,
    address: Address,
    index: Option<IndexNumber>,
    modification: Byte,
}
impl Instruction {
    fn new(
        operation: Operation,
        address: Address,
        index: Option<IndexNumber>,
        modification: Byte,
    ) -> Self {
        Instruction {
            operation,
            address,
            index,
            modification,
        }
    }

    fn lda(address: Address, index: Option<IndexNumber>, f: Option<FieldSpecification>) -> Self {
        Self::new(Operation::LDA, address, index, f.unwrap_or_default().into())
    }
}

impl Mix {
    fn contents(&self, address: Address) -> Word {
        let i = address.bytes[0].0 as usize * 64 + address.bytes[1].0 as usize;
        self.memory[i]
    }
    fn exec(mut self, instruction: Instruction) -> Self {
        match instruction.operation {
            LDA => {
                self.a = self
                    .contents(instruction.address)
                    .slice(FieldSpecification::from(instruction.modification));
                self
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod spec {
    use super::*;

    #[test]
    fn field_byte_conversions() {
        for l in 0..5 {
            for r in l..6 {
                let field = FieldSpecification { l, r };
                let byte: Byte = field.clone().into();
                assert_eq!(
                    field,
                    FieldSpecification::from(byte.clone()),
                    "round trip conversion of field specification through byte should be idempotent"
                );
                assert_eq!(
                    byte.clone(),
                    FieldSpecification::from(byte.clone()).into(),
                    "round trip conversion of byte through field specification through should be idempotent"
                );
            }
        }
    }

    #[test]
    fn lda_full() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word {
            sign: Sign::Minus,
            bytes: [
                Byte::new(1),
                Byte::new(16),
                Byte::new(3),
                Byte::new(5),
                Byte::new(4),
            ],
        };
        assert_eq!(
            mix.exec(Instruction::lda(Address::new(2000), None, None)).a,
            Word {
                sign: Sign::Minus,
                bytes: [
                    Byte::new(1),
                    Byte::new(16),
                    Byte::new(3),
                    Byte::new(5),
                    Byte::new(4),
                ],
            }
        );
    }

    #[test]
    fn lda_just_bytes() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word {
            sign: Sign::Minus,
            bytes: [
                Byte::new(1),
                Byte::new(16),
                Byte::new(3),
                Byte::new(5),
                Byte::new(4),
            ],
        };
        assert_eq!(
            mix.exec(Instruction::lda(
                Address::new(2000),
                None,
                Some(FieldSpecification::new(1, 5))
            ))
            .a,
            Word {
                sign: Sign::Plus,
                bytes: [
                    Byte::new(1),
                    Byte::new(16),
                    Byte::new(3),
                    Byte::new(5),
                    Byte::new(4),
                ],
            }
        );
    }

    #[test]
    fn lda_second_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word {
            sign: Sign::Minus,
            bytes: [
                Byte::new(1),
                Byte::new(16),
                Byte::new(3),
                Byte::new(5),
                Byte::new(4),
            ],
        };
        assert_eq!(
            mix.exec(Instruction::lda(
                Address::new(2000),
                None,
                Some(FieldSpecification::new(3, 5))
            ))
            .a,
            Word {
                sign: Sign::Plus,
                bytes: [
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(3),
                    Byte::new(5),
                    Byte::new(4),
                ],
            }
        );
    }

    #[test]
    fn lda_first_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word {
            sign: Sign::Minus,
            bytes: [
                Byte::new(1),
                Byte::new(16),
                Byte::new(3),
                Byte::new(5),
                Byte::new(4),
            ],
        };
        assert_eq!(
            mix.exec(Instruction::lda(
                Address::new(2000),
                None,
                Some(FieldSpecification::new(0, 3))
            ))
            .a,
            Word {
                sign: Sign::Minus,
                bytes: [
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(1),
                    Byte::new(16),
                    Byte::new(3),
                ],
            }
        );
    }

    #[test]
    fn lda_single_byte() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word {
            sign: Sign::Minus,
            bytes: [
                Byte::new(1),
                Byte::new(16),
                Byte::new(3),
                Byte::new(5),
                Byte::new(4),
            ],
        };
        assert_eq!(
            mix.exec(Instruction::lda(
                Address::new(2000),
                None,
                Some(FieldSpecification::new(4, 4))
            ))
            .a,
            Word {
                sign: Sign::Plus,
                bytes: [
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(5),
                ],
            }
        );
    }
    #[test]
    fn lda_just_sign() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word {
            sign: Sign::Minus,
            bytes: [
                Byte::new(1),
                Byte::new(16),
                Byte::new(3),
                Byte::new(5),
                Byte::new(4),
            ],
        };
        assert_eq!(
            mix.exec(Instruction::lda(
                Address::new(2000),
                None,
                Some(FieldSpecification::new(0, 0))
            ))
            .a,
            Word {
                sign: Sign::Minus,
                bytes: [
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(0),
                    Byte::new(0),
                ],
            }
        );
    }
}
