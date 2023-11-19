// Copyright (C) 2023  Wojciech Graj
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

use std::{collections::HashMap, fmt::Display};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use thiserror::Error;

use crate::{
    cfg,
    ir::{
        self, BackgroundTransparency, BrushType, Coordinates, FontSlant, FontUnderline, FontWeight,
        LogicalOperator, MathOperator, MessageBoxIcon, MessageBoxType, PenType, SetWindowOption,
        WaitMode,
    },
};

#[derive(Parser)]
#[grammar = "oriel.pest"]
struct OrielParser;

macro_rules! next_pair {
    ($pairs:expr) => {
        ($pairs.next().as_ref().ok_or_else(|| Error::MissingArgError))
    };
}

macro_rules! next_pair_unchecked {
    ($pairs:expr) => {
        ($pairs.next().as_ref().unwrap())
    };
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

fn str_lit_parse(s: &str) -> Option<&str> {
    if s.starts_with('"') && s.ends_with('"') {
        Some(&s[1..(s.len() - 1)])
    } else {
        None
    }
}

fn next_pair_set_menu_label<'a>(
    pairs: &mut Pairs<'a, Rule>,
) -> Result<Option<ir::Identifier<'a>>, Error<'a>> {
    let pair = pairs.next().ok_or_else(|| Error::MissingArgError)?;
    Ok(match pair.as_str() {
        "IGNORE" => None,
        _ => Some((&pair).try_into()?),
    })
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
    BackgroundTransparency,
    (Opaque, "OPAQUE"),
    (Transparent, "TRANSPARENT")
);

enum_impl_from_str!(
    BrushType,
    (Solid, "SOLID"),
    (DiagonalUp, "DIAGONALUP"),
    (DiagonalDown, "DIAGONALDOWN"),
    (DiagonalCross, "DIAGONALCROSS"),
    (Horizontal, "HORIZONTAL"),
    (Vertical, "VERTICAL"),
    (Cross, "CROSS"),
    (Null, "NULL")
);

enum_impl_from_str!(Coordinates, (Pixel, "PIXEL"), (Metric, "METRIC"));

enum_impl_from_str!(WaitMode, (Null, "NULL"), (Focus, "FOCUS"));

enum_impl_from_str!(
    PenType,
    (Solid, "SOLID"),
    (Null, "NULL"),
    (Dash, "DASH"),
    (Dot, "DOT"),
    (DashDot, "DASHDOT"),
    (DashDotDot, "DASHDOTDOT")
);

enum_impl_from_str!(FontWeight, (Bold, "BOLD"), (NoBold, "NOBOLD"));

enum_impl_from_str!(FontSlant, (Italic, "ITALIC"), (NoItalic, "NOITALIC"));

enum_impl_from_str!(
    FontUnderline,
    (Underline, "UNDERLINE"),
    (NoUnderline, "NOUNDERLINE")
);

impl<'a> TryFrom<&Pair<'a, Rule>> for ir::Key<'a> {
    type Error = Error<'a>;

    fn try_from(value: &Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match value.as_rule() {
            Rule::identifier | Rule::integer => Ok(ir::Key::Virtual(value.try_into()?)),
            Rule::string => Ok(ir::Key::Physical(value.try_into()?)),
            _ => Err(Error::ArgTypeError(value.into(), value.as_str())),
        }
    }
}

impl<'a> TryFrom<&Pair<'a, Rule>> for ir::Identifier<'a> {
    type Error = Error<'a>;

    fn try_from(value: &Pair<'a, Rule>) -> Result<Self, Self::Error> {
        if let Rule::identifier = value.as_rule() {
            Ok(ir::Identifier(value.as_str()))
        } else {
            Err(Error::ArgTypeError(value.into(), value.as_str()))
        }
    }
}

impl<'a> TryFrom<&Pair<'a, Rule>> for ir::Integer<'a> {
    type Error = Error<'a>;

    fn try_from(pair: &Pair<'a, Rule>) -> Result<ir::Integer<'a>, Self::Error> {
        match pair.as_rule() {
            Rule::integer => {
                Ok(ir::Integer::Literal(pair.as_str().parse::<u16>().map_err(
                    |_| Self::Error::ParseIntError(pair.into(), pair.as_str()),
                )?))
            }
            Rule::identifier => Ok(ir::Integer::Variable(ir::Identifier(pair.as_str()))),
            _ => Err(Error::ArgTypeError(pair.into(), pair.as_str())),
        }
    }
}

