// This file is a part of Redox OS games, which is distributed under terms of
// MIT license.
//
//     Copyright (c) 2018 Árni Dagur <arni@dagur.eu>
//
// vim: et:ts=4:sts=4:sw=4
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate chrono;
use chrono::{Local, DateTime, TimeZone, Utc};
use chrono::offset;

use std::env;
use std::process;
use std::io::{self, Write};
use std::f64::consts::PI;

/// Phase of the moon. Calculates the current phase of the moon based on
/// routines from 'Practical Astronomy with Your Calculator or Spreadsheet' by
/// Duffet-Smith. The 4th edition of the book is available online for free at
/// https://archive.org/
//  Comments give the section from the book that particular piece of code was
//  adapted from.

const HELP: &'static str = r#"pom ~ Phase of the Moon

flags:
    -h  | --help       ~ this help message.
    -dt | --datetime   ~ specify datetime in "YY-MM-DD HH:MM:SS" format
    -p  | --percentage ~ only print the percentage

author:
    Árni Dagur <arni@dagur.eu>
"#;

// We define an epoch on which we shall base our calculations; here it is
// 2010 January 0.0, equivilent to the midnight between 30. and 31. december of
// 2009 (see section 3 for details).
const EPSILON_g: f64 = 279.447208f64; // The Sun's mean ecliptic long at epoch.
const RHO_g: f64 = 283.112438f64; // The longitude of the Sun at perigee.
const ECC: f64 = 0.016705f64; // Eccintricity of the Sun-Earth orbit.
const FRAC_360_TROP_YEAR: f64 = 0.9856473563866f64; // 360 divided by 365.242191

const L_0: f64 = 91.929335f64; // Moon's mean longitude at the epoch
const P_0: f64 = 130.143076f64; // Moon's mean longitude of the perigee at epoch

/// Calculate the phase of the moon given a certain number of days away from the
/// epoch January 2010.
fn potm(days: f64) -> f64 {
    //             Section 46: Calculating the position of the sun
    let n = adj360(FRAC_360_TROP_YEAR * days);
    // We calulate:
    //     (a) The true solar anomoly in an ellipse
    let M_sol = adj360(n + EPSILON_g - RHO_g);
    //     (b) The longitude of the sun
    let Lambda_sol = adj360(n + 360.0 / PI * ECC
                       * M_sol.to_radians().sin() + EPSILON_g);

    //             Section 65: Calculating the Moon's position
    // TODO: Switch to more accurate MoonPos2 model instead of MoonPos1
    // We calculate:
    //     (a) the Sun's ecliptic longitude Lambda_sol, and mean anomaly M_sol,
    //          by the method given in section 46. (Done above)
    //     (b) the Moon's mean longitude, l
    let l = adj360(13.1763966f64 * days + L_0);
    //     (c) the Moon's mean anomaly, M_m
    let M_m = adj360(l - 0.1114041f64 * days - P_0);
    // Next we calculate the corrections for:
    //     (a) Evection
    let E_v = 1.2739 * (2.0 * (l - Lambda_sol) - M_m).to_radians().sin();
    //     (b) The annual equation
    let A_e = 0.1858 * M_sol.to_radians().sin();
    //     (c) And a 'third' correction
    let A_3 = 0.37 * M_sol.to_radians().sin();
    // Applying these corrections gives the Moon's corrected anomaly
    let M_m_prime = M_m - E_v - A_e - A_3;
    // Correction for the equation of the centre:
    let E_c = 6.2886 * M_m_prime.to_radians().sin();
    // Another correction term must be calculated:
    let A_4 = 0.214 * (2.0 * M_m_prime).to_radians().sin();
    // We can now find the Moon's corrected longitude, l_prime
    let l_prime = l + E_v + E_c - A_e + A_4;
    // The final correction to apply to the Moon's longitude is the variation
    let V = 0.6583 * (2.0 * (l_prime - Lambda_sol)).to_radians().sin();
    // So the Moon's true orbital longitude is:
    let l_2prime = l_prime + V;

    //             Section 67: The phases of the Moon
    // Calculate the 'age' of the moon.
    let D = l_2prime - Lambda_sol;
    // The Moon's phase, F, on the scale from 0 to 100, is given by:
    50.0 * (1.0 - D.to_radians().cos())
}

