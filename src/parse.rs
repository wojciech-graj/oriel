use std::{collections::HashMap, fmt::Display};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use thiserror::Error;

#[derive(Parser)]
#[grammar = "oriel.pest"]
struct IdentParser;

macro_rules! next_pair {
    ($pairs:expr) => {
        (&($pairs.next().ok_or_else(|| Error::MissingArgError)?))
    };
}

macro_rules! next_pair_unchecked {
    ($pairs:expr) => {
        (&($pairs.next().unwrap()))
    };
}

macro_rules! next_pair_str {
    ($pairs:expr) => {{
        let span = next_pair!($pairs).as_span();
        pest::Span::new(span.get_input(), span.start() + 1, span.end() - 1)
            .unwrap()
            .as_str()
    }};
}

macro_rules! make_enum_impl_from_str {
    (
        $name:ident, $( ( $variant:ident, $str_rep:literal ) ),*
    ) => {
        #[derive(Debug)]
        pub enum $name {
            $( $variant, )*
        }
        impl<'a> TryFrom<&Pair<'a, Rule>> for $name {
            type Error = Error<'a>;
            fn try_from(value: &Pair<'a, Rule>) -> Result<Self, Self::Error> {
                match value.as_str() {
                    $($str_rep => Ok($name::$variant),)*
                    _ => Err(Self::Error::MatchTokenError(value.into(), value.as_str())),
                }
            }
        }
    };
}

make_enum_impl_from_str!(
    LogicalOperator,
    (Equal, "="),
    (Less, "<"),
    (Greater, ">"),
    (LEqual, "<="),
    (GEqual, ">="),
    (NEqual, "<>")
);

make_enum_impl_from_str!(
    MathOperator,
    (Add, "+"),
    (Subtract, "-"),
    (Multiply, "*"),
    (Divide, "/")
);

make_enum_impl_from_str!(
    MessageBoxType,
    (Ok, "OK"),
    (OkCancel, "OKCANCEL"),
    (YesNo, "YESNO"),
    (YesNoCancel, "YESNOCANCEL")
);

make_enum_impl_from_str!(
    MessageBoxIcon,
    (Information, "INFORMATION"),
    (Exclamation, "EXCLAMATION"),
    (Question, "QUESTION"),
    (Stop, "STOP"),
    (NoIcon, "NOICON")
);

make_enum_impl_from_str!(
    SetWindowOption,
    (Maximize, "MAXIMIZE"),
    (Minimize, "MINIMIZE"),
    (Restore, "RESTORE")
);

make_enum_impl_from_str!(
    UseBackgroundOption,
    (Opaque, "OPAQUE"),
    (Transparent, "TRANSPARENT")
);

make_enum_impl_from_str!(
    UseBrushOption,
    (Solid, "SOLID"),
    (DiagonalUp, "DIAGONALUP"),
    (DiagonalDown, "DIAGONALDOWN"),
    (DiagonalCross, "DIAGONALCROSS"),
    (Horizontal, "HORIZONTAL"),
    (Vertical, "VERTICAL"),
    (Cross, "CROSS"),
    (Null, "NULL")
);

make_enum_impl_from_str!(UseCoordinatesOption, (Pixel, "PIXEL"), (Metric, "METRIC"));

make_enum_impl_from_str!(WaitMode, (Null, "NULL"), (Focus, "FOCUS"));

make_enum_impl_from_str!(
    UsePenOption,
    (Solid, "SOLID"),
    (Null, "NULL"),
    (Dash, "DASH"),
    (Dot, "DOT"),
    (DashDot, "DASHDOT"),
    (DashDotDot, "DASHDOTDOT")
);

make_enum_impl_from_str!(UseFontBold, (Bold, "BOLD"), (NoBold, "NOBOLD"));

make_enum_impl_from_str!(UseFontItalic, (Italic, "ITALIC"), (NoItalic, "NOITALIC"));

make_enum_impl_from_str!(
    UseFontUnderline,
    (Underline, "UNDERLINE"),
    (NoUnderline, "NOUNDERLINE")
);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Identifier<'a>(pub &'a str);

impl<'a> From<&'a str> for Identifier<'a> {
    fn from(value: &'a str) -> Self {
        Identifier(value)
    }
}

