use std::collections::HashMap;

#[derive(Debug)]
pub enum LogicalOperator {
    Equal,
    Less,
    Greater,
    LEqual,
    GEqual,
    NEqual,
}

#[derive(Debug)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum MessageBoxType {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

#[derive(Debug)]
pub enum MessageBoxIcon {
    Information,
    Exclamation,
    Question,
    Stop,
    NoIcon,
}

#[derive(Debug)]
pub enum SetWindowOption {
    Maximize,
    Minimize,
    Restore,
}

#[derive(Debug)]
pub enum UseBackgroundOption {
    Opaque,
    Transparent,
}

#[derive(Debug)]
pub enum UseBrushOption {
    Solid,
    DiagonalUp,
    DiagonalDown,
    DiagonalCross,
    Horizontal,
    Vertical,
    Cross,
    Null,
}

#[derive(Debug)]
pub enum UseCoordinatesOption {
    Pixel,
    Metric,
}

#[derive(Debug)]
pub enum WaitMode {
    Null,
    Focus,
}

#[derive(Debug)]
pub enum UsePenOption {
    Solid,
    Null,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
}

#[derive(Debug)]
pub enum UseFontBold {
    Bold,
    NoBold,
}

#[derive(Debug)]
pub enum UseFontItalic {
    Italic,
    NoItalic,
}

#[derive(Debug)]
pub enum UseFontUnderline {
    Underline,
    NoUnderline,
}

#[derive(Debug)]
pub struct SetKeyboardParam<'a> {
    pub key: &'a str,
    pub label: Identifier<'a>,
}

#[derive(Debug)]
pub struct SetMouseParam<'a> {
    pub x1: Integer<'a>,
    pub y1: Integer<'a>,
    pub x2: Integer<'a>,
    pub y2: Integer<'a>,
    pub label: Identifier<'a>,
    pub x: Identifier<'a>,
    pub y: Identifier<'a>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Identifier<'a>(pub &'a str);

#[derive(Debug)]
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
    SetKeyboard(Vec<SetKeyboardParam<'a>>),
    SetMenu(), // TODO
    SetMouse(Vec<SetMouseParam<'a>>),
    SetWaitMode(WaitMode),
    SetWindow(SetWindowOption),
    UseBackground {
        option: UseBackgroundOption,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    UseBrush {
        option: UseBrushOption,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    UseCaption(&'a str),
    UseCoordinates(UseCoordinatesOption),
    UseFont {
        name: &'a str,
        width: Integer<'a>,
        height: Integer<'a>,
        bold: UseFontBold,
        italic: UseFontItalic,
        underline: UseFontUnderline,
        r: Integer<'a>,
        g: Integer<'a>,
        b: Integer<'a>,
    },
    UsePen {
        option: UsePenOption,
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