/// Adjusts value so 0 <= deg <= 360
fn adj360(mut deg: f64) -> f64 {
    loop {
        if deg < 0.0 {
            deg += 360.0;
        } else if deg > 360.0 {
            deg -= 360.0;
        } else {
            break;
        }
    }
    deg
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    let epoch: DateTime<Utc> = offset::Utc.ymd(2009, 12, 31).and_hms(0, 0, 0);
    let mut datetime: DateTime<Local> = Local::now();

    let mut args = env::args().skip(1);
    let mut percentage_only = false;
    loop {
        // Read the arguments.
        let arg = if let Some(x) = args.next() {
            x
        } else {
            break;
        };

        match arg.as_str() {
            "-h" | "--help" => {
                stdout.write(HELP.as_bytes()).unwrap();
                stdout.flush().unwrap();
                process::exit(0);
            },
            "-dt" | "--datetime" => {
                datetime = offset::Local.datetime_from_str(
                    &args.next().unwrap_or_else(|| {
                        stderr.write(b"No datetime given.\n").unwrap();
                        stderr.flush().unwrap();
                        process::exit(1);
                    }),
                    &"%Y-%m-%d %H:%M:%S"
                ).unwrap_or_else(|_| {
                    stderr.write(b"Invalid datetime given.\n").unwrap();
                    stderr.flush().unwrap();
                    process::exit(1);
                });
            },
            "-p" | "--percentage" => {
                percentage_only = true;
            },
            _ => {
                stderr.write(b"Unknown argument.\n").unwrap();
                stderr.flush().unwrap();
                process::exit(1);
            }
        }
    }

    let seconds = datetime.signed_duration_since(epoch).num_seconds();
    let days = seconds as f64 / 86400.0;
    let today = potm(days);

    if percentage_only {
        println!("{:.2}", today);
        process::exit(0);
    }

    stdout.write(b"The Moon is").unwrap();
    if today.round() == 100.0 {
        stdout.write(b"Full\n").unwrap();
    } else if today.round() == 0.0 {
        stdout.write(b"New\n").unwrap();
    } else {
        let tomorrow = potm(days + 1.0);
        if today.round() == 50.0 {
            if tomorrow > today {
                stdout.write(b"at the First Quarter\n").unwrap();
            } else {
                stdout.write(b"at the Last Quarter\n").unwrap();
            }
        } else {
            if tomorrow > today {
                stdout.write(b" Waxing ").unwrap();
            } else {
                stdout.write(b" Waning ").unwrap();
            }
            if today > 50.0 {
                println!("Gibbous ({:.2}% of Full)", today)
            } else {
                println!("Crescent ({:.2}% of Full)", today)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_potm_on_epoch() {
        assert_eq!(potm(0.0).round(), 99.0);
    }

    #[test]
    fn test_potm_on_leap_day_2016_noon() {
        assert_eq!(potm(2251.5).round(), 64.0);
    }

    #[test]
    fn test_potm_on_13_october_2004_0530am() {
        assert_eq!(potm(-1904.770833333).round(), 1.0);
    }

    #[test]
    fn test_adj360_above_360() {
        assert!((adj360(821.0) - 101.0).abs() < 1e-10);
    }

    #[test]
    fn test_adj360_below_0() {
        assert!((adj360(-643.4) - 76.6).abs() < 1e-10);
    }

    #[test]
    fn test_adj360_between_0_and_360() {
        assert!((adj360(168.0) - 168.0).abs() < 1e-10);
    }
}