#[derive(Debug)]
pub enum Integer<'a> {
    Literal(u16),
    Variable(Identifier<'a>),
}

impl<'a> TryFrom<&Pair<'a, Rule>> for Integer<'a> {
    type Error = Error<'a>;

    fn try_from(pair: &Pair<'a, Rule>) -> Result<Integer<'a>, Self::Error> {
        match pair.as_rule() {
            Rule::integer => {
                Ok(Integer::Literal(pair.as_str().parse::<u16>().map_err(
                    |_| Self::Error::ParseIntError(pair.into(), pair.as_str()),
                )?))
            }
            Rule::identifier => Ok(Integer::Variable(Identifier(pair.as_str()))),
            _ => unreachable!(),
        }
    }
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
        n_commands: usize,
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
    SetKeyboard(
        // TODO
    ),
    SetMenu(
        // TODO
    ),
    SetMouse(
        // TODO
    ),
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
pub struct ErrorLoc {
    line: usize,
    col: usize,
}

impl<'a> From<&Pair<'a, Rule>> for ErrorLoc {
    fn from(value: &Pair<'a, Rule>) -> Self {
        let (line, col) = value.as_span().start_pos().line_col();
        ErrorLoc { line, col }
    }
}

impl Display for ErrorLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:", self.line, self.col)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("{} Failed to parse integer '{}'", .0, .1)]
    ParseIntError(ErrorLoc, &'a str),
    #[error("{}", .0)]
    PestParseError(Box<pest::error::Error<Rule>>),
    #[error("Expected another argument")]
    MissingArgError,
    #[error("{} Failed to match token '{}'", .0, .1)]
    MatchTokenError(ErrorLoc, &'a str),
    #[error("{} Label '{}' is not at line start", .0, .1)]
    LabelIndentationError(ErrorLoc, &'a str),
    #[error("{} Command '{}' has too many arguments", .0, .1)]
    ExtraneousArgError(ErrorLoc, &'a str),
}

impl From<pest::error::Error<Rule>> for Error<'_> {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Error::PestParseError(Box::new(value))
    }
}

impl<'a> Command<'a> {
    fn from_keyword(command: &Pair<'a, Rule>) -> Command<'a> {
        match command.as_str().to_lowercase().as_str() {
            "beep" => Command::Beep,
            "drawbackground" => Command::DrawBackground,
            "end" => Command::End,
            "return" => Command::Return,
            _ => unreachable!(),
        }
    }

    fn try_from_func(kwords: &mut Pairs<'a, Rule>) -> Result<Command<'a>, Error<'a>> {
        let fname = next_pair_unchecked!(kwords).as_str();
        let command = match fname.to_lowercase().as_str() {
            "drawarc" => Command::DrawArc {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
                x4: next_pair!(kwords).try_into()?,
                y4: next_pair!(kwords).try_into()?,
            },
            "drawbitmap" => Command::DrawBitmap {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                filename: next_pair_str!(kwords),
            },
            "drawchord" => Command::DrawChord {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
                x4: next_pair!(kwords).try_into()?,
                y4: next_pair!(kwords).try_into()?,
            },
            "drawellipse" => Command::DrawEllipse {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
            },
            "drawflood" => Command::DrawFlood {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            },
            "drawline" => Command::DrawLine {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
            },
            "drawnumber" => Command::DrawNumber {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                n: next_pair!(kwords).try_into()?,
            },
            "drawpie" => Command::DrawPie {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
                x4: next_pair!(kwords).try_into()?,
                y4: next_pair!(kwords).try_into()?,
            },
            "drawrectangle" => Command::DrawRectangle {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
            },
            "drawroundrectangle" => Command::DrawRoundRectangle {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
            },
            "drawsizedbitmap" => Command::DrawSizedBitmap {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                filename: next_pair_str!(kwords),
            },
            "drawtext" => Command::DrawText {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                text: next_pair_str!(kwords),
            },
            "messagebox" => Command::MessageBox {
                typ: next_pair!(kwords).try_into()?,
                default_button: next_pair!(kwords).try_into()?,
                icon: next_pair!(kwords).try_into()?,
                text: next_pair_str!(kwords),
                caption: next_pair_str!(kwords),
                button_pushed: Identifier(next_pair!(kwords).as_str()),
            },
            "run" => Command::Run(next_pair_str!(kwords)),
            "setkeyboard" => Command::SetKeyboard(),
            "setmenu" => Command::SetMenu(),
            "setmouse" => Command::SetMouse(),
            "setwaitmode" => Command::SetWaitMode(next_pair!(kwords).try_into()?),
            "setwindow" => Command::SetWindow(next_pair!(kwords).try_into()?),
            "usebackground" => Command::UseBackground {
                option: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            },
            "usebrush" => Command::UseBrush {
                option: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            },
            "usecaption" => Command::UseCaption(next_pair_str!(kwords)),
            "usecoordinates" => Command::UseCoordinates(next_pair!(kwords).try_into()?),
            "usefont" => Command::UseFont {
                name: next_pair_str!(kwords),
                width: next_pair!(kwords).try_into()?,
                height: next_pair!(kwords).try_into()?,
                bold: next_pair!(kwords).try_into()?,
                italic: next_pair!(kwords).try_into()?,
                underline: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            },
            "usepen" => Command::UsePen {
                option: next_pair!(kwords).try_into()?,
                width: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            },
            "waitinput" => Command::WaitInput(if let Some(ref milliseconds) = kwords.next() {
                Some(milliseconds.try_into()?)
            } else {
                None
            }),
            _ => unreachable!(),
        };

        if let Some(ref pair) = kwords.next() {
            Err(Error::ExtraneousArgError(pair.into(), fname))
        } else {
            Ok(command)
        }
    }
}

pub struct Program<'a> {
    pub commands: Vec<Command<'a>>,
    pub labels: HashMap<Identifier<'a>, usize>,
}

pub fn parse(src: &str) -> Result<Program<'_>, Error> {
    let mut pairs = IdentParser::parse(Rule::program, src)?;

    let mut prog = Program {
        commands: Vec::new(),
        labels: HashMap::new(),
    };

    for command_group in pairs.next().unwrap().into_inner() {
        let mut if_indices: Vec<usize> = Vec::new();
        for command in command_group.into_inner() {
            for command_part in command.into_inner() {
                match command_part.as_rule() {
                    Rule::kword_command_nfunc => {
                        prog.commands.push(Command::from_keyword(&command_part));
                    }
                    Rule::command_func => prog
                        .commands
                        .push(Command::try_from_func(&mut command_part.into_inner())?),
                    Rule::command_goto => {
                        prog.commands.push(Command::Goto(Identifier(
                            next_pair_unchecked!(command_part.into_inner().skip(1)).as_str(),
                        )));
                    }
                    Rule::command_gosub => {
                        prog.commands.push(Command::Gosub(Identifier(
                            next_pair_unchecked!(command_part.into_inner().skip(1)).as_str(),
                        )));
                    }
                    Rule::command_if_then => {
                        let mut kwords = command_part.into_inner();
                        if_indices.push(prog.commands.len());
                        prog.commands.push(Command::If {
                            i1: next_pair_unchecked!(kwords).try_into()?,
                            op: next_pair_unchecked!(kwords).try_into()?,
                            i2: next_pair_unchecked!(kwords).try_into()?,
                            n_commands: 0,
                        });
                    }
                    Rule::command_set => {
                        let mut kwords = command_part.into_inner();
                        prog.commands.push(Command::Set {
                            var: Identifier(next_pair_unchecked!(kwords).as_str()),
                            i1: next_pair_unchecked!(kwords).try_into()?,
                            op: next_pair_unchecked!(kwords).try_into()?,
                            i2: next_pair_unchecked!(kwords).try_into()?,
                        });
                    }
                    Rule::label => {
                        let label = next_pair_unchecked!(command_part.into_inner());
                        if label.as_span().start_pos().line_col().1 != 0 {
                            return Err(Error::LabelIndentationError(label.into(), label.as_str()));
                        }
                        prog.labels
                            .insert(Identifier(label.as_str()), prog.commands.len());
                    }
                    _ => unreachable!(),
                };
            }
        }

        for idx in if_indices {
            let nc = prog.commands.len() - idx - 1;
            if let Command::If {
                i1: _,
                op: _,
                i2: _,
                n_commands,
            } = &mut prog.commands[idx]
            {
                *n_commands = nc;
            }
        }
    }

    Ok(prog)
}