impl<'a> TryFrom<&Pair<'a, Rule>> for ir::Str<'a> {
    type Error = Error<'a>;

    fn try_from(pair: &Pair<'a, Rule>) -> Result<ir::Str<'a>, Self::Error> {
        match pair.as_rule() {
            Rule::string => Ok(ir::Str::Literal(
                str_lit_parse(pair.as_str())
                    .ok_or_else(|| Error::ArgTypeError(pair.into(), pair.as_str()))?,
            )),
            Rule::identifier_str => Ok(ir::Str::Variable(ir::Identifier(pair.as_str()))),
            _ => Err(Error::ArgTypeError(pair.into(), pair.as_str())),
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
    #[error("{} Argument '{}' has incorrect type", .0, .1)]
    ArgTypeError(ErrorLoc, &'a str),
    #[error("Number of labels exceeds 500")]
    ExcessLabelsError,
    #[error("{} '{}' is unsupported by Oriel {}", .0, .1, .2)]
    StandardUnsupportedError(ErrorLoc, &'a str, cfg::Standard),
}

impl From<pest::error::Error<Rule>> for Error<'_> {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Error::PestParseError(Box::new(value))
    }
}

impl<'a> ir::Command<'a> {
    fn try_from_keyword(command: &Pair<'a, Rule>) -> Result<ir::Command<'a>, Error<'a>> {
        match command.as_str().to_lowercase().as_str() {
            "beep" => Ok(ir::Command::Beep),
            "drawbackground" => Ok(ir::Command::DrawBackground),
            "end" => Ok(ir::Command::End),
            "return" => Ok(ir::Command::Return),
            _ => unreachable!(),
        }
    }

    fn try_from_func(kwords: &mut Pairs<'a, Rule>) -> Result<ir::Command<'a>, Error<'a>> {
        let fname = next_pair_unchecked!(kwords).as_str();
        let command = match fname.to_lowercase().as_str() {
            "drawarc" => ir::Command::DrawArc {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
                x3: next_pair!(kwords)?.try_into()?,
                y3: next_pair!(kwords)?.try_into()?,
                x4: next_pair!(kwords)?.try_into()?,
                y4: next_pair!(kwords)?.try_into()?,
            },
            "drawbitmap" => ir::Command::DrawBitmap {
                x: next_pair!(kwords)?.try_into()?,
                y: next_pair!(kwords)?.try_into()?,
                filename: next_pair!(kwords)?.try_into()?,
            },
            "drawchord" => ir::Command::DrawChord {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
                x3: next_pair!(kwords)?.try_into()?,
                y3: next_pair!(kwords)?.try_into()?,
                x4: next_pair!(kwords)?.try_into()?,
                y4: next_pair!(kwords)?.try_into()?,
            },
            "drawellipse" => ir::Command::DrawEllipse {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
            },
            "drawflood" => ir::Command::DrawFlood {
                x: next_pair!(kwords)?.try_into()?,
                y: next_pair!(kwords)?.try_into()?,
                r: next_pair!(kwords)?.try_into()?,
                g: next_pair!(kwords)?.try_into()?,
                b: next_pair!(kwords)?.try_into()?,
            },
            "drawline" => ir::Command::DrawLine {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
            },
            "drawnumber" => ir::Command::DrawNumber {
                x: next_pair!(kwords)?.try_into()?,
                y: next_pair!(kwords)?.try_into()?,
                n: next_pair!(kwords)?.try_into()?,
            },
            "drawpie" => ir::Command::DrawPie {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
                x3: next_pair!(kwords)?.try_into()?,
                y3: next_pair!(kwords)?.try_into()?,
                x4: next_pair!(kwords)?.try_into()?,
                y4: next_pair!(kwords)?.try_into()?,
            },
            "drawrectangle" => ir::Command::DrawRectangle {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
            },
            "drawroundrectangle" => ir::Command::DrawRoundRectangle {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
                x3: next_pair!(kwords)?.try_into()?,
                y3: next_pair!(kwords)?.try_into()?,
            },
            "drawsizedbitmap" => ir::Command::DrawSizedBitmap {
                x1: next_pair!(kwords)?.try_into()?,
                y1: next_pair!(kwords)?.try_into()?,
                x2: next_pair!(kwords)?.try_into()?,
                y2: next_pair!(kwords)?.try_into()?,
                filename: next_pair!(kwords)?.try_into()?,
            },
            "drawtext" => ir::Command::DrawText {
                x: next_pair!(kwords)?.try_into()?,
                y: next_pair!(kwords)?.try_into()?,
                text: next_pair!(kwords)?.try_into()?,
            },
            "messagebox" => ir::Command::MessageBox {
                typ: next_pair!(kwords)?.try_into()?,
                default_button: next_pair!(kwords)?.try_into()?,
                icon: next_pair!(kwords)?.try_into()?,
                text: next_pair!(kwords)?.try_into()?,
                caption: next_pair!(kwords)?.try_into()?,
                button_pushed: next_pair!(kwords)?.try_into()?,
            },
            "run" => ir::Command::Run(next_pair!(kwords)?.try_into()?),
            "setkeyboard" => ir::Command::SetKeyboard({
                let mut params: HashMap<ir::Key, ir::Identifier> = HashMap::new();
                while kwords.peek().is_some() {
                    params.insert(
                        next_pair!(kwords)?.try_into()?,
                        next_pair!(kwords)?.try_into()?,
                    );
                }
                params
            }),
            "setmenu" => {
                let mut items: Vec<ir::MenuCategory> = Vec::new();
                while kwords.peek().is_some() {
                    items.push(ir::MenuCategory {
                        item: ir::MenuItem {
                            name: next_pair!(kwords)?.try_into()?,
                            label: next_pair_set_menu_label(kwords)?,
                        },
                        members: {
                            let mut members = Vec::new();
                            loop {
                                let pair = kwords.next().ok_or_else(|| Error::MissingArgError)?;
                                members.push(match pair.as_str() {
                                    "ENDPOPUP" => break,
                                    "SEPARATOR" => ir::MenuMember::Separator,
                                    _ => ir::MenuMember::Item(ir::MenuItem {
                                        name: (&pair).try_into()?,
                                        label: next_pair_set_menu_label(kwords)?,
                                    }),
                                });
                            }
                            members
                        },
                    });
                }
                ir::Command::SetMenu(items)
            }
            "setmouse" => ir::Command::SetMouse({
                let mut params: Vec<ir::MouseRegion> = Vec::new();
                while kwords.peek().is_some() {
                    params.push(ir::MouseRegion {
                        x1: next_pair!(kwords)?.try_into()?,
                        y1: next_pair!(kwords)?.try_into()?,
                        x2: next_pair!(kwords)?.try_into()?,
                        y2: next_pair!(kwords)?.try_into()?,
                        callbacks: ir::MouseCallbacks {
                            label: next_pair!(kwords)?.try_into()?,
                            x: next_pair!(kwords)?.try_into()?,
                            y: next_pair!(kwords)?.try_into()?,
                        },
                    });
                }
                params
            }),
            "setwaitmode" => ir::Command::SetWaitMode(next_pair!(kwords)?.try_into()?),
            "setwindow" => ir::Command::SetWindow(next_pair!(kwords)?.try_into()?),
            "usebackground" => ir::Command::UseBackground {
                option: next_pair!(kwords)?.try_into()?,
                r: next_pair!(kwords)?.try_into()?,
                g: next_pair!(kwords)?.try_into()?,
                b: next_pair!(kwords)?.try_into()?,
            },
            "usebrush" => ir::Command::UseBrush {
                option: next_pair!(kwords)?.try_into()?,
                r: next_pair!(kwords)?.try_into()?,
                g: next_pair!(kwords)?.try_into()?,
                b: next_pair!(kwords)?.try_into()?,
            },
            "usecaption" => ir::Command::UseCaption(next_pair!(kwords)?.try_into()?),
            "usecoordinates" => ir::Command::UseCoordinates(next_pair!(kwords)?.try_into()?),
            "usefont" => ir::Command::UseFont {
                name: next_pair!(kwords)?.try_into()?,
                width: next_pair!(kwords)?.try_into()?,
                height: next_pair!(kwords)?.try_into()?,
                bold: next_pair!(kwords)?.try_into()?,
                italic: next_pair!(kwords)?.try_into()?,
                underline: next_pair!(kwords)?.try_into()?,
                r: next_pair!(kwords)?.try_into()?,
                g: next_pair!(kwords)?.try_into()?,
                b: next_pair!(kwords)?.try_into()?,
            },
            "usepen" => ir::Command::UsePen {
                option: next_pair!(kwords)?.try_into()?,
                width: next_pair!(kwords)?.try_into()?,
                r: next_pair!(kwords)?.try_into()?,
                g: next_pair!(kwords)?.try_into()?,
                b: next_pair!(kwords)?.try_into()?,
            },
            "waitinput" => ir::Command::WaitInput(if let Some(ref milliseconds) = kwords.next() {
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

impl<'a> ir::Program<'a> {
    pub fn from_src(src: &'a str, config: &cfg::Config) -> Result<Self, Error<'a>> {
        let mut pairs = OrielParser::parse(Rule::program, src)?;

        let mut prog = Self {
            commands: Vec::new(),
            labels: HashMap::new(),
        };

        for command_group in pairs.next().unwrap().into_inner() {
            let mut if_indices: Vec<usize> = Vec::new();
            for command in command_group.into_inner() {
                for command_part in command.into_inner() {
                    match command_part.as_rule() {
                        Rule::kword_command_nfunc => {
                            prog.commands
                                .push(ir::Command::try_from_keyword(&command_part)?);
                        }
                        Rule::command_func => prog
                            .commands
                            .push(ir::Command::try_from_func(&mut command_part.into_inner())?),
                        Rule::command_goto => {
                            prog.commands.push(ir::Command::Goto(
                                next_pair_unchecked!(command_part.into_inner()).try_into()?,
                            ));
                        }
                        Rule::command_gosub => {
                            prog.commands.push(ir::Command::Gosub(
                                next_pair_unchecked!(command_part.into_inner()).try_into()?,
                            ));
                        }
                        Rule::command_if_then => {
                            let mut kwords = command_part.into_inner();
                            if_indices.push(prog.commands.len());
                            prog.commands.push(ir::Command::If {
                                i1: next_pair_unchecked!(kwords).try_into()?,
                                op: next_pair_unchecked!(kwords).try_into()?,
                                i2: next_pair_unchecked!(kwords).try_into()?,
                                goto_false: 0,
                            });
                        }
                        Rule::command_set => {
                            let mut kwords = command_part.into_inner();
                            let var = next_pair_unchecked!(kwords).try_into()?;
                            let i1 = next_pair_unchecked!(kwords).try_into()?;
                            let val = {
                                if kwords.peek().is_none() {
                                    ir::SetValue::Value(i1)
                                } else {
                                    ir::SetValue::Expression {
                                        i1,
                                        op: next_pair_unchecked!(kwords).try_into()?,
                                        i2: next_pair_unchecked!(kwords).try_into()?,
                                    }
                                }
                            };
                            prog.commands.push(ir::Command::Set { var, val });
                        }
                        Rule::label => {
                            if config.pedantic && prog.labels.len() >= 500 {
                                return Err(Error::ExcessLabelsError);
                            }
                            let label = &(command_part.into_inner().next().unwrap());
                            if label.as_span().start_pos().line_col().1 > 1 {
                                println!("{}", label.as_span().start_pos().line_col().1);
                                return Err(Error::LabelIndentationError(
                                    label.into(),
                                    label.as_str(),
                                ));
                            }
                            prog.labels
                                .insert(ir::Identifier(label.as_str()), prog.commands.len());
                        }
                        _ => unreachable!(),
                    };
                }
            }

            for idx in if_indices {
                let goto_false_tgt = prog.commands.len();
                if let ir::Command::If {
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

        prog.commands.push(ir::Command::End);

        Ok(prog)
    }
}
