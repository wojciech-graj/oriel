use std::{collections::HashMap, fmt::Display};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use thiserror::Error;

pub use crate::ir::*;

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

macro_rules! next_pair_str_lit {
    ($pairs:expr) => {{
        let span = next_pair!($pairs).as_span();
        pest::Span::new(span.get_input(), span.start() + 1, span.end() - 1)
            .unwrap()
            .as_str()
    }};
}

macro_rules! enum_impl_from_str {
    (
        $name:ident, $( ( $variant:ident, $str_rep:literal ) ),*
    ) => {
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

enum_impl_from_str!(
    LogicalOperator,
    (Equal, "="),
    (Less, "<"),
    (Greater, ">"),
    (LEqual, "<="),
    (GEqual, ">="),
    (NEqual, "<>")
);

enum_impl_from_str!(
    MathOperator,
    (Add, "+"),
    (Subtract, "-"),
    (Multiply, "*"),
    (Divide, "/")
);

enum_impl_from_str!(
    MessageBoxType,
    (Ok, "OK"),
    (OkCancel, "OKCANCEL"),
    (YesNo, "YESNO"),
    (YesNoCancel, "YESNOCANCEL")
);

enum_impl_from_str!(
    MessageBoxIcon,
    (Information, "INFORMATION"),
    (Exclamation, "EXCLAMATION"),
    (Question, "QUESTION"),
    (Stop, "STOP"),
    (NoIcon, "NOICON")
);

enum_impl_from_str!(
    SetWindowOption,
    (Maximize, "MAXIMIZE"),
    (Minimize, "MINIMIZE"),
    (Restore, "RESTORE")
);

enum_impl_from_str!(
    UseBackgroundOption,
    (Opaque, "OPAQUE"),
    (Transparent, "TRANSPARENT")
);

enum_impl_from_str!(
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

enum_impl_from_str!(UseCoordinatesOption, (Pixel, "PIXEL"), (Metric, "METRIC"));

enum_impl_from_str!(WaitMode, (Null, "NULL"), (Focus, "FOCUS"));

enum_impl_from_str!(
    UsePenOption,
    (Solid, "SOLID"),
    (Null, "NULL"),
    (Dash, "DASH"),
    (Dot, "DOT"),
    (DashDot, "DASHDOT"),
    (DashDotDot, "DASHDOTDOT")
);

enum_impl_from_str!(UseFontBold, (Bold, "BOLD"), (NoBold, "NOBOLD"));

enum_impl_from_str!(UseFontItalic, (Italic, "ITALIC"), (NoItalic, "NOITALIC"));

enum_impl_from_str!(
    UseFontUnderline,
    (Underline, "UNDERLINE"),
    (NoUnderline, "NOUNDERLINE")
);

impl<'a> From<&Pair<'a, Rule>> for Identifier<'a> {
    fn from(value: &Pair<'a, Rule>) -> Self {
        Identifier(value.as_str())
    }
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
                filename: next_pair_str_lit!(kwords),
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
                filename: next_pair_str_lit!(kwords),
            },
            "drawtext" => Command::DrawText {
                x: next_pair!(kwords).try_into()?,
                y: next_pair!(kwords).try_into()?,
                text: next_pair_str_lit!(kwords),
            },
            "messagebox" => Command::MessageBox {
                typ: next_pair!(kwords).try_into()?,
                default_button: next_pair!(kwords).try_into()?,
                icon: next_pair!(kwords).try_into()?,
                text: next_pair_str_lit!(kwords),
                caption: next_pair_str_lit!(kwords),
                button_pushed: Identifier(next_pair!(kwords).as_str()),
            },
            "run" => Command::Run(next_pair_str_lit!(kwords)),
            "setkeyboard" => {
                let mut params: Vec<SetKeyboardParam> = Vec::new();
                while kwords.peek().is_some() {
                    params.push(SetKeyboardParam {
                        key: next_pair_str_lit!(kwords),
                        label: next_pair!(kwords).into(),
                    });
                }
                Command::SetKeyboard(params)
            }
            "setmenu" => Command::SetMenu(),
            "setmouse" => {
                let mut params: Vec<SetMouseParam> = Vec::new();
                while kwords.peek().is_some() {
                    params.push(SetMouseParam {
                        x1: next_pair!(kwords).try_into()?,
                        y1: next_pair!(kwords).try_into()?,
                        x2: next_pair!(kwords).try_into()?,
                        y2: next_pair!(kwords).try_into()?,
                        label: next_pair!(kwords).into(),
                        x: next_pair!(kwords).into(),
                        y: next_pair!(kwords).into(),
                    });
                }
                Command::SetMouse(params)
            }
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
            "usecaption" => Command::UseCaption(next_pair_str_lit!(kwords)),
            "usecoordinates" => Command::UseCoordinates(next_pair!(kwords).try_into()?),
            "usefont" => Command::UseFont {
                name: next_pair_str_lit!(kwords),
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
                            goto_false: 0,
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
                        if label.as_span().start_pos().line_col().1 > 1 {
                            println!("{}", label.as_span().start_pos().line_col().1);
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
            let goto_false_tgt = prog.commands.len();
            if let Command::If {
                i1: _,
                op: _,
                i2: _,
                goto_false: goto_false_idx,
            } = &mut prog.commands[idx]
            {
                *goto_false_idx = goto_false_tgt;
            }
        }
    }

    prog.commands.push(Command::End);

    Ok(prog)
}
