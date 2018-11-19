use runtime::convert::ConversionFmt;
use runtime::InterpretErr;
use std::env;

pub struct Options
{
    interactive: bool,
    format: ConversionFmt,
}

impl Options
{
    pub fn new() -> Options
    {
        Options {
            interactive: true,
            format: ConversionFmt::Desc,
        }
    }

    pub fn get_opts() -> Result<(Options, Vec<String>), InterpretErr>
    {
        let mut opts = Options::new();
        let mut extras = Vec::with_capacity(env::args().count());
        let mut args = env::args();
        args.next(); // skip program name

        loop
        {
            let arg = match args.next()
            {
            Some(opt) => opt,
            None => break,
            };

            if arg.starts_with("--")
            {
                match arg.as_ref()
                {
                "--help" => return Err(InterpretErr::HelpSig),
                "--version" => return Err(InterpretErr::VersionSig),
                _ => return Err(InterpretErr::UnknownLongOpt(arg)),
                };
            }
            else if arg.starts_with("-")
            {
                if arg.parse::<f64>().is_ok()
                {
                    extras.push(arg);

                    for extra in args
                    {
                        extras.push(extra);
                    }

                    if extras.len() < 3
                    {
                        return Err(InterpretErr::IncompleteErr);
                    }

                    opts.interactive = false;
                    break;
                }
                else
                {
                    let mut chars = arg.chars();
                    chars.next(); // get rid of dash
                    for ch in chars
                    {
                        match ch
                        {
                        's' => opts.format = ConversionFmt::Short,
                        'l' => opts.format = ConversionFmt::Long,
                        _ => return Err(InterpretErr::UnknownShortOpt(ch)),
                        };
                    }
                }
            }
            else
            {
                extras.push(arg);

                for extra in args
                {
                    extras.push(extra);
                }

                if extras.len() < 3
                {
                    return Err(InterpretErr::IncompleteErr);
                }

                opts.interactive = false;
                break;
            }
        }

        Ok((opts, extras))
    }
}