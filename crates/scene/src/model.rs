/// A parsed, valid project: its configuration and its scenes in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub(crate) config: Config,
    pub(crate) scenes: Vec<Scene>,
}

impl Project {
    /// The headmatter.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The scenes, in source order. Never empty.
    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    /// The length of the whole project in seconds: the sum of its scenes.
    pub fn duration(&self) -> f64 {
        self.scenes.iter().map(|scene| scene.duration).sum()
    }

    /// The last global step: every scene counts one stop, plus one per step.
    pub fn steps(&self) -> u32 {
        self.scenes.iter().map(|scene| scene.steps + 1).sum::<u32>() - 1
    }
}

/// The project configuration, from the headmatter.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Config {
    /// `title`: the document title. Empty when absent.
    pub title: String,
    /// `theme`.
    pub theme: Theme,
    /// `scheme`: the colors of the theme.
    pub scheme: Scheme,
    /// `size`: the stage in CSS pixels.
    pub size: Size,
    /// `step_duration`: the seconds one step lasts in a scene with no `duration`.
    pub step_duration: f64,
    /// `lang`: the language tag of the compiled document.
    pub lang: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: String::new(),
            theme: Theme::Facile,
            scheme: Scheme::Light,
            size: Size { width: 1920, height: 1080 },
            step_duration: 4.0,
            lang: "en".to_owned(),
        }
    }
}

/// The stage size in CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// The width.
    pub width: u32,
    /// The height.
    pub height: u32,
}

/// A theme: the stylesheet every scene of a project shares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Theme {
    /// The suite theme.
    Facile,
}

/// The colors a theme is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scheme {
    /// Dark text on a light stage.
    Light,
    /// Light text on a dark stage.
    Dark,
    /// Whichever the viewer's system prefers.
    Auto,
}

impl Scheme {
    /// The name as the source and the compiled document spell it.
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
            Scheme::Auto => "auto",
        }
    }
}

/// How a scene lays its body out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Layout {
    /// Title and content from the top.
    Default,
    /// A title scene, set large at the bottom left.
    Cover,
    /// Everything centered.
    Center,
    /// Text on the left, pictures and tables on the right.
    Split,
    /// The body is an HTML composition, with no markdown and no wrapper.
    Raw,
}

impl Layout {
    /// The name as the source and the compiled document spell it.
    pub fn name(self) -> &'static str {
        match self {
            Layout::Default => "default",
            Layout::Cover => "cover",
            Layout::Center => "center",
            Layout::Split => "split",
            Layout::Raw => "raw",
        }
    }
}

/// One scene: a slide of a deck, or a shot of a video.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Scene {
    /// `layout`.
    pub layout: Layout,
    /// The number of steps: `steps`, or the highest step the body uses.
    pub steps: u32,
    /// The length in seconds: `duration`, or the end of the last clip, or `step_duration` for
    /// each of the `steps + 1` stops.
    pub duration: f64,
    /// `class`: extra class names for the scene element.
    pub class: String,
    /// The body as written, without its frontmatter and its presenter note.
    pub body: String,
    /// The presenter note: the text of the trailing HTML comment.
    pub note: Option<String>,
    /// The 1-based source line the body starts on.
    pub line: usize,
    pub(crate) html: String,
}
