use crate::{fmt, Display, Data};
#[derive(Default)]
pub struct Rows {
    pub rows:Vec<Row>,
    pub verts:Vec<usize>
}
impl Rows {
    pub fn header(s:&str)->Rows{
        let mut rows = Vec::new();
        let mut verts = Vec::new();
        for item in s.chars() {
            match item {
                '|' => verts.push(rows.len()),
                'l' => rows.push(Row::new_align(RowAlign::Left)),
                'r' => rows.push(Row::new_align(RowAlign::Right)),
                _ => rows.push(Row::new_align(RowAlign::Center)) /*any other characters regarded as 'c'*/
            }
        }
        Self{rows, verts}
    }
}


impl Display for Rows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut verts=0;
        for (n,i) in self.rows.iter().enumerate() {
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
    pub fn new()->Self {
        Default::default()
    }
    pub fn new_align(align:RowAlign)->Self{
        Self {
            align:align,
            ..Default::default()
        }
    }
}
impl Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}", match self.align {
            RowAlign::Left=>"l",
            RowAlign::Center=>"c",
            RowAlign::Right=>"r",
        })
    }
}
#[derive(Default)]
pub enum RowAlign {
    Left,
    #[default]
    Center,
    Right
}
#[derive(Default)]
pub struct ItemFn {
    rounding: usize,
    as_percentage: bool,
}
impl ItemFn {
    pub fn fmt(&self, mut item:Data<f64>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.as_percentage {
            item.rounding = (self.rounding as isize-2).max(0) as usize;
            item.as_percentage = true
        } else {
            item.rounding = self.rounding;
            item.as_percentage = false
        }
        write!(f, "{item}")
    }
}
fn main(){}
