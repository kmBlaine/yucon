use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::io::prelude::*;
use types::Unit;
use regex::Regex;

fn strip_comments( string: &mut String )
{
    let mut new_size: usize = string.len();
    
    {
        let mut chars = string.char_indices();

        loop
        {
            let (byte_offset, character) = match chars.next()
            {
                None           => break,
                Some( values ) => values,
            };
            
            if character == '\\'
            {
                chars.next();
            }
            else if character == '#'
            {
                new_size = byte_offset;
                break;
            }
        }
    };
    
    string.truncate( new_size );
}

pub fn load_units_list() -> Vec<String>
{
    let file = match File::open("/etc/yucon/units2.cfg")
    {
        Err( desc ) => panic!("Aborting. Unable to open units.cfg: {}", desc.description() ),
        Ok( file )  => file,
    };

    let mut units_cfg = BufReader::new( file );
    let mut units_list: Vec<String> = Vec::new();
    let mut line = String::new();

    while units_cfg.read_line( &mut line ).unwrap() > 0
    {
        //strip away comments, leading/trailing whitespace
        strip_comments( &mut line );
        line = line.trim().to_string(); //ignore leading and trailing whitespace

        //if the line is empty, don't even bother. wasted CPU and memory
        if line.is_empty()
        {
            continue;
        }

        let mut next_line = String::new();
        next_line.insert_str(0, &line );
        units_list.push( next_line ); //add without trailing whitespace
        
        line.clear();
    }

    let units_list = units_list;

    units_list
}
