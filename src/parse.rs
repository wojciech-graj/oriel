use std::collections::HashMap;

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
        $pairs.next().ok_or_else(|| Error::MissingPairError)?
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

macro_rules! make_from_str_enum {
    (
        $name:ident, $( ( $variant:ident, $str_rep:literal ) ),*
    ) => {
        #[derive(Debug)]
        pub enum $name {
            $( $variant, )*
        }
        impl<'a> TryFrom<Pair<'a, Rule>> for $name {
            type Error = Error<'a>;
            fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
                match value.as_str() {
                    $($str_rep => Ok($name::$variant),)*
                    _ => Err(Self::Error::MatchTokenError(value)),
                }
            }
        }
    };
}

make_from_str_enum!(
    LogicalOperator,
    (Equal, "="),
    (Less, "<"),
    (Greater, ">"),
    (LEqual, "<="),
    (GEqual, ">="),
    (NEqual, "<>")
);

make_from_str_enum!(
    MathOperator,
    (Add, "+"),
    (Subtract, "-"),
    (Multiply, "*"),
    (Divide, "/")
);

make_from_str_enum!(
    MessageBoxType,
    (Ok, "OK"),
    (OkCancel, "OKCANCEL"),
    (YesNo, "YESNO"),
    (YesNoCancel, "YESNOCANCEL")
);

make_from_str_enum!(
    MessageBoxIcon,
    (Information, "INFORMATION"),
    (Exclamation, "EXCLAMATION"),
    (Question, "QUESTION"),
    (Stop, "STOP"),
    (NoIcon, "NOICON")
);

make_from_str_enum!(
    SetWindowOption,
    (Maximize, "MAXIMIZE"),
    (Minimize, "MINIMIZE"),
    (Restore, "RESTORE")
);

make_from_str_enum!(
    UseBackgroundOption,
    (Opaque, "OPAQUE"),
    (Transparent, "TRANSPARENT")
);

