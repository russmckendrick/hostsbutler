use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use hostsbutler::app::App;
use hostsbutler::event::{AppEvent, EventHandler};
use hostsbutler::parser;
use hostsbutler::platform::detect_platform;
use hostsbutler::tui;
use hostsbutler::ui;

#[derive(clap::Parser, Debug)]
#[command(
    name = "hostsbutler",
    version,
    about = "A TUI for managing the system hosts file"
)]
struct Cli {
    /// Path to hosts file (overrides platform default)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Read-only mode (no writes)
    #[arg(short, long)]
    readonly: bool,

    /// Export entries to file (JSON, CSV, or hosts format)
    #[arg(long)]
    export: Option<PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install().ok();

    let cli = Cli::parse();
    let platform = detect_platform();

    let hosts_path = cli.file.unwrap_or_else(|| platform.hosts_path());

    // Read hosts file
    let content = if hosts_path == platform.hosts_path() {
        platform.read_hosts()?
    } else {
        std::fs::read_to_string(&hosts_path)?
    };

    let hosts = parser::parse_hosts_file(&content, hosts_path);

    // Handle export subcommand
    if let Some(export_path) = cli.export {
        match export_path.extension().and_then(|e| e.to_str()) {
            Some("json") => hostsbutler::commands::file_cmds::export_json(&hosts, &export_path)?,
            Some("csv") => hostsbutler::commands::file_cmds::export_csv(&hosts, &export_path)?,
            _ => hostsbutler::commands::file_cmds::export_hosts(&hosts, &export_path)?,
        }
        println!("Exported to {}", export_path.display());
        return Ok(());
    }

    // Launch TUI
    let mut terminal = tui::init()?;
    let events = EventHandler::new(250);
    let mut app = App::new(hosts);

    while app.running {
        terminal.draw(|f| ui::render::render(f, &app))?;

        match events.next()? {
            AppEvent::Key(key) => {
                app.handle_key(key);
            }
            AppEvent::Tick => {
                app.clear_stale_toast();
            }
            AppEvent::Resize(_, _) => {}
        }

        if app.needs_full_redraw {
            terminal.clear()?;
            app.needs_full_redraw = false;
        }
    }

    tui::restore()?;
    Ok(())
}
