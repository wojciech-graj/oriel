use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    Equal,
    Less,
    Greater,
    LEqual,
    GEqual,
    NEqual,
}

#[derive(Debug, Clone, Copy)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageBoxType {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageBoxIcon {
    Information,
    Exclamation,
    Question,
    Stop,
    NoIcon,
}

#[derive(Debug, Clone, Copy)]
pub enum SetWindowOption {
    Maximize,
    Minimize,
    Restore,
}

#[derive(Debug, Clone, Copy)]
pub enum BackgroundTransparency {
    Opaque,
    Transparent,
}

#[derive(Debug, Clone, Copy)]
pub enum BrushType {
    Solid,
    DiagonalUp,
    DiagonalDown,
    DiagonalCross,
    Horizontal,
    Vertical,
    Cross,
    Null,
}

#[derive(Debug, Clone, Copy)]
pub enum Coordinates {
    Pixel,
    Metric,
}

#[derive(Debug, Clone, Copy)]
pub enum WaitMode {
    Null,
    Focus,
}

#[derive(Debug, Clone, Copy)]
pub enum PenType {
    Solid,
    Null,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
}

#[derive(Debug, Clone, Copy)]
pub enum FontWeight {
    Bold,
    NoBold,
}

#[derive(Debug, Clone, Copy)]
pub enum FontSlant {
    Italic,
    NoItalic,
}

