#[derive(Debug, PartialEq, Clone)]
struct Byte(pub u8);

impl Byte {
    fn new(b: u8) -> Byte {
        assert!(b < 64, "Byte should be less than 64");
        Byte(b)
    }
}

enum Sign {
    Plus,
    Minus,
}

struct Word {
    sign: Sign,
    bytes: [Byte; 5],
}

// Registers
struct Accumulator {
    sign: Sign,
    bytes: [Byte; 5],
}
struct Extension {
    sign: Sign,
    bytes: [Byte; 5],
}
struct Index {
    sign: Sign,
    bytes: [Byte; 2],
}
struct Jump {
    //assume sign is always Plus
    bytes: [Byte; 2],
}

enum Toggle {
    On,
    Off,
}
enum Comparison {
    Less,
    Equal,
    Greater,
}

struct Mix {
    a: Accumulator,
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
struct Instruction {
    operation_code: Byte,
    address: Address,
    index: Option<IndexNumber>,
    modification: Byte,
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
}
