const BYTE: u8 = 64;
const WORD_BYTES: u8 = 5;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Default)]
struct Byte(pub u8);
impl Byte {
    fn new(b: u8) -> Byte {
        debug_assert!(b < BYTE, "Byte value should be smaller than {}", BYTE);
        Byte(b)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Sign {
    Minus,
    Plus,
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

#[derive(Debug, PartialEq, PartialOrd, Default, Copy, Clone)]
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

    fn slice(self, m: Modification) -> Self {
        match m {
            Modification::Field { l, r } => {
                let sign = if l > 0 { Sign::default() } else { self.sign };
                let mut bytes = [Byte::default(); WORD_BYTES as usize];
                let len = (r - l) + 1;
                for i in 0..len {
                    if l + i == 0 {
                        continue;
                    };
                    let index_from = (l + i - 1) as usize;
                    let index_to = (WORD_BYTES + i - len) as usize;
                    bytes[index_to] = self.bytes[index_from];
                }
                Self { sign, bytes }
            }
        }
    }

    fn merge(self, word: Word, m: Modification) -> Self {
        match m {
            Modification::Field { l, r } => {
                let mut result = self;
                if l == 0 {
                    result.sign = word.sign;
                };

                let len = (r - l) + 1;
                for i in 0..len {
                    if l + i == 0 {
                        continue;
                    };
                    let index_from = (WORD_BYTES + i - len) as usize;
                    let index_to = (l + i - 1) as usize;
                    result.bytes[index_to] = word.bytes[index_from];
                }
                result
            }
        }
    }

    fn overflowing_add(self, other: Self) -> (Self, bool) {
        let mut a = self;
        let mut b = other;

        let mut carry = 0;
        if a.sign == b.sign {
            for i in (0..WORD_BYTES as usize).rev() {
                let sum = a.bytes[i].0 + b.bytes[i].0 + carry;
                a.bytes[i] = Byte::new(sum % BYTE);
                carry = sum / BYTE;
            }
        } else {
            if a < -b {
                std::mem::swap(&mut a, &mut b);
            }
            for i in (0..WORD_BYTES as usize).rev() {
                let mut s = a.bytes[i].0 as i16 - b.bytes[i].0 as i16 - carry as i16;
                if s < 0 {
                    s += BYTE as i16;
                    carry = 1;
                } else {
                    carry = 0;
                }
                a.bytes[i] = Byte::new(s.abs() as u8 % BYTE);
            }
        }

        (a, 0 < carry)
    }
}

impl std::ops::Neg for Word {
    type Output = Self;
    fn neg(self) -> Self {
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
impl Into<Word> for Index {
    fn into(self) -> Word {
        Word {
            sign: self.sign,
            bytes: [
                Byte::default(),
                Byte::default(),
                Byte::default(),
                self.bytes[0],
                self.bytes[1],
            ],
        }
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
struct Jump {
    bytes: [Byte; 2],
}
impl Jump {
    fn new(b0: u8, b1: u8) -> Self {
        Self {
            bytes: [Byte::new(b0), Byte::new(b1)],
        }
    }
}
impl From<Word> for Jump {
    fn from(word: Word) -> Self {
        Self {
            bytes: [word.bytes[3], word.bytes[4]],
        }
    }
}
impl Into<Word> for Jump {
    fn into(self) -> Word {
        Word {
            sign: Sign::Plus,
            bytes: [
                Byte::default(),
                Byte::default(),
                Byte::default(),
                self.bytes[0],
                self.bytes[1],
            ],
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Toggle {
    On,
    Off,
}
impl Default for Toggle {
    fn default() -> Self {
        Toggle::Off
    }
}
impl From<bool> for Toggle {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
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
enum Modification {
    Field { l: u8, r: u8 },
}
impl From<Byte> for Modification {
    fn from(b: Byte) -> Self {
        let l = b.0 / 8;
        let r = b.0 % 8;
        Modification::field(l, r)
    }
}
impl Into<Byte> for Modification {
    fn into(self) -> Byte {
        match self {
            Modification::Field { l, r } => Byte::new(l * 8 + r),
        }
    }
}
impl Modification {
    fn field(l: u8, r: u8) -> Self {
        Modification::Field { l, r }
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
        let b0 = Byte::new((address.abs() / BYTE as i16) as u8);
        let b1 = Byte::new((address.abs() % BYTE as i16) as u8);
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
    ST1,
    ST2,
    ST3,
    ST4,
    ST5,
    ST6,
    STJ,
    STZ,
    ADD,
}
impl Operation {
    fn default_modification(self) -> Modification {
        match self {
            Operation::STJ => Modification::field(0, 2),
            _ => Modification::field(0, 5),
        }
    }
}
struct Instruction {
    operation: Operation,
    address: Address,
    index: Option<IndexNumber>,
    modification: Option<Modification>,
}
impl Instruction {
    fn new(
        operation: Operation,
        address: Address,
        index: Option<IndexNumber>,
        modification: Option<Modification>,
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
        let i = address.bytes[0].0 as usize * BYTE as usize + address.bytes[1].0 as usize;
        self.memory[i]
    }

    fn save_contents(&mut self, address: &Address, word: Word) {
        let i = address.bytes[0].0 as usize * BYTE as usize + address.bytes[1].0 as usize;
        self.memory[i] = word;
    }

    fn load(&self, instruction: Instruction) -> Word {
        let operation = instruction.operation;
        let field = instruction
            .modification
            .unwrap_or_else(|| operation.default_modification());
        self.contents(&instruction.address).slice(field)
    }

    fn store(&mut self, word: Word, instruction: Instruction) {
        let cell = self.contents(&instruction.address);
        let operation = instruction.operation;
        let field = instruction
            .modification
            .unwrap_or_else(|| operation.default_modification());
        self.save_contents(&instruction.address, cell.merge(word, field));
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
                self.a = -self.load(instruction);
            }
            Operation::LDXN => {
                self.x = -self.load(instruction);
            }
            Operation::LD1N => {
                self.i1 = Index::from(-self.load(instruction));
            }
            Operation::LD2N => {
                self.i2 = Index::from(-self.load(instruction));
            }
            Operation::LD3N => {
                self.i3 = Index::from(-self.load(instruction));
            }
            Operation::LD4N => {
                self.i4 = Index::from(-self.load(instruction));
            }
            Operation::LD5N => {
                self.i5 = Index::from(-self.load(instruction));
            }
            Operation::LD6N => {
                self.i6 = Index::from(-self.load(instruction));
            }
            Operation::STA => {
                self.store(self.a, instruction);
            }
            Operation::STX => {
                self.store(self.x, instruction);
            }
            Operation::ST1 => {
                self.store(self.i1.into(), instruction);
            }
            Operation::ST2 => {
                self.store(self.i2.into(), instruction);
            }
            Operation::ST3 => {
                self.store(self.i3.into(), instruction);
            }
            Operation::ST4 => {
                self.store(self.i4.into(), instruction);
            }
            Operation::ST5 => {
                self.store(self.i5.into(), instruction);
            }
            Operation::ST6 => {
                self.store(self.i6.into(), instruction);
            }
            Operation::STJ => {
                self.store(self.j.into(), instruction);
            }
            Operation::STZ => {
                self.store(Word::default(), instruction);
            }
            Operation::ADD => {
                let (sum, overflows) = self.a.overflowing_add(self.load(instruction));
                self.a = sum;
                self.overflow = Toggle::from(overflows);
            }
        };
        self
    }
}

fn main() {}

#[cfg(test)]
mod spec {
    use super::*;
    use Operation::*;
    use Sign::*;
    use Toggle::*;

    #[test]
    fn field_byte_conversions() {
        for l in 0..8 {
            for r in l..8 {
                let field = Modification::field(l, r);
                let byte: Byte = field.clone().into();
                assert_eq!(
                    field,
                    Modification::from(byte.clone()),
                    "round trip conversion of field specification through byte should be idempotent"
                );
                assert_eq!(
                    byte.clone(),
                    Modification::from(byte.clone()).into(),
                    "round trip conversion of byte through field specification through should be idempotent"
                );
            }
        }
    }

    fn instruction(
        operation: Operation,
        address: i16,
        index: Option<IndexNumber>,
        f: Option<Modification>,
    ) -> Instruction {
        Instruction::new(operation, Address::new(address), index, f)
    }

    fn fields(l: u8, r: u8) -> Option<Modification> {
        Some(Modification::field(l, r))
    }

    fn w(b0: u8, b1: u8, b2: u8, b3: u8, b4: u8) -> Word {
        Word::new(Plus, b0, b1, b2, b3, b4)
    }

    #[test]
    fn lda() {
        assert(None, -w(1, 16, 3, 5, 4));
        assert(fields(1, 5), w(1, 16, 3, 5, 4));
        assert(fields(3, 5), w(0, 0, 3, 5, 4));
        assert(fields(0, 3), -w(0, 0, 1, 16, 3));
        assert(fields(4, 4), w(0, 0, 0, 0, 5));
        assert(fields(0, 0), -w(0, 0, 0, 0, 0));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = -w(1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDA, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.a, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ldan() {
        assert(None, w(1, 16, 3, 5, 4));
        assert(fields(1, 5), -w(1, 16, 3, 5, 4));
        assert(fields(3, 5), -w(0, 0, 3, 5, 4));
        assert(fields(0, 3), w(0, 0, 1, 16, 3));
        assert(fields(4, 4), -w(0, 0, 0, 0, 5));
        assert(fields(0, 0), w(0, 0, 0, 0, 0));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = -w(1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDAN, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.a, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ldx() {
        assert(None, -w(1, 16, 3, 5, 4));
        assert(fields(1, 5), w(1, 16, 3, 5, 4));
        assert(fields(3, 5), w(0, 0, 3, 5, 4));
        assert(fields(0, 3), -w(0, 0, 1, 16, 3));
        assert(fields(4, 4), w(0, 0, 0, 0, 5));
        assert(fields(0, 0), -w(0, 0, 0, 0, 0));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = -w(1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LDX, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.x, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn ldxn() {
        assert(None, w(1, 16, 3, 5, 4));
        assert(fields(1, 5), -w(1, 16, 3, 5, 4));
        assert(fields(3, 5), -w(0, 0, 3, 5, 4));
        assert(fields(0, 3), w(0, 0, 1, 16, 3));
        assert(fields(4, 4), -w(0, 0, 0, 0, 5));
        assert(fields(0, 0), w(0, 0, 0, 0, 0));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
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
        fn assert(f: Option<Modification>, expected: Index) {
            let before = -w(1, 16, 3, 5, 4);
            let mut mix = Mix::default();
            mix.memory[2000] = before;

            let mix = mix.exec(instruction(LD6N, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], before, "should not change");
            assert_eq!(mix.i6, expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn sta() {
        assert(None, w(6, 7, 8, 9, 0));
        assert(fields(1, 5), -w(6, 7, 8, 9, 0));
        assert(fields(5, 5), -w(1, 2, 3, 4, 0));
        assert(fields(2, 2), -w(1, 0, 3, 4, 5));
        assert(fields(2, 3), -w(1, 9, 0, 4, 5));
        assert(fields(0, 1), w(0, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = w(6, 7, 8, 9, 0);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.a = before;

            let mix = mix.exec(instruction(STA, 2000, None, f.clone()));

            assert_eq!(mix.a, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn stx() {
        assert(None, w(6, 7, 8, 9, 0));
        assert(fields(1, 5), -w(6, 7, 8, 9, 0));
        assert(fields(5, 5), -w(1, 2, 3, 4, 0));
        assert(fields(2, 2), -w(1, 0, 3, 4, 5));
        assert(fields(2, 3), -w(1, 9, 0, 4, 5));
        assert(fields(0, 1), w(0, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = w(6, 7, 8, 9, 0);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.x = before;

            let mix = mix.exec(instruction(STX, 2000, None, f.clone()));

            assert_eq!(mix.x, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn st1() {
        assert(None, w(0, 0, 0, 6, 7));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Index::new(Plus, 6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.i1 = before;

            let mix = mix.exec(instruction(ST1, 2000, None, f.clone()));

            assert_eq!(mix.i1, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn st2() {
        assert(None, w(0, 0, 0, 6, 7));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Index::new(Plus, 6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.i2 = before;

            let mix = mix.exec(instruction(ST2, 2000, None, f.clone()));

            assert_eq!(mix.i2, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn st3() {
        assert(None, w(0, 0, 0, 6, 7));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Index::new(Plus, 6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.i3 = before;

            let mix = mix.exec(instruction(ST3, 2000, None, f.clone()));

            assert_eq!(mix.i3, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn st4() {
        assert(None, w(0, 0, 0, 6, 7));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Index::new(Plus, 6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.i4 = before;

            let mix = mix.exec(instruction(ST4, 2000, None, f.clone()));

            assert_eq!(mix.i4, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn st5() {
        assert(None, w(0, 0, 0, 6, 7));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Index::new(Plus, 6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.i5 = before;

            let mix = mix.exec(instruction(ST5, 2000, None, f.clone()));

            assert_eq!(mix.i5, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn st6() {
        assert(None, w(0, 0, 0, 6, 7));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Index::new(Plus, 6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.i6 = before;

            let mix = mix.exec(instruction(ST6, 2000, None, f.clone()));

            assert_eq!(mix.i6, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn stj() {
        assert(None, w(6, 7, 3, 4, 5));
        assert(fields(0, 2), w(6, 7, 3, 4, 5));
        assert(fields(1, 5), -w(0, 0, 0, 6, 7));
        assert(fields(5, 5), -w(1, 2, 3, 4, 7));
        assert(fields(2, 2), -w(1, 7, 3, 4, 5));
        assert(fields(2, 3), -w(1, 6, 7, 4, 5));
        assert(fields(0, 1), w(7, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let before = Jump::new(6, 7);
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);
            mix.j = before;

            let mix = mix.exec(instruction(STJ, 2000, None, f.clone()));

            assert_eq!(mix.j, before, "should not change");
            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn stz() {
        assert(None, w(0, 0, 0, 0, 0));
        assert(fields(1, 5), -w(0, 0, 0, 0, 0));
        assert(fields(5, 5), -w(1, 2, 3, 4, 0));
        assert(fields(2, 2), -w(1, 0, 3, 4, 5));
        assert(fields(2, 3), -w(1, 0, 0, 4, 5));
        assert(fields(0, 1), w(0, 2, 3, 4, 5));
        fn assert(f: Option<Modification>, expected: Word) {
            let mut mix = Mix::default();
            mix.memory[2000] = -w(1, 2, 3, 4, 5);

            let mix = mix.exec(instruction(STZ, 2000, None, f.clone()));

            assert_eq!(mix.memory[2000], expected, "for specification {:?}", f);
        }
    }

    #[test]
    fn add_overflow() {
        assert(w(5, 1, 0, 2, 1), w(5, 1, 0, 3, 2), w(10, 2, 0, 5, 3), Off);
        assert(
            w(0, BYTE - 1, 0, 0, 0),
            w(0, 2, 0, 0, 0),
            w(1, 1, 0, 0, 0),
            Off,
        );
        assert(
            w(BYTE - 1, 0, 0, 0, 0),
            w(2, 0, 0, 0, 0),
            w(1, 0, 0, 0, 0),
            On,
        );
        assert(
            -w(BYTE - 1, 0, 0, 0, 0),
            -w(2, 0, 0, 0, 0),
            -w(1, 0, 0, 0, 0),
            On,
        );
        assert(
            w(0, BYTE - 2, BYTE - 1, 0, 0),
            w(0, 1, 1, 0, 0),
            w(1, 0, 0, 0, 0),
            Off,
        );
        fn assert(a: Word, b: Word, expected: Word, overflow: Toggle) {
            let mut mix = Mix::default();
            mix.a = a;
            mix.memory[2000] = b;

            let mix = mix.exec(instruction(ADD, 2000, None, None));

            assert_eq!(mix.memory[2000], b, "stays the same");
            assert_eq!(mix.a, expected);
            assert_eq!(mix.overflow, overflow);
        }
    }

    #[test]
    fn add_sign() {
        assert(w(0, 0, 0, 1, 0), -w(0, 0, 0, 0, 1), w(0, 0, 0, 0, BYTE - 1));
        assert(-w(0, 0, 0, 0, 1), -w(0, 0, 0, 0, 1), -w(0, 0, 0, 0, 2));
        assert(w(0, 0, 0, 0, 1), -w(0, 0, 0, 0, 3), -w(0, 0, 0, 0, 2));
        assert(-w(0, 0, 0, 0, 1), w(0, 0, 0, 0, 3), w(0, 0, 0, 0, 2));
        assert(w(0, 0, 0, 0, 1), -w(0, 0, 0, 0, 1), w(0, 0, 0, 0, 0));
        assert(-w(0, 0, 0, 0, 1), w(0, 0, 0, 0, 1), -w(0, 0, 0, 0, 0));
        fn assert(a: Word, b: Word, expected: Word) {
            let mut mix = Mix::default();
            mix.a = a;
            mix.memory[2000] = b;

            let mix = mix.exec(instruction(ADD, 2000, None, None));

            assert_eq!(mix.memory[2000], b, "stays the same");
            assert_eq!(mix.a, expected);
            assert_eq!(mix.overflow, Off);
        }
    }

    #[test]
    fn add_field() {
        assert(w(14, 13, 12, 11, 10), fields(1, 1), w(5, 4, 3, 2, 15));
        assert(w(14, 13, 12, 11, 10), fields(3, 3), w(5, 4, 3, 2, 13));
        assert(w(14, 13, 12, 11, 10), fields(5, 5), w(5, 4, 3, 2, 11));
        assert(-w(1, 1, 1, 1, 1), fields(0, 2), w(5, 4, 3, 1, 0));
        assert(-w(1, 1, 1, 1, 1), fields(1, 2), w(5, 4, 3, 3, 2));
        fn assert(b: Word, f: Option<Modification>, expected: Word) {
            let mut mix = Mix::default();
            mix.a = w(5, 4, 3, 2, 1);
            mix.memory[2000] = b;

            let mix = mix.exec(instruction(ADD, 2000, None, f));

            assert_eq!(mix.memory[2000], b, "stays the same");
            assert_eq!(mix.a, expected);
            assert_eq!(mix.overflow, Off);
        }
    }
}
