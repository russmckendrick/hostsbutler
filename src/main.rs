use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;

use hostsbutler::app::App;
use hostsbutler::backup::BackupManager;
use hostsbutler::commands::file_cmds;
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
    #[arg(long, conflicts_with = "import")]
    export: Option<PathBuf>,

    /// Import entries from file (JSON, CSV, or hosts format)
    #[arg(long, conflicts_with = "export")]
    import: Option<PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install().ok();

    let cli = Cli::parse();
    let platform = detect_platform();
    let backup_manager = BackupManager::new(&platform.config_dir());

    let hosts_path = cli.file.unwrap_or_else(|| platform.hosts_path());
    let content = file_cmds::read_hosts_content(&hosts_path, platform.as_ref())?;

    let mut hosts = parser::parse_hosts_file(&content, hosts_path);

    // Handle export subcommand
    if let Some(export_path) = cli.export {
        match export_path.extension().and_then(|e| e.to_str()) {
            Some("json") => file_cmds::export_json(&hosts, &export_path)?,
            Some("csv") => file_cmds::export_csv(&hosts, &export_path)?,
            _ => file_cmds::export_hosts(&hosts, &export_path)?,
        }
        println!("Exported to {}", export_path.display());
        return Ok(());
    }

    if let Some(import_path) = cli.import {
        if cli.readonly {
            bail!("--readonly cannot be used with --import");
        }

        let imported = file_cmds::import_file(&mut hosts, &import_path)?;
        let result = file_cmds::persist_hosts_with_actions(
            &hosts,
            platform.as_ref(),
            &backup_manager,
            |content| {
                platform.write_hosts(content)?;
                Ok(())
            },
            || {
                platform.flush_dns()?;
                Ok(())
            },
        )?;

        if let Some(warning) = result.backup_warning {
            eprintln!("{warning}");
        }
        if let Some(warning) = result.dns_flush_warning {
            eprintln!("{warning}");
        }

        println!(
            "Imported {} entries from {} into {}",
            imported,
            import_path.display(),
            hosts.path.display()
        );
        return Ok(());
    }

    // Launch TUI
    let mut terminal = tui::init()?;
    let events = EventHandler::new(250);
    let mut app = App::new(hosts, cli.readonly);

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
