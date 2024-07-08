use crate::{fmt, Data, Display};
#[derive(Default)]
pub struct Rows {
    pub rows: Vec<Row>,
    pub names: Vec<String>,
    pub verts: Vec<usize>,
}
impl Rows {
    pub fn header(s: &str) -> Rows {
        let mut rows = Vec::new();
        let mut verts = Vec::new();
        for item in s.chars() {
            match item {
                '|' => verts.push(rows.len()),
                'l' => rows.push(Row::new_align(RowAlign::Left)),
                'r' => rows.push(Row::new_align(RowAlign::Right)),
                '%' => {
                    rows.last_mut().map(|x| x.ifn.as_percentage = true);
                }
                ch @ '0'..='9' => {
                    rows.last_mut()
                        .map(|x| x.ifn.rounding = x.ifn.rounding * 10 + (ch as u8 - b'0') as u32);
                }
                _ => rows.push(Row::new_align(RowAlign::Center)), /*any other characters regarded as 'c'*/
            }
        }
        let names = vec![String::new(); rows.len()];
        Self { rows, names, verts }
    }
    pub fn set_names(&mut self, names: impl Into<Vec<String>>) {
        self.names = names.into()
    }
    pub fn set_roundings(&mut self, rounding: impl Into<u32>) {
        let rounding = rounding.into();
        for i in &mut self.rows {
            i.ifn.rounding = rounding;
        }
    }
    pub fn set_align(&mut self, index: impl Into<usize>, align: impl Into<RowAlign>) {
        self.rows[index.into()].align = align.into()
    }
    pub fn set_rounding(&mut self, index: impl Into<usize>, rounding: impl Into<u32>) {
        self.rows[index.into()].ifn.rounding = rounding.into()
    }
    pub fn set_as_percentage(&mut self, index: impl Into<usize>, as_percentage: impl Into<bool>) {
        self.rows[index.into()].ifn.as_percentage = as_percentage.into()
    }
    pub fn set_name(&mut self, index: impl Into<usize>, name: impl Into<String>) {
        self.names[index.into()] = name.into()
    }
}

impl Display for Rows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut verts = 0;
        for (n, i) in self.rows.iter().enumerate() {
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

/// Row, for both colname and data row.
#[derive(Default)]
pub struct Row {
    /// content align
    pub align: RowAlign,
    /// available for data row only.
    pub ifn: ItemFn,
}

impl Row {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new_align(align: RowAlign) -> Self {
        Self {
            align: align,
            ..Default::default()
        }
    }
}
impl Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.align {
                RowAlign::Left => "l",
                RowAlign::Center => "c",
                RowAlign::Right => "r",
            }
        )
    }
}
#[derive(Default)]
pub enum RowAlign {
    Left,
    #[default]
    Center,
    Right,
}
#[derive(Default)]
pub struct ItemFn {
    pub rounding: u32,
    pub as_percentage: bool,
}
impl ItemFn {
    pub fn fmt(&self, mut item: Data<f64>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.as_percentage {
            item.rounding = (self.rounding as i32 - 2).max(0) as u32;
            item.as_percentage = true
        } else {
            item.rounding = self.rounding;
            item.as_percentage = false
        }
        write!(f, "{item}")
    }
}