make_from_str_enum!(
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

make_from_str_enum!(UseCoordinatesOption, (Pixel, "PIXEL"), (Metric, "METRIC"));

make_from_str_enum!(WaitMode, (Null, "NULL"), (Focus, "FOCUS"));

make_from_str_enum!(
    UsePenOption,
    (Solid, "SOLID"),
    (Null, "NULL"),
    (Dash, "DASH"),
    (Dot, "DOT"),
    (DashDot, "DASHDOT"),
    (DashDotDot, "DASHDOTDOT")
);

make_from_str_enum!(UseFontBold, (Bold, "BOLD"), (NoBold, "NOBOLD"));

make_from_str_enum!(UseFontItalic, (Italic, "ITALIC"), (NoItalic, "NOITALIC"));

make_from_str_enum!(
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

impl<'a> TryFrom<Pair<'a, Rule>> for Integer<'a> {
    type Error = Error<'a>;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Integer<'a>, Self::Error> {
        match pair.as_rule() {
            Rule::integer => Ok(Integer::Literal(
                pair.as_str()
                    .parse::<u16>()
                    .map_err(|_| Self::Error::ParseIntError(pair))?,
            )),
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

fn pair_fmt_loc(pair: &Pair<Rule>) -> String {
    let (line, col) = pair.as_span().start_pos().line_col();
    format!("{}:{}:", line, col)
}

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("{} Failed to parse integer '{}'", pair_fmt_loc(.0), .0.as_str())]
    ParseIntError(Pair<'a, Rule>),
    #[error("{}", .0)]
    PestParseError(#[from] pest::error::Error<Rule>),
    #[error("Expected another tok")]
    MissingPairError,
    #[error("{} Failed to match token '{}'", pair_fmt_loc(.0), .0.as_str())]
    MatchTokenError(Pair<'a, Rule>),
    #[error("{} Label '{}' is not at line start", pair_fmt_loc(.0), .0.as_str())]
    LabelIndentationError(Pair<'a, Rule>),
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
        match next_pair!(kwords).as_str().to_lowercase().as_str() {
            "drawarc" => Ok(Command::DrawArc {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
                x4: next_pair!(kwords).try_into()?,
                y4: next_pair!(kwords).try_into()?,
            }),
            "drawbitmap" => Ok(Command::DrawBitmap {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                filename: next_pair_str!(kwords),
            }),
            "drawchord" => Ok(Command::DrawChord {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
                x4: next_pair!(kwords).try_into()?,
                y4: next_pair!(kwords).try_into()?,
            }),
            "drawellipse" => Ok(Command::DrawEllipse {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
            }),
            "drawflood" => Ok(Command::DrawFlood {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            }),
            "drawline" => Ok(Command::DrawLine {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
            }),
            "drawnumber" => Ok(Command::DrawNumber {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                n: next_pair!(kwords).try_into()?,
            }),
            "drawpie" => Ok(Command::DrawPie {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
                x4: next_pair!(kwords).try_into()?,
                y4: next_pair!(kwords).try_into()?,
            }),
            "drawrectangle" => Ok(Command::DrawRectangle {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
            }),
            "drawroundrectangle" => Ok(Command::DrawRoundRectangle {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                x3: next_pair!(kwords).try_into()?,
                y3: next_pair!(kwords).try_into()?,
            }),
            "drawsizedbitmap" => Ok(Command::DrawSizedBitmap {
                x1: next_pair!(kwords).try_into()?,
                y1: next_pair!(kwords).try_into()?,
                x2: next_pair!(kwords).try_into()?,
                y2: next_pair!(kwords).try_into()?,
                filename: next_pair_str!(kwords),
            }),
            "drawtext" => Ok(Command::DrawText {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                text: next_pair_str!(kwords),
            }),
            "messagebox" => Ok(Command::MessageBox {
                typ: next_pair!(kwords).try_into()?,
                default_button: next_pair!(kwords).try_into()?,
                icon: next_pair!(kwords).try_into()?,
                text: next_pair_str!(kwords),
                caption: next_pair_str!(kwords),
                button_pushed: Identifier(next_pair!(kwords).as_str()),
            }),
            "run" => Ok(Command::Run(next_pair_str!(kwords))),
            "setkeyboard" => Ok(Command::SetKeyboard()),
            "setmenu" => Ok(Command::SetMenu()),
            "setmouse" => Ok(Command::SetMouse()),
            "setwaitmode" => Ok(Command::SetWaitMode(next_pair!(kwords).try_into()?)),
            "setwindow" => Ok(Command::SetWindow(next_pair!(kwords).try_into()?)),
            "usebackground" => Ok(Command::UseBackground {
                option: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            }),
            "usebrush" => Ok(Command::UseBrush {
                option: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            }),
            "usecaption" => Ok(Command::UseCaption(next_pair_str!(kwords))),
            "usecoordinates" => Ok(Command::UseCoordinates(next_pair!(kwords).try_into()?)),
            "usefont" => Ok(Command::UseFont {
                name: next_pair_str!(kwords),
                width: next_pair!(kwords).try_into()?,
                height: next_pair!(kwords).try_into()?,
                bold: next_pair!(kwords).try_into()?,
                italic: next_pair!(kwords).try_into()?,
                underline: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            }),
            "usepen" => Ok(Command::UsePen {
                option: next_pair!(kwords).try_into()?,
                width: next_pair!(kwords).try_into()?,
                r: next_pair!(kwords).try_into()?,
                g: next_pair!(kwords).try_into()?,
                b: next_pair!(kwords).try_into()?,
            }),
            "waitinput" => Ok(Command::WaitInput(
                if let Some(milliseconds) = kwords.next() {
                    Some(milliseconds.try_into()?)
                } else {
                    None
                },
            )),
            _ => unreachable!(),
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

    //TODO: check if exhausted pairs

    let program = next_pair!(pairs);
    for command_group in program.into_inner() {
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
                            next_pair!(command_part.into_inner().skip(1)).as_str(),
                        )));
                    }
                    Rule::command_gosub => {
                        prog.commands.push(Command::Gosub(Identifier(
                            next_pair!(command_part.into_inner().skip(1)).as_str(),
                        )));
                    }
                    Rule::command_if_then => {
                        let mut kwords = command_part.into_inner();
                        if_indices.push(prog.commands.len());
                        prog.commands.push(Command::If {
                            i1: next_pair!(kwords).try_into()?,
                            op: next_pair!(kwords).try_into()?,
                            i2: next_pair!(kwords).try_into()?,
                            n_commands: 0,
                        });
                    }
                    Rule::command_set => {
                        let mut kwords = command_part.into_inner();
                        prog.commands.push(Command::Set {
                            var: Identifier(next_pair!(kwords).as_str()),
                            i1: next_pair!(kwords).try_into()?,
                            op: next_pair!(kwords).try_into()?,
                            i2: next_pair!(kwords).try_into()?,
                        });
                    }
                    Rule::label => {
                        let label = next_pair!(command_part.into_inner());
                        if label.as_span().start_pos().line_col().1 != 0 {
                            return Err(Error::LabelIndentationError(label));
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
