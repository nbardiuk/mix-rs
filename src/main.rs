const BYTE_SIZE: u8 = 64;
const WORD_BYTES: u8 = 5;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
struct Byte(pub u8);
impl Byte {
    fn new(b: u8) -> Byte {
        debug_assert!(b < BYTE_SIZE, "Byte value should be smaller than 64");
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
impl Sign {
    fn opposite(&self) -> Self {
        match self {
            Sign::Plus => Sign::Minus,
            Sign::Minus => Sign::Plus,
        }
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
struct Word {
    sign: Sign,
    bytes: [Byte; WORD_BYTES as usize],
}

impl Word {
    fn new(sign: Sign, b0: u8, b1: u8, b2: u8, b3: u8, b4: u8) -> Self {
        Self {
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

    fn merge(self, word: Word, field_spec: FieldSpecification) -> Self {
        let mut result = self;
        if field_spec.l == 0 {
            result.sign = word.sign;
        };

        let len = (field_spec.r - field_spec.l) + 1;
        for i in 0..len {
            if field_spec.l + i == 0 {
                continue;
            };
            let index_from = (WORD_BYTES + i - len) as usize;
            let index_to = (field_spec.l + i - 1) as usize;
            result.bytes[index_to] = word.bytes[index_from];
        }
        result
    }

    fn negate(self) -> Self {
        let mut word = self;
        word.sign = self.sign.opposite();
        word
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
struct Index {
    sign: Sign,
    bytes: [Byte; 2],
}
impl Index {
    fn new(sign: Sign, b0: u8, b1: u8) -> Self {
        Self {
            sign,
            bytes: [Byte::new(b0), Byte::new(b1)],
        }
    }
}
impl From<Word> for Index {
    fn from(word: Word) -> Self {
        Self {
            sign: word.sign,
            bytes: [word.bytes[3], word.bytes[4]],
        }
    }
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
    LDX,
    LD1,
    LD2,
    LD3,
    LD4,
    LD5,
    LD6,
    LDAN,
    LDXN,
    LD1N,
    LD2N,
    LD3N,
    LD4N,
    LD5N,
    LD6N,
    STA,
    STX,
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
    fn contents(&self, address: &Address) -> Word {
        let i = address.bytes[0].0 as usize * BYTE_SIZE as usize + address.bytes[1].0 as usize;
        self.memory[i]
    }

    fn save_contents(&mut self, address: &Address, word: Word) {
        let i = address.bytes[0].0 as usize * BYTE_SIZE as usize + address.bytes[1].0 as usize;
        self.memory[i] = word;
    }

    fn load(&self, instruction: Instruction) -> Word {
        self.contents(&instruction.address)
            .slice(FieldSpecification::from(instruction.modification))
    }

    fn store(&mut self, word: Word, instruction: Instruction) {
        let cell = self.contents(&instruction.address);
        self.save_contents(
            &instruction.address,
            cell.merge(word, FieldSpecification::from(instruction.modification)),
        );
    }

    fn exec(mut self, instruction: Instruction) -> Self {
        match instruction.operation {
            Operation::LDA => {
                self.a = self.load(instruction);
            }
            Operation::LDX => {
                self.x = self.load(instruction);
            }
            Operation::LD1 => {
                self.i1 = Index::from(self.load(instruction));
            }
            Operation::LD2 => {
                self.i2 = Index::from(self.load(instruction));
            }
            Operation::LD3 => {
                self.i3 = Index::from(self.load(instruction));
            }
            Operation::LD4 => {
                self.i4 = Index::from(self.load(instruction));
            }
            Operation::LD5 => {
                self.i5 = Index::from(self.load(instruction));
            }
            Operation::LD6 => {
                self.i6 = Index::from(self.load(instruction));
            }
            Operation::LDAN => {
                self.a = self.load(instruction).negate();
            }
            Operation::LDXN => {
                self.x = self.load(instruction).negate();
            }
            Operation::LD1N => {
                self.i1 = Index::from(self.load(instruction).negate());
            }
            Operation::LD2N => {
                self.i2 = Index::from(self.load(instruction).negate());
            }
            Operation::LD3N => {
                self.i3 = Index::from(self.load(instruction).negate());
            }
            Operation::LD4N => {
                self.i4 = Index::from(self.load(instruction).negate());
            }
            Operation::LD5N => {
                self.i5 = Index::from(self.load(instruction).negate());
            }
            Operation::LD6N => {
                self.i6 = Index::from(self.load(instruction).negate());
            }
            Operation::STA => {
                self.store(self.a, instruction);
            }
            Operation::STX => {
                self.store(self.x, instruction);
            }
        };
        self
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod spec {
    use super::*;
    use Operation::*;
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

    fn instruction(
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
    fn lda() {
        assert(None, Word::new(Minus, 1, 16, 3, 5, 4));
        assert(fields(1, 5), Word::new(Plus, 1, 16, 3, 5, 4));
        assert(fields(3, 5), Word::new(Plus, 0, 0, 3, 5, 4));
        assert(fields(0, 3), Word::new(Minus, 0, 0, 1, 16, 3));
        assert(fields(4, 4), Word::new(Plus, 0, 0, 0, 0, 5));
        assert(fields(0, 0), Word::new(Minus, 0, 0, 0, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Word) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDA, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.a, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ldan() {
        assert(None, Word::new(Plus, 1, 16, 3, 5, 4));
        assert(fields(1, 5), Word::new(Minus, 1, 16, 3, 5, 4));
        assert(fields(3, 5), Word::new(Minus, 0, 0, 3, 5, 4));
        assert(fields(0, 3), Word::new(Plus, 0, 0, 1, 16, 3));
        assert(fields(4, 4), Word::new(Minus, 0, 0, 0, 0, 5));
        assert(fields(0, 0), Word::new(Plus, 0, 0, 0, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Word) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDAN, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.a, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ldx() {
        assert(None, Word::new(Minus, 1, 16, 3, 5, 4));
        assert(fields(1, 5), Word::new(Plus, 1, 16, 3, 5, 4));
        assert(fields(3, 5), Word::new(Plus, 0, 0, 3, 5, 4));
        assert(fields(0, 3), Word::new(Minus, 0, 0, 1, 16, 3));
        assert(fields(4, 4), Word::new(Plus, 0, 0, 0, 0, 5));
        assert(fields(0, 0), Word::new(Minus, 0, 0, 0, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Word) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDX, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.x, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ldxn() {
        assert(None, Word::new(Plus, 1, 16, 3, 5, 4));
        assert(fields(1, 5), Word::new(Minus, 1, 16, 3, 5, 4));
        assert(fields(3, 5), Word::new(Minus, 0, 0, 3, 5, 4));
        assert(fields(0, 3), Word::new(Plus, 0, 0, 1, 16, 3));
        assert(fields(4, 4), Word::new(Minus, 0, 0, 0, 0, 5));
        assert(fields(0, 0), Word::new(Plus, 0, 0, 0, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Word) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDXN, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.x, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld1() {
        assert(fields(4, 5), Index::new(Plus, 5, 4));
        assert(fields(0, 2), Index::new(Minus, 1, 16));
        assert(fields(4, 4), Index::new(Plus, 0, 5));
        assert(fields(0, 0), Index::new(Minus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD1, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i1, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld2() {
        assert(fields(4, 5), Index::new(Plus, 5, 4));
        assert(fields(0, 2), Index::new(Minus, 1, 16));
        assert(fields(4, 4), Index::new(Plus, 0, 5));
        assert(fields(0, 0), Index::new(Minus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD2, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i2, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld3() {
        assert(fields(4, 5), Index::new(Plus, 5, 4));
        assert(fields(0, 2), Index::new(Minus, 1, 16));
        assert(fields(4, 4), Index::new(Plus, 0, 5));
        assert(fields(0, 0), Index::new(Minus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD3, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i3, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld4() {
        assert(fields(4, 5), Index::new(Plus, 5, 4));
        assert(fields(0, 2), Index::new(Minus, 1, 16));
        assert(fields(4, 4), Index::new(Plus, 0, 5));
        assert(fields(0, 0), Index::new(Minus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD4, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i4, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld5() {
        assert(fields(4, 5), Index::new(Plus, 5, 4));
        assert(fields(0, 2), Index::new(Minus, 1, 16));
        assert(fields(4, 4), Index::new(Plus, 0, 5));
        assert(fields(0, 0), Index::new(Minus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD5, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i5, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld6() {
        assert(fields(4, 5), Index::new(Plus, 5, 4));
        assert(fields(0, 2), Index::new(Minus, 1, 16));
        assert(fields(4, 4), Index::new(Plus, 0, 5));
        assert(fields(0, 0), Index::new(Minus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD6, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i6, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld1n() {
        assert(fields(4, 5), Index::new(Minus, 5, 4));
        assert(fields(0, 2), Index::new(Plus, 1, 16));
        assert(fields(4, 4), Index::new(Minus, 0, 5));
        assert(fields(0, 0), Index::new(Plus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD1N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i1, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld2n() {
        assert(fields(4, 5), Index::new(Minus, 5, 4));
        assert(fields(0, 2), Index::new(Plus, 1, 16));
        assert(fields(4, 4), Index::new(Minus, 0, 5));
        assert(fields(0, 0), Index::new(Plus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD2N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i2, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld3n() {
        assert(fields(4, 5), Index::new(Minus, 5, 4));
        assert(fields(0, 2), Index::new(Plus, 1, 16));
        assert(fields(4, 4), Index::new(Minus, 0, 5));
        assert(fields(0, 0), Index::new(Plus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD3N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i3, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld4n() {
        assert(fields(4, 5), Index::new(Minus, 5, 4));
        assert(fields(0, 2), Index::new(Plus, 1, 16));
        assert(fields(4, 4), Index::new(Minus, 0, 5));
        assert(fields(0, 0), Index::new(Plus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD4N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i4, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld5n() {
        assert(fields(4, 5), Index::new(Minus, 5, 4));
        assert(fields(0, 2), Index::new(Plus, 1, 16));
        assert(fields(4, 4), Index::new(Minus, 0, 5));
        assert(fields(0, 0), Index::new(Plus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD5N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i5, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ld6n() {
        assert(fields(4, 5), Index::new(Minus, 5, 4));
        assert(fields(0, 2), Index::new(Plus, 1, 16));
        assert(fields(4, 4), Index::new(Minus, 0, 5));
        assert(fields(0, 0), Index::new(Plus, 0, 0));
        fn assert(f: Option<FieldSpecification>, expected: Index) {
            let before = Word::new(Minus, 1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD6N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i6, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn sta() {
        assert(None, Word::new(Plus, 6, 7, 8, 9, 0));
        assert(fields(1, 5), Word::new(Minus, 6, 7, 8, 9, 0));
        assert(fields(5, 5), Word::new(Minus, 1, 2, 3, 4, 0));
        assert(fields(2, 2), Word::new(Minus, 1, 0, 3, 4, 5));
        assert(fields(2, 3), Word::new(Minus, 1, 9, 0, 4, 5));
        assert(fields(0, 1), Word::new(Plus, 0, 2, 3, 4, 5));
        fn assert(f: Option<FieldSpecification>, expected: Word) {
            let before = Word::new(Plus, 6, 7, 8, 9, 0);
            let mut mix = Mix::default();
            mix.memory[2000] = Word::new(Minus, 1, 2, 3, 4, 5);
            mix.a = before;

            let mix = mix.exec(instruction(STA, 2000, None, f.clone()));

            assert_eq!(mix.a, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn stx() {
        assert(None, Word::new(Plus, 6, 7, 8, 9, 0));
        assert(fields(1, 5), Word::new(Minus, 6, 7, 8, 9, 0));
        assert(fields(5, 5), Word::new(Minus, 1, 2, 3, 4, 0));
        assert(fields(2, 2), Word::new(Minus, 1, 0, 3, 4, 5));
        assert(fields(2, 3), Word::new(Minus, 1, 9, 0, 4, 5));
        assert(fields(0, 1), Word::new(Plus, 0, 2, 3, 4, 5));
        fn assert(f: Option<FieldSpecification>, expected: Word) {
            let before = Word::new(Plus, 6, 7, 8, 9, 0);
            let mut mix = Mix::default();
            mix.memory[2000] = Word::new(Minus, 1, 2, 3, 4, 5);
            mix.x = before;

            let mix = mix.exec(instruction(STX, 2000, None, f.clone()));

            assert_eq!(mix.x, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }
}
