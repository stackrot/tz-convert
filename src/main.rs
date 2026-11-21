use clap::Parser;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};
use jiff::{civil::DateTime, tz::TimeZone, Timestamp};
use prettytable::{row, Table};
use std::error::Error;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

mod time;
mod zone;

use zone::Zone;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Time to convert, in YYYY-MM-DD HH:MM format (1800–2099)
    #[arg(long, value_parser = time::parse)]
    time: Option<DateTime>,

    /// Source IANA timezone, or 'local' for the system timezone
    #[arg(long, default_value = "local", value_name = "ZONE")]
    from: Zone,

    /// Destination timezone; repeat or separate names with commas
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "ZONE",
        conflicts_with = "all"
    )]
    to: Vec<Zone>,

    /// Display the time in every available IANA timezone
    #[arg(long)]
    all: bool,

    /// List every available IANA timezone name and exit
    #[arg(long, conflicts_with_all = ["time", "from", "to", "all"])]
    list_timezones: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error)
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe) =>
        {
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.list_timezones {
        let mut output = io::stdout().lock();
        for zone in zone::all() {
            writeln!(output, "{zone}")?;
        }
        return Ok(());
    }

    let time = match args.time {
        Some(time) => args.from.resolve(time)?,
        None => Timestamp::now(),
    };
    let destinations = if args.all {
        zone::all()
    } else if args.to.is_empty() {
        vec![Zone::Named(TimeZone::UTC)]
    } else {
        args.to
    };

    display(time, &args.from, &destinations, args.time.is_some())?;

    if std::env::args_os().len() == 1
        && io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && io::stderr().is_terminal()
    {
        interactive()?;
    }

    Ok(())
}

fn display(
    time: Timestamp,
    source: &Zone,
    destinations: &[Zone],
    converted: bool,
) -> io::Result<()> {
    let heading = if converted {
        "Converted times"
    } else {
        "Current times"
    };
    let mut output = io::stdout().lock();
    writeln!(output, "{} (source: {source}):", heading.bold().cyan())?;

    let mut table = Table::new();
    table.add_row(row!["Timezone", "Time", "UTC offset"]);
    let mut displayed = Vec::new();

    for zone in std::iter::once(source).chain(destinations) {
        if displayed.contains(&zone) {
            continue;
        }
        displayed.push(zone);
        table.add_row(row![
            zone.to_string(),
            zone.format(time, "%Y-%m-%d %H:%M:%S %Z"),
            zone.format(time, "%::z")
        ]);
    }

    table.print(&mut output)?;
    Ok(())
}

fn select_zone(prompt: &str, default: Zone) -> Result<Option<Zone>, dialoguer::Error> {
    let mut zones = vec![Zone::Local];
    zones.extend(zone::all());
    let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&zones)
        .default(zones.iter().position(|zone| *zone == default).unwrap_or(0))
        .max_length(12)
        .interact_opt()?;
    Ok(selected.map(|index| zones.swap_remove(index)))
}

fn interactive() -> Result<(), Box<dyn Error>> {
    if Confirm::new()
        .with_prompt("Convert a time?")
        .default(false)
        .interact_opt()?
        != Some(true)
    {
        return Ok(());
    }

    let Some(source) = select_zone("Source timezone (type to search)", Zone::Local)? else {
        return Ok(());
    };
    let input: String = Input::new()
        .with_prompt(format!("Time in {source} (YYYY-MM-DD HH:MM)"))
        .default(source.format(Timestamp::now(), "%Y-%m-%d %H:%M"))
        .validate_with(|value: &String| {
            time::parse(value)
                .and_then(|time| source.resolve(time))
                .map(|_| ())
        })
        .interact_text()?;
    let time = source.resolve(time::parse(&input)?)?;
    let destinations = select_destinations()?;

    if !destinations.is_empty() {
        display(time, &source, &destinations, true)?;
    }
    Ok(())
}

fn select_destinations() -> Result<Vec<Zone>, dialoguer::Error> {
    let mut destinations = Vec::new();

    while let Some(zone) = select_zone(
        "Destination timezone (type to search)",
        Zone::Named(TimeZone::UTC),
    )? {
        if !destinations.contains(&zone) {
            destinations.push(zone);
        }
        if Confirm::new()
            .with_prompt("Add another destination?")
            .default(false)
            .interact_opt()?
            != Some(true)
        {
            break;
        }
    }

    Ok(destinations)
}
