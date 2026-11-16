use std::fmt;

/// One problem in a source file, at a 1-based line.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("line {line}: {kind}")]
pub struct Error {
    /// The 1-based line of the source the problem is on.
    pub line: usize,
    /// What is wrong.
    pub kind: ErrorKind,
}

/// What can be wrong with a source file.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The file holds no scene.
    #[error("the project has no scene")]
    Empty,
    /// A frontmatter block has no closing `---`.
    #[error("the frontmatter opened here is never closed by `---`")]
    UnclosedFrontmatter,
    /// A line of a frontmatter block is not `key: value`.
    #[error("a frontmatter line must be `key: value`")]
    BadFrontmatterLine,
    /// A key the format does not define.
    #[error("unknown key `{key}`")]
    UnknownKey {
        /// The key as written.
        key: String,
    },
    /// A key written twice in one block.
    #[error("duplicate key `{key}`")]
    DuplicateKey {
        /// The key as written.
        key: String,
    },
    /// A project key in a scene's frontmatter.
    #[error("`{key}` belongs to the headmatter, not to a scene")]
    ProjectKeyInScene {
        /// The key as written.
        key: String,
    },
    /// A value of the wrong shape.
    #[error("`{key}` must be {expected}")]
    BadValue {
        /// The key, or the HTML attribute, the value belongs to.
        key: String,
        /// What the value must be.
        expected: &'static str,
    },
    /// A `{step=` marker that is not `{step=k}` with `k` from 1 up.
    #[error("a step marker must be `{{step=k}}` with k from 1 up")]
    BadStepMarker,
    /// A step marker outside a paragraph, heading, list item or quote.
    #[error("a step marker must sit in a paragraph, heading, list item or quote; elsewhere use `data-step`")]
    StepMarkerOutsideBlock,
    /// A step past the scene's declared `steps`.
    #[error("step {step} is past `steps: {steps}`")]
    StepOutOfRange {
        /// The highest step the scene uses.
        step: u32,
        /// The count the frontmatter declares.
        steps: u32,
    },
    /// A resource loaded from the network.
    #[error("remote URL `{url}`: a scene loads only files of its project")]
    RemoteUrl {
        /// The URL as written.
        url: String,
    },
    /// An asset path that is absolute or leaves the project folder.
    #[error("asset path `{path}` must be relative and stay inside the project folder")]
    BadAssetPath {
        /// The path as written.
        path: String,
    },
}

/// Every problem of one source file, in line order. Never empty.
#[derive(Debug, Clone, PartialEq)]
pub struct Errors(pub Vec<Error>);

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.0.iter().map(Error::to_string).collect();
        f.write_str(&lines.join("\n"))
    }
}

impl std::error::Error for Errors {}
