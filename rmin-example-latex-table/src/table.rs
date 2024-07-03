use crate::{Caption,TableContent,fmt,Display};
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
    pub content: TableContent,
}
impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"\begin{{table}}
        {caption}
        \begin{{tabular}}{{{format}}}\toprule
        {content}
        \buttomrule\end{{tabular}}
        \end{{table}}"#, caption = self.caption, format = self.content.format(), content = self.content)
    }
}
