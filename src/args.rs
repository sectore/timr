use clap::Parser;
use color_eyre::{
    eyre::{ensure, eyre},
    Report,
};
use std::time::Duration;

use crate::common::{Content, Style};

#[derive(Parser)]
#[command(version)]
pub struct Args {
    #[arg(long, short, value_parser = parse_duration,
        help = "Countdown time to start from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss'"
    )]
    pub countdown: Option<Duration>,

    #[arg(long, short, value_parser = parse_duration,
        help = "Work time to count down from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss'"
    )]
    pub work: Option<Duration>,

    #[arg(long, short, value_parser = parse_duration,
        help = "Pause time to count down from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss'"
    )]
    pub pause: Option<Duration>,

    #[arg(long, short = 'd', help = "Whether to show deciseconds or not.")]
    pub decis: bool,

    #[arg(long, short = 'm', value_enum, help = "Mode to start with.")]
    pub mode: Option<Content>,

    #[arg(long, short = 's', value_enum, help = "Style to display time with.")]
    pub style: Option<Style>,

    #[arg(long, value_enum, help = "Whether to open the menu or not.")]
    pub menu: bool,

    #[arg(long, short = 'r', help = "Reset stored values to default.")]
    pub reset: bool,
}

fn parse_duration(arg: &str) -> Result<Duration, Report> {
    let parts: Vec<&str> = arg.split(':').rev().collect();

    let parse_seconds = |s: &str| -> Result<u64, Report> {
        let secs = s.parse::<u64>().map_err(|_| eyre!("Invalid seconds"))?;
        ensure!(secs < 60, "Seconds must be less than 60.");
        Ok(secs)
    };

    let parse_minutes = |m: &str| -> Result<u64, Report> {
        let mins = m.parse::<u64>().map_err(|_| eyre!("Invalid minutes"))?;
        ensure!(mins < 60, "Minutes must be less than 60.");
        Ok(mins)
    };

    let parse_hours = |h: &str| -> Result<u64, Report> {
        let hours = h.parse::<u64>().map_err(|_| eyre!("Invalid hours"))?;
        ensure!(hours < 100, "Hours must be less than 100.");
        Ok(hours)
    };

    let seconds = match parts.as_slice() {
        [ss] => parse_seconds(ss)?,
        [ss, mm] => {
            let s = parse_seconds(ss)?;
            let m = parse_minutes(mm)?;
            m * 60 + s
        }
        [ss, mm, hh] => {
            let s = parse_seconds(ss)?;
            let m = parse_minutes(mm)?;
            let h = parse_hours(hh)?;
            h * 60 * 60 + m * 60 + s
        }
        _ => return Err(eyre!("Invalid time format. Use 'ss', mm:ss, or hh:mm:ss")),
    };

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        // ss
        assert_eq!(parse_duration("50").unwrap(), Duration::from_secs(50));
        // mm:ss
        assert_eq!(
            parse_duration("01:30").unwrap(),
            Duration::from_secs(60 + 30)
        );
        // hh:mm:ss
        assert_eq!(
            parse_duration("01:30:00").unwrap(),
            Duration::from_secs(60 * 60 + 30 * 60)
        );
        // errors
        assert!(parse_duration("1:60").is_err()); // invalid seconds
        assert!(parse_duration("60:00").is_err()); // invalid minutes
        assert!(parse_duration("100:00:00").is_err()); // invalid hours
        assert!(parse_duration("abc").is_err()); // invalid input
        assert!(parse_duration("01:02:03:04").is_err()); // too many parts
    }
}
