//! The command line as clap reads it.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// The whole command line.
#[derive(Parser, Debug)]
#[command(name = "diapo", version, about = "Write, serve and sync slide decks kept as one text file")]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
    /// The flags every command takes.
    #[command(flatten)]
    pub globals: Globals,
}

/// The flags every command takes.
#[derive(Args, Debug, Default, Clone)]
pub struct Globals {
    /// `--json`.
    #[arg(long, global = true, help = "Print one JSON document instead of text")]
    pub json: bool,
    /// `--no-color`.
    #[arg(long, global = true, help = "Disable colored output")]
    pub no_color: bool,
    /// `--server`.
    #[arg(long, global = true, value_name = "url", help = "Use this Diapo server")]
    pub server: Option<String>,
}

/// Every command, in two groups: the ones that work offline and the ones that talk to a server.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// The commands that need no server.
    #[command(flatten)]
    Local(Local),
    /// The commands that talk to a Diapo server.
    #[command(flatten)]
    Remote(Remote),
}

/// The commands that need no server.
#[derive(Subcommand, Debug)]
pub enum Local {
    /// Create a folder with a starter deck
    New(NewArgs),
    /// Serve a deck locally and reload it on every change
    Serve(ServeArgs),
    /// Build a deck into one offline HTML file
    Build(BuildArgs),
    /// Validate a deck and list every problem with its line
    Check(FolderArgs),
    /// Print the format reference
    Guide,
    /// Explain where PDF, PPTX and MP4 exports come from
    Export(ExportArgs),
}

/// The commands that talk to a Diapo server.
#[derive(Subcommand, Debug)]
pub enum Remote {
    /// Sign in to a Diapo server
    Login(LoginArgs),
    /// Remove the stored login
    Logout,
    /// Show the signed-in account
    Whoami,
    /// List your decks
    List(ListArgs),
    /// Download a deck and its assets into a folder
    Pull(PullArgs),
    /// Upload a folder's deck and assets
    Push(PushArgs),
    /// Open a pulled deck in the web app
    Open(OpenArgs),
}

/// `diapo new`.
#[derive(Args, Debug)]
pub struct NewArgs {
    /// The folder to create.
    #[arg(value_name = "name", help = "Folder to create")]
    pub name: PathBuf,
}

/// A command that takes only the deck folder.
#[derive(Args, Debug)]
pub struct FolderArgs {
    /// The deck folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Deck folder or file [default: .]")]
    pub dir: Option<PathBuf>,
}

/// `diapo serve`.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// The deck folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Deck folder or file [default: .]")]
    pub dir: Option<PathBuf>,
    /// The port to listen on.
    #[arg(short, long, default_value_t = 3030, value_name = "port", help = "Port to listen on, 0 for any free port")]
    pub port: u16,
}

/// `diapo build`.
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// The deck folder, or its `.md` file.
    #[arg(value_name = "dir", help = "Deck folder or file [default: .]")]
    pub dir: Option<PathBuf>,
    /// The file to write.
    #[arg(short, long, value_name = "file", help = "File to write [default: <folder name>.html]")]
    pub output: Option<PathBuf>,
}

/// `diapo export`.
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Whatever was typed after `export`, read by nothing.
    #[arg(hide = true, trailing_var_arg = true, allow_hyphen_values = true)]
    pub rest: Vec<String>,
}

/// `diapo login`.
#[derive(Args, Debug)]
pub struct LoginArgs {
    /// The server to sign in to.
    #[arg(
        value_name = "server",
        help = "Server to sign in to [default: the stored one, or https://diapo.facile.studio]"
    )]
    pub target: Option<String>,
    /// Use the device grant even where a browser can open.
    #[arg(long, help = "Approve the login from another device instead of this machine's browser")]
    pub device: bool,
    /// Print the sign-in link and open nothing.
    #[arg(long, help = "Print the sign-in link without opening a browser")]
    pub no_browser: bool,
}

/// `diapo list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Print bare deck ids.
    #[arg(short, long, help = "Print deck ids only")]
    pub quiet: bool,
}

/// `diapo pull`.
#[derive(Args, Debug)]
pub struct PullArgs {
    /// A deck id or its web URL. Without it, the folder's own deck.
    #[arg(value_name = "deck", help = "Deck id or URL [default: the deck this folder was pulled from]")]
    pub deck: Option<String>,
    /// The folder to write into.
    #[arg(value_name = "dir", help = "Folder to write into [default: a folder named after the deck, or .]")]
    pub dir: Option<PathBuf>,
    /// Discard local changes.
    #[arg(long, help = "Overwrite local changes")]
    pub force: bool,
}

/// `diapo push`.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// The deck folder.
    #[arg(value_name = "dir", help = "Deck folder [default: .]")]
    pub dir: Option<PathBuf>,
    /// Overwrite what the server holds.
    #[arg(long, help = "Overwrite the server's copy even if it changed since the last pull")]
    pub force: bool,
    /// Create the deck when the folder is not linked to one.
    #[arg(long, help = "Create a new deck on the server from this folder")]
    pub create: bool,
}

/// `diapo open`.
#[derive(Args, Debug)]
pub struct OpenArgs {
    /// The deck folder.
    #[arg(value_name = "dir", help = "Deck folder [default: .]")]
    pub dir: Option<PathBuf>,
    /// Print the URL and open nothing.
    #[arg(long, help = "Print the URL without opening a browser")]
    pub print: bool,
}
