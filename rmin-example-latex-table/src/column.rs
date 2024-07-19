use crate::{fmt, Data, Display};
#[derive(Default)]
pub struct Columns {
    pub columns: Vec<Column>,
    pub names: Vec<String>,
    pub verts: Vec<usize>,
}
impl Columns {
    pub fn header(s: &str) -> Columns {
        let mut columns = Vec::new();
        let mut verts = Vec::new();
        for item in s.chars() {
            match item {
                '|' => verts.push(columns.len()),
                'L' => columns.push(Column::new_align(ColumnAlign::Left)),
                'l' => columns.push(Column::new_align(ColumnAlign::LeftAlignWithRightElementPadding)),
                'r' => columns.push(Column::new_align(ColumnAlign::Right)),
                '%' => {
                    columns.last_mut().map(|x| x.ifn.as_percentage = true);
                }
                ch @ '0'..='9' => {
                    columns
                        .last_mut()
                        .map(|x| x.ifn.rounding = if x.ifn.rounding<0 {(ch as u8 - b'0') as i32} else {x.ifn.rounding * 10 + (ch as u8 - b'0') as i32});
                }
                _ => columns.push(Column::new_align(ColumnAlign::Center)), /*any other characters regarded as 'c'*/
            }
        }
        let names = vec![String::new(); columns.len()];
        Self {
            columns,
            names,
            verts,
        }
    }
    pub fn set_names(&mut self, names: impl Into<Vec<String>>) {
        self.names = names.into()
    }
    pub fn set_roundings(&mut self, rounding: impl Into<i32>) {
        let rounding = rounding.into();
        for i in &mut self.columns {
            i.ifn.rounding = rounding;
        }
    }
    pub fn apply_default_roundings(&mut self, rounding: impl Into<i32>) {
        let rounding = rounding.into();
        for i in &mut self.columns {
            if i.ifn.rounding < 0 {
                i.ifn.rounding = rounding;
            }
        }
    }
    pub fn apply_roundings(&mut self, roundings: impl Iterator<Item:Into<i32>>) {
        self
            .columns
            .iter_mut()
            .zip(roundings)
            .for_each(|(x, y)| x.ifn.rounding = y.into())
    }
    pub fn set_align(&mut self, index: impl Into<usize>, align: impl Into<ColumnAlign>) {
        self.columns[index.into()].align = align.into()
    }
    pub fn set_rounding(&mut self, index: impl Into<usize>, rounding: impl Into<i32>) {
        self.columns[index.into()].ifn.rounding = rounding.into()
    }
    pub fn set_as_percentage(&mut self, index: impl Into<usize>, as_percentage: impl Into<bool>) {
        self.columns[index.into()].ifn.as_percentage = as_percentage.into()
    }
    pub fn set_name(&mut self, index: impl Into<usize>, name: impl Into<String>) {
        self.names[index.into()] = name.into()
    }
}

impl Display for Columns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut verts = 0;
        for (n, i) in self.columns.iter().enumerate() {
            while let Some(t) = self.verts.get(verts).copied() {
                if t <= n {
                    verts += 1;
                    write!(f, "|")?
                } else {
                    break;
                }
            }
            i.fmt(f)?
        }
        for _ in verts..self.verts.len() {
            write!(f, "|")?
        }
        Ok(())
    }
}

/// Column, for both colname and data column.
#[derive(Default)]
pub struct Column {
    /// content align
    pub align: ColumnAlign,
    /// available for data column only.
    pub ifn: ItemFn,
}

impl Column {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new_align(align: ColumnAlign) -> Self {
        let mut s = Self {
            align: align,
            ..Default::default()
        };
        s.ifn.rounding = -1;
        s
    }
}
impl Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.align {
                ColumnAlign::Left => "l",
                ColumnAlign::LeftAlignWithRightElementPadding => "l",
                ColumnAlign::Center => "c",
                ColumnAlign::Right => "r",
            }
        )
    }
}
#[derive(Default, PartialEq)]
pub enum ColumnAlign {
    /// Leftmost, L
    Left,
    /// normal left, l
    /// for template row name with mutiple rows, padding "name" with "ll|" into "& names &" rather than "names &&"
    LeftAlignWithRightElementPadding, 
    #[default]
    Center,
    Right,
}
#[derive(Default)]
pub struct ItemFn {
    pub rounding: i32,
    pub as_percentage: bool,
}
impl ItemFn {
    pub fn fmt(&self, mut item: Data<f64>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.as_percentage {
            item.rounding = (self.rounding - 2).max(0);
            item.as_percentage = true
        } else {
            item.rounding = self.rounding;
            item.as_percentage = false
        }
        write!(f, "{item}")
    }
}
