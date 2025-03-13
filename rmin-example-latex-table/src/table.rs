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
///     {$$(footnotes)$$}
///	\end{table}
/// ```

pub struct Table {
    pub caption: Caption,
    pub table_rules: Rules<1, Vec<String>, String>,
    pub top_rules: Rules<2, Vec<String>, String>,
    pub content: TableContent,
    pub bottom_rules: Rules<2, Vec<String>, String>,
    pub footnotes: Rules<1, Vec<String>, String>,
}
impl Default for Table {
    fn default() -> Self {
        Self {
            top_rules: Rules(vec![r#"\toprules"#.to_string()], PhantomData),
            bottom_rules: Rules(vec![r#"\bottomrules"#.to_string()], PhantomData),
            caption: Default::default(),
            table_rules: Rules(Default::default(), PhantomData),
            content: Default::default(),
            footnotes: Rules(Default::default(), PhantomData),
        }
    }
}
impl Table {
    pub fn new(table_rules: Vec<String>) -> Self {
        Self {
            table_rules: Rules::new(table_rules),
            top_rules: Rules::new(vec![r#"\toprules"#.to_string()]),
            bottom_rules: Rules::new(vec![r#"\bottomrules"#.to_string()]),
            caption: Default::default(),
            content: Default::default(),
            footnotes: Rules(Default::default(), PhantomData),
        }
    }
    /// Add functions
    pub fn add_table_rule(&mut self, rule: impl Into<String>) {
        self.table_rules.0.push(rule.into())
    }
    /// Add functions
    pub fn add_top_rule(&mut self, rule: impl Into<String>) {
        self.top_rules.0.push(rule.into())
    }
    /// Add functions
    pub fn add_bottom_rule(&mut self, rule: impl Into<String>) {
        self.bottom_rules.0.push(rule.into())
    }
    /// Add functions
    pub fn add_footnote(&mut self, rule: impl Into<String>) {
        self.footnotes.0.push(rule.into())
    }
    /// Set functions
    pub fn set_caption(&mut self, caption: impl Into<String>) {
        self.caption.cap = Some(caption.into())
    }
    /// Set functions
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.caption.label = Some(label.into())
    }
    /// Set functions
    pub fn set_table_rules(&mut self, rules: impl Into<Vec<String>>) {
        self.table_rules.0 = rules.into()
    }
    /// Set functions
    pub fn set_top_rules(&mut self, rules: impl Into<Vec<String>>) {
        self.top_rules.0 = rules.into()
    }
    /// Set functions
    pub fn set_bottom_rules(&mut self, rules: impl Into<Vec<String>>) {
        self.bottom_rules.0 = rules.into()
    }
    /// Set functions
    pub fn set_footnotes(&mut self, rules: impl Into<Vec<String>>) {
        self.footnotes.0 = rules.into()
    }
    /// delete functions
    pub fn remove_caption(&mut self) {
        self.caption.cap = None
    }
    /// delete functions
    pub fn remove_label(&mut self) {
        self.caption.label = None
    }
}
impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"\begin{{{table}}}{caption}{table_rules}
    \begin{{tabular}}{{{format}}}{top_rules}{content}{bottom_rules}
    \end{{tabular}}{footnotes}
\end{{{table}}}"#,
            caption = Rules::<1, Vec<&Caption>, &Caption>::new(
                if self
                    .caption
                    .cap
                    .as_ref()
                    .or(self.caption.label.as_ref())
                    .is_some()
                {
                    vec![&self.caption]
                } else {
                    vec![]
                }
            ),
            table_rules = self.table_rules,
            format = self.content.format(),
            content = self.content,
            top_rules = self.top_rules,
            bottom_rules = self.bottom_rules,
            footnotes = self.footnotes,
            table = if let Some(true) = self.content.data.get(0).map(|x|x.use_threeparttable) {"threeparttable"} else {"table"}
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
    for<'a> &'a V: Display,
{
    pub fn new(t: T) -> Self {
        Self(t, PhantomData)
    }
}
impl<const IDENT: usize, T, V> Display for Rules<IDENT, T, V>
where
    for<'a> &'a T: IntoIterator<Item = &'a V>,
    for<'a> &'a V: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.0.into_iter() {
            write!(f, "\n{nothing:ident$}{i}", nothing = "", ident = 4 * IDENT)?
        }
        Ok(())
    }
}
