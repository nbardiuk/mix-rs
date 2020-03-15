#[derive(Debug, PartialEq, Clone, Copy, Default)]
struct Byte(pub u8);

const BYTE_SIZE: u8 = 64;
const WORD_BYTES: u8 = 5;

impl Byte {
    fn new(b: u8) -> Byte {
        assert!(b < BYTE_SIZE, "Byte value should be smaller than 64");
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
    bytes: [Byte; WORD_BYTES as usize],
}

impl Word {
    fn new(sign: Sign, b0: u8, b1: u8, b2: u8, b3: u8, b4: u8) -> Self {
        Word {
            sign,
            bytes: [
                Byte::new(b0),
                Byte::new(b1),
                Byte::new(b2),
                Byte::new(b3),
                Byte::new(b4),
            ],
        }
    }

    fn slice(self, field_spec: FieldSpecification) -> Self {
        let sign = if field_spec.l > 0 {
            Sign::default()
        } else {
            self.sign
        };
        let mut bytes = [Byte::default(); WORD_BYTES as usize];
        let len = (field_spec.r - field_spec.l) + 1;
        for i in 0..len {
            if field_spec.l + i == 0 {
                continue;
            };
            let index_from = (field_spec.l + i - 1) as usize;
            let index_to = (WORD_BYTES + i - len) as usize;
            bytes[index_to] = self.bytes[index_from];
        }
        Self { sign, bytes }
    }
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
    x: Word,
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
        let b0 = Byte::new((address.abs() / BYTE_SIZE as i16) as u8);
        let b1 = Byte::new((address.abs() % BYTE_SIZE as i16) as u8);
        Self {
            sign,
            bytes: [b0, b1],
        }
    }
}
enum Operation {
    LDA,
    LDAN,
    LDX,
    LDXN,
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
}

fn lda(address: i16, index: Option<IndexNumber>, f: Option<FieldSpecification>) -> Instruction {
    Instruction::new(
        Operation::LDA,
        Address::new(address),
        index,
        f.unwrap_or_else(|| fields(0, WORD_BYTES)).into(),
    )
}

fn ldan(address: i16, index: Option<IndexNumber>, f: Option<FieldSpecification>) -> Instruction {
    Instruction::new(
        Operation::LDAN,
        Address::new(address),
        index,
        f.unwrap_or_else(|| fields(0, WORD_BYTES)).into(),
    )
}

fn ldx(address: i16, index: Option<IndexNumber>, f: Option<FieldSpecification>) -> Instruction {
    Instruction::new(
        Operation::LDX,
        Address::new(address),
        index,
        f.unwrap_or_else(|| fields(0, WORD_BYTES)).into(),
    )
}

fn ldxn(address: i16, index: Option<IndexNumber>, f: Option<FieldSpecification>) -> Instruction {
    Instruction::new(
        Operation::LDXN,
        Address::new(address),
        index,
        f.unwrap_or_else(|| fields(0, WORD_BYTES)).into(),
    )
}

fn fields(l: u8, r: u8) -> FieldSpecification {
    FieldSpecification::new(l, r)
}

impl Mix {
    fn contents(&self, address: Address) -> Word {
        let i = address.bytes[0].0 as usize * BYTE_SIZE as usize + address.bytes[1].0 as usize;
        self.memory[i]
    }

    fn load(&self, instruction: Instruction) -> Word {
        self.contents(instruction.address)
            .slice(FieldSpecification::from(instruction.modification))
    }

