use std::marker::PhantomData;

use crate::{fmt, Caption, Display, TableContent};

/// Latex table, contains the following parts:
///
/// ```latex
/// % use $$(var)$$ to show that, the
///	\begin{table}
///		\caption{$$(caption)$$\label{$$(label)$$}}
///     {$$(table_rules)$$}
///		\begin{tabular}{$$(content.format)$$}
///         {$$(top_rules)$$}
///         {$$(content)$$}
///         {$$(bottom_rules)$$}
///     \end{tabular}
///	\end{table}
/// ```

pub struct Table {
    pub caption: Caption,
    pub table_rules: Rules<1, Vec<String>, String>,
    pub top_rules: Rules<2, Vec<String>, String>,
    pub content: TableContent,
    pub bottom_rules: Rules<2, Vec<String>, String>,
}
impl Default for Table {
    fn default() -> Self {
        Self {
            top_rules: Rules(vec![r#"\toprules"#.to_string()], PhantomData),
            bottom_rules: Rules(vec![r#"\bottomrules"#.to_string()], PhantomData),
            caption: Default::default(),
            table_rules: Rules(Default::default(), PhantomData),
            content: Default::default(),
        }
    }
}
impl Table {
    fn new(table_rules: Vec<String>) -> Self {
        Self {
            table_rules: Rules::new(table_rules),
            top_rules: Rules::new(vec![r#"\toprules"#.to_string()]),
            bottom_rules: Rules::new(vec![r#"\bottomrules"#.to_string()]),
            caption: Default::default(),
            content: Default::default(),
        }
    }
}
impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"\begin{{table}}{caption}{table_rules}
    \begin{{tabular}}{{{format}}}{top_rules}
{content}{bottom_rules}
    \end{{tabular}}
\end{{table}}"#,
            caption = Rules::<1, [&Caption; 1], &Caption>::new([&self.caption]),
            table_rules = self.table_rules,
            format = self.content.format(),
            content = self.content,
            top_rules = self.top_rules,
            bottom_rules = self.bottom_rules
        )
    }
}
pub struct Rules<const IDENT: usize, T, V>(T, PhantomData<V>)
where
    for<'a> &'a T: IntoIterator<Item = &'a V>,
    for<'a> &'a V: Display;
impl<const IDENT: usize, T, V> Rules<IDENT, T, V>
where
    for<'a> &'a T: IntoIterator<Item = &'a V>,
    for<'a> &'a V: Display
{
    fn new(t: T) -> Self {
        Self(t, PhantomData)
    }
}
impl<const IDENT: usize, T, V> Display for Rules<IDENT, T, V>
where
    for<'a> &'a T: IntoIterator<Item = &'a V>,
    for<'a> &'a V: Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.0.into_iter() {
            write!(f, "\n{nothing:ident$}{i}", nothing = "", ident = 4 * IDENT)?
        }
        Ok(())
    }
}
