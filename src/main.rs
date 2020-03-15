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
                let field = FieldSpecification::new(l, r);
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

    fn loading(
        operation: Operation,
        address: i16,
        index: Option<IndexNumber>,
        f: Option<FieldSpecification>,
    ) -> Instruction {
        Instruction::new(
            operation,
            Address::new(address),
            index,
            f.unwrap_or_else(|| FieldSpecification::new(0, WORD_BYTES))
                .into(),
        )
    }

    fn fields(l: u8, r: u8) -> Option<FieldSpecification> {
        Some(FieldSpecification::new(l, r))
    }

    #[test]
    fn lda_all() {
        test_lda(
            "should load full word",
            Word::new(Minus, 1, 16, 3, 5, 4),
            None,
            Word::new(Minus, 1, 16, 3, 5, 4),
        );
        test_lda(
            "should load just bytes",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(1, 5),
            Word::new(Plus, 1, 16, 3, 5, 4),
        );
        test_lda(
            "should load only second half",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(3, 5),
            Word::new(Plus, 0, 0, 3, 5, 4),
        );
        test_lda(
            "should load only first half",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(0, 3),
            Word::new(Minus, 0, 0, 1, 16, 3),
        );
        test_lda(
            "should load single byte",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(4, 4),
            Word::new(Plus, 0, 0, 0, 0, 5),
        );
        test_lda(
            "should load just sign",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(0, 0),
            Word::new(Minus, 0, 0, 0, 0, 0),
        );
        fn test_lda(message: &str, before: Word, f: Option<FieldSpecification>, expected: Word) {
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(loading(Operation::LDA, 2000, None, f));

            assert_eq!(mix.a, expected, "{}", message);
        }
    }

    #[test]
    fn ldan_all() {
        test_ldan(
            "should load negative word",
            Word::new(Plus, 1, 16, 3, 5, 4),
            None,
            Word::new(Minus, 1, 16, 3, 5, 4),
        );
        test_ldan(
            "should load just bytes",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(1, 5),
            Word::new(Minus, 1, 16, 3, 5, 4),
        );
        test_ldan(
            "should load only second half",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(3, 5),
            Word::new(Minus, 0, 0, 3, 5, 4),
        );
        test_ldan(
            "should load only first half",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(0, 3),
            Word::new(Minus, 0, 0, 1, 16, 3),
        );
        test_ldan(
            "should load single byte",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(4, 4),
            Word::new(Minus, 0, 0, 0, 0, 5),
        );
        test_ldan(
            "should not load just sign",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(0, 0),
            Word::new(Minus, 0, 0, 0, 0, 0),
        );
        fn test_ldan(message: &str, before: Word, f: Option<FieldSpecification>, expected: Word) {
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(loading(Operation::LDAN, 2000, None, f));

            assert_eq!(mix.a, expected, "{}", message);
        }
    }

    #[test]
    fn ldx_all() {
        test_ldx(
            "should load full word",
            Word::new(Minus, 1, 16, 3, 5, 4),
            None,
            Word::new(Minus, 1, 16, 3, 5, 4),
        );
        test_ldx(
            "should load just bytes",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(1, 5),
            Word::new(Plus, 1, 16, 3, 5, 4),
        );
        test_ldx(
            "should load only second half",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(3, 5),
            Word::new(Plus, 0, 0, 3, 5, 4),
        );
        test_ldx(
            "should load only first half",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(0, 3),
            Word::new(Minus, 0, 0, 1, 16, 3),
        );
        test_ldx(
            "should load single byte",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(4, 4),
            Word::new(Plus, 0, 0, 0, 0, 5),
        );
        test_ldx(
            "should load just sign",
            Word::new(Minus, 1, 16, 3, 5, 4),
            fields(0, 0),
            Word::new(Minus, 0, 0, 0, 0, 0),
        );
        fn test_ldx(message: &str, before: Word, f: Option<FieldSpecification>, expected: Word) {
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(loading(Operation::LDX, 2000, None, f));

            assert_eq!(mix.x, expected, "{}", message);
        }
    }

    #[test]
    fn ldxn_all() {
        test_ldxn(
            "should load negative word",
            Word::new(Plus, 1, 16, 3, 5, 4),
            None,
            Word::new(Minus, 1, 16, 3, 5, 4),
        );
        test_ldxn(
            "should load just bytes",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(1, 5),
            Word::new(Minus, 1, 16, 3, 5, 4),
        );
        test_ldxn(
            "should load only second half",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(3, 5),
            Word::new(Minus, 0, 0, 3, 5, 4),
        );
        test_ldxn(
            "should load only first half",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(0, 3),
            Word::new(Minus, 0, 0, 1, 16, 3),
        );
        test_ldxn(
            "should load single byte",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(4, 4),
            Word::new(Minus, 0, 0, 0, 0, 5),
        );
        test_ldxn(
            "should not load just sign",
            Word::new(Plus, 1, 16, 3, 5, 4),
            fields(0, 0),
            Word::new(Minus, 0, 0, 0, 0, 0),
        );
        fn test_ldxn(message: &str, before: Word, f: Option<FieldSpecification>, expected: Word) {
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(loading(Operation::LDXN, 2000, None, f));

            assert_eq!(mix.x, expected, "{}", message);
        }
    }
}