    fn exec(mut self, instruction: Instruction) -> Self {
        match instruction.operation {
            Operation::LDA => {
                self.a = self.load(instruction);
                self
            }
            Operation::LDAN => {
                self.a = self.load(instruction);
                self.a.sign = Sign::Minus;
                self
            }
            Operation::LDX => {
                self.x = self.load(instruction);
                self
            }
            Operation::LDXN => {
                self.x = self.load(instruction);
                self.x.sign = Sign::Minus;
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
    use Sign::*;

    #[test]
    fn field_byte_conversions() {
        for l in 0..WORD_BYTES + 1 {
            for r in l..WORD_BYTES + 1 {
                let field = fields(l, r);
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
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(lda(2000, None, None));

        assert_eq!(mix.a, Word::new(Minus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn lda_just_bytes() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(lda(2000, None, Some(fields(1, 5))));

        assert_eq!(mix.a, Word::new(Plus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn lda_second_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(lda(2000, None, Some(fields(3, 5))));

        assert_eq!(mix.a, Word::new(Plus, 0, 0, 3, 5, 4));
    }

    #[test]
    fn lda_first_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(lda(2000, None, Some(fields(0, 3))));

        assert_eq!(mix.a, Word::new(Minus, 0, 0, 1, 16, 3));
    }

    #[test]
    fn lda_single_byte() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(lda(2000, None, Some(fields(4, 4))));

        assert_eq!(mix.a, Word::new(Plus, 0, 0, 0, 0, 5));
    }

    #[test]
    fn lda_just_sign() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(lda(2000, None, Some(fields(0, 0))));

        assert_eq!(mix.a, Word::new(Minus, 0, 0, 0, 0, 0));
    }

    #[test]
    fn ldan_full() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldan(2000, None, None));

        assert_eq!(mix.a, Word::new(Minus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn ldan_just_bytes() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldan(2000, None, Some(fields(1, 5))));

        assert_eq!(mix.a, Word::new(Minus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn ldan_second_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldan(2000, None, Some(fields(3, 5))));

        assert_eq!(mix.a, Word::new(Minus, 0, 0, 3, 5, 4));
    }

    #[test]
    fn ldan_first_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldan(2000, None, Some(fields(0, 3))));

        assert_eq!(mix.a, Word::new(Minus, 0, 0, 1, 16, 3));
    }

    #[test]
    fn ldan_single_byte() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldan(2000, None, Some(fields(4, 4))));

        assert_eq!(mix.a, Word::new(Minus, 0, 0, 0, 0, 5));
    }

    #[test]
    fn ldan_just_sign() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldan(2000, None, Some(fields(0, 0))));

        assert_eq!(mix.a, Word::new(Minus, 0, 0, 0, 0, 0));
    }

    #[test]
    fn ldx_full() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldx(2000, None, None));

        assert_eq!(mix.x, Word::new(Minus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn ldx_just_bytes() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldx(2000, None, Some(fields(1, 5))));

        assert_eq!(mix.x, Word::new(Plus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn ldx_second_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldx(2000, None, Some(fields(3, 5))));

        assert_eq!(mix.x, Word::new(Plus, 0, 0, 3, 5, 4));
    }

    #[test]
    fn ldx_first_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldx(2000, None, Some(fields(0, 3))));

        assert_eq!(mix.x, Word::new(Minus, 0, 0, 1, 16, 3));
    }

    #[test]
    fn ldx_single_byte() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldx(2000, None, Some(fields(4, 4))));

        assert_eq!(mix.x, Word::new(Plus, 0, 0, 0, 0, 5));
    }

    #[test]
    fn ldx_just_sign() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldx(2000, None, Some(fields(0, 0))));

        assert_eq!(mix.x, Word::new(Minus, 0, 0, 0, 0, 0));
    }

    #[test]
    fn ldxn_full() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldxn(2000, None, None));

        assert_eq!(mix.x, Word::new(Minus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn ldxn_just_bytes() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldxn(2000, None, Some(fields(1, 5))));

        assert_eq!(mix.x, Word::new(Minus, 1, 16, 3, 5, 4));
    }

    #[test]
    fn ldxn_second_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldxn(2000, None, Some(fields(3, 5))));

        assert_eq!(mix.x, Word::new(Minus, 0, 0, 3, 5, 4));
    }

    #[test]
    fn ldxn_first_half() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldxn(2000, None, Some(fields(0, 3))));

        assert_eq!(mix.x, Word::new(Minus, 0, 0, 1, 16, 3));
    }

    #[test]
    fn ldxn_single_byte() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldxn(2000, None, Some(fields(4, 4))));

        assert_eq!(mix.x, Word::new(Minus, 0, 0, 0, 0, 5));
    }

    #[test]
    fn ldxn_just_sign() {
        let mut mix = Mix::default();
        mix.memory[2000] = Word::new(Minus, 1, 16, 3, 5, 4);

        let mix = mix.exec(ldxn(2000, None, Some(fields(0, 0))));

        assert_eq!(mix.x, Word::new(Minus, 0, 0, 0, 0, 0));
    }
}
