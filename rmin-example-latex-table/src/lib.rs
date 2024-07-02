//use rmin::*;
use core::fmt::{self, Display};
use std::collections::HashMap;
/// Latex table, contains the following parts:
/// 
///	\begin{{table}}
///		\caption{{{caption}\label{{{label}}}}}		
///		\begin{{tabular}}{{{format}}}
///                 \toprule
///                 {content}
///                 \bottomrule
///     	\end{{tabular}}
///	\end{{table}}

pub struct Table {
    pub caption: Caption,
    pub format: Format,
    pub content: TableContent,
}
impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"\begin{{table}}
    {caption}
    \begin{{tabular}}{{{format}}}
        \toprule
        {content}
        \buttomrule
    \end{{tabular}}
\end{{table}}"#, caption = self.caption, format = self.format, content = self.content)
    }
}
pub struct Caption {
    pub cap: Option<String>,
    pub label: Option<String>
}
impl Display for Caption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.cap.as_ref(), self.label.as_ref()) {
            (Some(c), Some(l)) => write!(f,r#"\caption{{{c}\label{{{l}}}}}"#),
            (Some(c), None)    => write!(f,r#"\caption{{{c}}}"#),
            (None, Some(l))    => write!(f,r#"\label{{{l}}}"#),
            (None, None)       => Ok(())
        }
    }
}
pub struct Format {
    pub data:Vec<Row>,
    pub row:Vec<Row>,
}
impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in self.data.iter().chain(self.row.iter()) {
            write!(f, "{r}")?
        }
        Ok(())
    }
}
pub enum Row {
    Left(ExtraFormat),
    Center(ExtraFormat),
    Right(ExtraFormat),
    LeftV(ExtraFormat),
    CenterV(ExtraFormat),
    RightV(ExtraFormat),
    VLeftV(ExtraFormat),
    VCenterV(ExtraFormat),
    VRightV(ExtraFormat),
}
impl Row {
    fn l()->Self{Row::Left(Default::default())}
    fn c()->Self{Row::Center(Default::default())}
    fn r()->Self{Row::Right(Default::default())}
    fn lv()->Self{Row::LeftV(Default::default())}
    fn cv()->Self{Row::CenterV(Default::default())}
    fn rv()->Self{Row::RightV(Default::default())}
}
#[derive(Default)]
pub struct ExtraFormat;

impl Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}", match self {
            Row::Left(_)=>"l",
            Row::Center(_)=>"c",
            Row::Right(_)=>"r",
            Row::LeftV(_)=>"l|",
            Row::CenterV(_)=>"c|",
            Row::RightV(_)=>"r|",
            Row::VLeftV(_)=>"|l|",
            Row::VCenterV(_)=>"|c|",
            Row::VRightV(_)=>"|r|",
        })
    }
}
pub struct TableContent{}
impl Display for TableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
fn main(){
    let a = Table {caption:Caption{cap:Some("fine".to_string()), label:Some("t1".to_string())}, format:Format{data:vec![Row::l(),Row::lv()], row:vec![Row::c(), Row::c(),Row::cv(),Row::c(), Row::c(), Row::c()]}, content:TableContent{}};
    println!("{a}")
}
