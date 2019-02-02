#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

extern crate serde;
extern crate serde_yaml;
extern crate log4rs;

mod units;

use units::UnitDatabase;
use structopt::StructOpt;


#[derive(StructOpt, Debug)]
#[structopt(
    name = "yucon",
    about = "A general purpose unit converter",
    author = "(C) 2016-2019 Blaine Murphy\nThis is free software licensed under GPL v3+. Use '--license' for more details."
)]
struct Options {
    /// Display conversions in the extended format
    #[structopt(short = "l", long = "long")]
    long_formatting: bool,

    /// Prints the license info and exits
    #[structopt(long = "license")]
    license: bool,

    /// Input value
    value: Option<f64>,
    
    /// Unit being converted from
    unit_from: Option<String>,

    /// Unit to convert into
    unit_to: Option<String>,
}

const LICENSE_MESG: &str =
r"Licensed under the GNU General Public License v3+
  Released 18 Jan 2019
  Source code available at <https://github.com/kmBlaine/yucon>
  See doc/Changelog.md for version specific details

This program is free software: you can redistribute it and/or modify it under
the terms of GPLv3 or any later version. You should have recieved a copy along
with this program. If not, see <https://gnu.org/licenses/gpl.html>.

There is NO WARRANTY, to the extent permitted by law. See full license for more
details.";

fn main() {
    let opts = Options::from_args();
    
    if opts.license {
        println!("{}", LICENSE_MESG);
        return;
    }

    if opts.value.is_some() {
        if opts.unit_from.is_none() || opts.unit_to.is_none() {
            println!("Conversion is incomplete. You must specify a value, unit, and target unit");
            return;
        }
    }
    match log4rs::init_file("../../cfg/logging.yaml", Default::default()) {
        Err(err) => {
            println!("Error loading the logging configuration: {}", err);
            return;
        },
        _ => (),
    }
    let units_db = match UnitDatabase::load_from_file("../../cfg/units.yaml".to_string(), None) {
        Some(db) => db,
        None => {
            error!("Failed to load units. Exiting");
            return;
        }
    };
    println!("{:#?}", units_db);
}
