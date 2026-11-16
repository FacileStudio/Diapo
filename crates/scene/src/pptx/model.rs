//! What a markdown scene holds, as a slide needs it: text blocks, tables and pictures.

/// A stretch of text in one style.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
}

/// One block of a scene body, in source order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Block {
    /// A heading of level 1 to 6.
    Heading(u8, Vec<Run>),
    /// A paragraph.
    Paragraph(Vec<Run>),
    /// A list item at a nesting level from 0, numbered or bulleted.
    Item { level: u8, ordered: bool, runs: Vec<Run> },
    /// A quoted paragraph.
    Quote(Vec<Run>),
    /// A code block, verbatim.
    Code(String),
    /// A table: rows of cells, the first row being the header.
    Table(Vec<Vec<Vec<Run>>>),
    /// A picture of the project, with its alternative text.
    Image { path: String, alt: String },
}

/// The text of `runs`, without its styles.
pub(crate) fn plain(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pptx::read::blocks;

    #[test]
    fn tables_pictures_quotes_and_code_are_read() {
        let body = "| A | B |\n|---|---|\n| 1 | 2 |\n\n![Chart](assets/chart.png)\n\n> Quoted\n\n```\nlet x = 1;\n```\n\n<div>raw</div>\n";
        let read = blocks(body);
        let Block::Table(rows) = &read[0] else { panic!("{read:?}") };
        assert_eq!((rows.len(), plain(&rows[0][1]), plain(&rows[1][0])), (2, "B".to_owned(), "1".to_owned()));
        assert_eq!(read[1], Block::Image { path: "assets/chart.png".into(), alt: "Chart".into() });
        assert!(matches!(&read[2], Block::Quote(runs) if plain(runs) == "Quoted"));
        assert_eq!(read[3], Block::Code("let x = 1;\n".into()));
        assert_eq!(read.len(), 4);
    }
}