#[derive(Debug, Clone, Copy)]
pub enum FontUnderline {
    Underline,
    NoUnderline,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum VirtualKey {
    BackSpace,
    Tab,
    NumPad5NoLock,
    Enter,
    Shift,
    Ctrl,
    Alt,
    Pause,
    CapsLock,
    Escape,
    Space,
    PgUp,
    PgDn,
    End,
    Home,
    LeftArrow,
    UpArrow,
    RightArrow,
    DownArrow,
    PrintScreen,
    Insert,
    Delete,
    AlNum(char),
    NumPad(char),
    F(u8),
    NumLock,
    ScrollLock,
    ColonOrSemiColon,
    PlusOrEqual,
    LessOrComma,
    UnderscoreOrHyphen,
    GreaterOrPeriod,
    QuestionOrSlash,
    TildeOrBackwardsSingleQuote,
    LeftCurlyOrLeftSquare,
    PipeOrBackslash,
    RightCurlyOrRightSquare,
    DoubleQuoteOrSingleQuote,
}

impl TryFrom<u16> for VirtualKey {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            8 => VirtualKey::BackSpace,
            9 => VirtualKey::Tab,
            12 => VirtualKey::NumPad5NoLock,
            13 => VirtualKey::Enter,
            16 => VirtualKey::Shift,
            17 => VirtualKey::Ctrl,
            18 => VirtualKey::Alt,
            19 => VirtualKey::Pause,
            20 => VirtualKey::CapsLock,
            27 => VirtualKey::Escape,
            32 => VirtualKey::Space,
            33 => VirtualKey::PgUp,
            34 => VirtualKey::PgDn,
            35 => VirtualKey::End,
            36 => VirtualKey::Home,
            37 => VirtualKey::LeftArrow,
            38 => VirtualKey::UpArrow,
            39 => VirtualKey::RightArrow,
            40 => VirtualKey::DownArrow,
            44 => VirtualKey::PrintScreen,
            45 => VirtualKey::Insert,
            46 => VirtualKey::Delete,
            48 => VirtualKey::AlNum('0'),
            49 => VirtualKey::AlNum('1'),
            50 => VirtualKey::AlNum('2'),
            51 => VirtualKey::AlNum('3'),
            52 => VirtualKey::AlNum('4'),
            53 => VirtualKey::AlNum('5'),
            54 => VirtualKey::AlNum('6'),
            55 => VirtualKey::AlNum('7'),
            56 => VirtualKey::AlNum('8'),
            57 => VirtualKey::AlNum('9'),
            65 => VirtualKey::AlNum('A'),
            66 => VirtualKey::AlNum('B'),
            67 => VirtualKey::AlNum('C'),
            68 => VirtualKey::AlNum('D'),
            69 => VirtualKey::AlNum('E'),
            70 => VirtualKey::AlNum('F'),
            71 => VirtualKey::AlNum('G'),
            72 => VirtualKey::AlNum('H'),
            73 => VirtualKey::AlNum('I'),
            74 => VirtualKey::AlNum('J'),
            75 => VirtualKey::AlNum('K'),
            76 => VirtualKey::AlNum('L'),
            77 => VirtualKey::AlNum('M'),
            78 => VirtualKey::AlNum('N'),
            79 => VirtualKey::AlNum('O'),
            80 => VirtualKey::AlNum('P'),
            81 => VirtualKey::AlNum('Q'),
            82 => VirtualKey::AlNum('R'),
            83 => VirtualKey::AlNum('S'),
            84 => VirtualKey::AlNum('T'),
            85 => VirtualKey::AlNum('U'),
            86 => VirtualKey::AlNum('V'),
            87 => VirtualKey::AlNum('W'),
            88 => VirtualKey::AlNum('X'),
            89 => VirtualKey::AlNum('Y'),
            90 => VirtualKey::AlNum('Z'),
            96 => VirtualKey::NumPad('0'),
            97 => VirtualKey::NumPad('1'),
            98 => VirtualKey::NumPad('2'),
            99 => VirtualKey::NumPad('3'),
            100 => VirtualKey::NumPad('4'),
            101 => VirtualKey::NumPad('5'),
            102 => VirtualKey::NumPad('6'),
            103 => VirtualKey::NumPad('7'),
            104 => VirtualKey::NumPad('8'),
            105 => VirtualKey::NumPad('9'),
            106 => VirtualKey::NumPad('*'),
            107 => VirtualKey::NumPad('+'),
            109 => VirtualKey::NumPad('-'),
            110 => VirtualKey::NumPad('.'),
            111 => VirtualKey::NumPad('/'),
            112 => VirtualKey::F(1),
            113 => VirtualKey::F(2),
            114 => VirtualKey::F(3),
            115 => VirtualKey::F(4),
            116 => VirtualKey::F(5),
            117 => VirtualKey::F(6),
            118 => VirtualKey::F(7),
            119 => VirtualKey::F(8),
            120 => VirtualKey::F(9),
            121 => VirtualKey::F(10),
            122 => VirtualKey::F(11),
            123 => VirtualKey::F(12),
            124 => VirtualKey::F(13),
            125 => VirtualKey::F(14),
            126 => VirtualKey::F(15),
            127 => VirtualKey::F(16),
            144 => VirtualKey::NumLock,
            145 => VirtualKey::ScrollLock,
            186 => VirtualKey::ColonOrSemiColon,
            187 => VirtualKey::PlusOrEqual,
            188 => VirtualKey::LessOrComma,
            189 => VirtualKey::UnderscoreOrHyphen,
            190 => VirtualKey::GreaterOrPeriod,
            191 => VirtualKey::QuestionOrSlash,
            192 => VirtualKey::TildeOrBackwardsSingleQuote,
            219 => VirtualKey::LeftCurlyOrLeftSquare,
            220 => VirtualKey::PipeOrBackslash,
            221 => VirtualKey::RightCurlyOrRightSquare,
            222 => VirtualKey::DoubleQuoteOrSingleQuote,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PhysicalKey {
    pub chr: char,
    pub ctrl: bool,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Key<'a> {
    Virtual(Integer<'a>),
    Physical(PhysicalKey),
}

#[derive(Debug, Clone)]
pub struct MenuCategory<'a> {
    pub item: MenuItem<'a>,
    pub members: Vec<MenuMember<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct MenuItem<'a> {
    pub name: &'a str,
    pub label: Option<Identifier<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuMember<'a> {
    Item(MenuItem<'a>),
    Separator,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseCallbacks<'a> {
    pub label: Identifier<'a>,
    pub x: Identifier<'a>,
    pub y: Identifier<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseRegion<'a> {
    pub x1: Integer<'a>,
    pub y1: Integer<'a>,
    pub x2: Integer<'a>,
    pub y2: Integer<'a>,
    pub callbacks: MouseCallbacks<'a>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Identifier<'a>(pub &'a str);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Integer<'a> {
    Literal(u16),
    Variable(Identifier<'a>),
}

#[derive(Debug)]
pub enum Command<'a> {
    Beep,
    DrawArc {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
        x3: Integer<'a>,
        y3: Integer<'a>,
        x4: Integer<'a>,
        y4: Integer<'a>,
    },
    DrawBackground,
    DrawBitmap {
        x: Integer<'a>,
        y: Integer<'a>,
        filename: &'a str,
    },
    DrawChord {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
        x3: Integer<'a>,
        y3: Integer<'a>,
        x4: Integer<'a>,
        y4: Integer<'a>,
    },
    DrawEllipse {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
    },
    DrawFlood {
        x: Integer<'a>,
        y: Integer<'a>,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    DrawLine {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
    },
    DrawNumber {
        x: Integer<'a>,
        y: Integer<'a>,
        n: Integer<'a>,
    },
    DrawPie {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
        x3: Integer<'a>,
        y3: Integer<'a>,
        x4: Integer<'a>,
        y4: Integer<'a>,
    },
    DrawRectangle {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
    },
    DrawRoundRectangle {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
        x3: Integer<'a>,
        y3: Integer<'a>,
    },
    DrawSizedBitmap {
        x1: Integer<'a>,
        y1: Integer<'a>,
        x2: Integer<'a>,
        y2: Integer<'a>,
        filename: &'a str,
    },
    DrawText {
        x: Integer<'a>,
        y: Integer<'a>,
        text: &'a str,
    },
    End,
    Gosub(Identifier<'a>),
    Return,
    Goto(Identifier<'a>),
    If {
        i1: Integer<'a>,
        op: LogicalOperator,
        i2: Integer<'a>,
        goto_false: usize,
    },
    MessageBox {
        typ: MessageBoxType,
        default_button: Integer<'a>,
        icon: MessageBoxIcon,
        text: &'a str,
        caption: &'a str,
        button_pushed: Identifier<'a>,
    },
    Run(&'a str),
    Set {
        var: Identifier<'a>,
        i1: Integer<'a>,
        op: MathOperator,
        i2: Integer<'a>,
    },
    SetKeyboard(HashMap<Key<'a>, Identifier<'a>>),
    SetMenu(Vec<MenuCategory<'a>>),
    SetMouse(Vec<MouseRegion<'a>>),
    SetWaitMode(WaitMode),
    SetWindow(SetWindowOption),
    UseBackground {
        option: BackgroundTransparency,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    UseBrush {
        option: BrushType,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    UseCaption(&'a str),
    UseCoordinates(Coordinates),
    UseFont {
        name: &'a str,
        width: Integer<'a>,
        height: Integer<'a>,
        bold: FontWeight,
        italic: FontSlant,
        underline: FontUnderline,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    UsePen {
        option: PenType,
        width: Integer<'a>,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    WaitInput(Option<Integer<'a>>),
}

#[derive(Debug)]
pub struct Program<'a> {
    pub commands: Vec<Command<'a>>,
    pub labels: HashMap<Identifier<'a>, usize>,
}
