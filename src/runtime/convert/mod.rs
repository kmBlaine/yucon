
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use ::runtime::units::{Unit, UnitDatabase};
use ::runtime::parse::ConvPrimitive;
use ::utils::{NO_PREFIX, prefix_as_num};


#[derive(Debug)]
pub enum ConversionError
{
    OutOfRange(bool),   // input or output value not a valid f64, false: input
    UnitNotFound(bool), // the unit was not found, false: input
    TypeMismatch,       // the units' types disagree, ie volume into length
}
const INPUT: bool = false;
const OUTPUT: bool = true;


#[derive(Debug, Copy, Clone)]
pub enum ConversionFmt
{
    Short,
    Desc,
    Long,
}

impl Display for ConversionFmt
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        match *self
        {
        ConversionFmt::Short => write!(f, "s: short / value only"),
        ConversionFmt::Desc => write!(f, "d: descriptive / value and output unit"),
        ConversionFmt::Long => write!(f, "l: long / input and output values and units"),
        }
    }
}

#[derive(Debug)]
pub struct Conversion
{
    from_prefix: char,
    to_prefix: char,
    pub from_alias: String,
    pub to_alias: String,
    pub from_tag: Option<String>,
    pub to_tag: Option<String>,
    pub from: Option<Rc<Unit>>,
    pub to: Option<Rc<Unit>>,
    pub input: f64,
    pub result: Result<f64, ConversionError>,
    pub format: ConversionFmt,
}

impl Conversion
{
    fn new(input_prefix: char, input_alias: String, input_tag: Option<String>,
        output_prefix: char, output_alias: String, output_tag: Option<String>,
        input_val: f64) -> Conversion
    {
        Conversion {
            from_prefix: input_prefix,
            to_prefix: output_prefix,
            from_alias: input_alias,
            to_alias: output_alias,
            from_tag: input_tag,
            to_tag: output_tag,
            from: None,
            to: None,
            input: input_val,
            result: Ok(1.0),
            format: ConversionFmt::Desc,
        }
    }
}

impl Display for Conversion
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.result
        {
        Ok(ref output) => {
            match self.format
            {
            ConversionFmt::Short => write!(f, "{}", output),
            ConversionFmt::Desc  => {
                let mut prefix = String::with_capacity(1);
                if self.to_prefix != NO_PREFIX
                {
                    prefix.push(self.to_prefix);
                }

                write!(f, "{} {}{}", output, prefix, self.to_alias)
            },
            ConversionFmt::Long  => {
                let mut to_prefix = String::with_capacity(1);
                let mut from_prefix = String::with_capacity(1);

                if self.to_prefix != NO_PREFIX
                {
                    to_prefix.push(self.to_prefix);
                }

                if self.from_prefix != NO_PREFIX
                {
                    from_prefix.push(self.from_prefix);
                }
                write!(f, "{} {}{} = {} {}{}", self.input, from_prefix, self.from_alias,
                    output, to_prefix, self.to_alias)
            },
            }
        },
        Err(ref err) => {
            match err
            {
            &ConversionError::OutOfRange(in_or_out) => {
                write!(f, "Conversion error: {} value is out of range",
                    if in_or_out == OUTPUT
                    {
                        "output"
                    }
                    else
                    {
                        "input"
                    })
            },
            &ConversionError::UnitNotFound(in_or_out) => {
                // TODO output tag
                write!(f, "Conversion error: no unit called \'{}\' was found",
                    if in_or_out == OUTPUT
                    {
                        &self.to_alias
                    }
                    else
                    {
                        &self.from_alias
                    })
            },
            &ConversionError::TypeMismatch =>
                write!(f, "Conversion error: input and output types differ.\
                          \'{}\' is a {} and \'{}\' is a {}",
                          self.from_alias, self.from.as_ref().unwrap().unit_type,
                          self.to_alias, self.to.as_ref().unwrap().unit_type),
            }
        },
        }
    }
}










/* Performs a unit conversion given as an input value, input unit and prefix,
 * and an output unit and prefix. Fetches the units from the given units database
 * A struct conversion is returned allowing the caller to do with it as they
 * please. Note that struct Conversion implements the Display trait and tracks
 * its own validity / error state. This function returns as soon as an error is
 * encountered.
 *
 * Parameters:
 *   - input: the value to be converted
 *   - from_prefix: the single character metric prefix of the input unit
 *   - from: name / alias of the unit to that will be converted
 *   - to_prefix: the single character metric prefix of the output unit
 *   - to: name / alias of the unit to convert to
 *   - units: reference to the database that holds all of the units
 *
 * Stages of Conversion:
 *   1. scale input using prefix and dimensions
 *   2. invert result if necessary
 *   3. change result to base units
 *   4. adjust result to output scale
 *   5. change result to output units
 *   6. invert result if necessary
 *   7. scale result using prefix and dimensions
 */
pub fn convert(input: f64, from_prefix: char, from: String, from_tag: Option<String>,
    to_prefix: char, to: String, to_tag: Option<String>, units: &UnitDatabase) -> Conversion
{
    //println!("from_tag: {:?}    to_tag: {:?}", from_tag, to_tag);
    let mut conversion = Conversion::new(from_prefix,from, from_tag,
                                         to_prefix,to, to_tag,
                                         input);

    // if the input value is NaN, INF, or too small
    // Exactly 0 is acceptable however which is_normal() does not account for
    if (!conversion.input.is_normal()) && (conversion.input != 0.0)
    {
        conversion.result = Err(ConversionError::OutOfRange(INPUT));
        return conversion;
    }

    conversion.from = units.query(&conversion.from_alias, conversion.from_tag.as_ref());
    conversion.to = units.query(&conversion.to_alias, conversion.to_tag.as_ref());

    if conversion.from.is_none()
    {
        conversion.result = Err(ConversionError::UnitNotFound(INPUT));
    }
    if conversion.to.is_none()
    {
        conversion.result = Err(ConversionError::UnitNotFound(OUTPUT));
    }
    if conversion.result.is_err()
    {
        return conversion;
    }

    if conversion.to.as_ref().unwrap().unit_type !=
        conversion.from.as_ref().unwrap().unit_type
    {
        conversion.result = Err(ConversionError::TypeMismatch);
        return conversion;
    }

    // do not initialize yet. we will fetch these values from conversion
    let from_conv_factor: f64;
    let from_zero_point: f64;
    let from_dims: i32;
    let from_is_inverse: bool;
    let to_conv_factor: f64;
    let to_zero_point: f64;
    let to_dims: i32;
    let to_is_inverse: bool;
    {
        // borrow scope for retrieving the unit properties
        // avoids massive method chains on struct Conversion
        let unit_from = conversion.from.as_ref().unwrap();
        from_conv_factor = unit_from.conv_factor;
        from_zero_point = unit_from.zero_point;
        from_dims = unit_from.dimensions as i32;
        from_is_inverse = unit_from.inverse;

        let unit_to = conversion.to.as_ref().unwrap();
        to_conv_factor = unit_to.conv_factor;
        to_zero_point = unit_to.zero_point;
        to_dims = unit_to.dimensions as i32;
        to_is_inverse = unit_to.inverse;
    } // end borrow scope

    // S1
    let mut output_val = conversion.input * prefix_as_num(
        conversion.from_prefix)
        .unwrap().powi(from_dims);

    // S2
    if from_is_inverse
    {
        output_val = 1.0 / output_val;
    }

    output_val *= from_conv_factor; // S3
    output_val += from_zero_point - to_zero_point; // S4
    output_val /= to_conv_factor; // S5

    // S6
    if to_is_inverse
    {
        output_val = 1.0 / output_val;
    }

    // S7
    output_val /= prefix_as_num(conversion.to_prefix).unwrap().powi(to_dims);

    // if the output value is NaN, INF, or too small to properly represent
    // Exactly 0 is acceptable however which is_normal() does not account for
    if (!output_val.is_normal()) && (output_val != 0.0)
    {
        conversion.result = Err(ConversionError::OutOfRange(OUTPUT));
        return conversion;
    }

    conversion.result = Ok(output_val);

    conversion
}

pub fn convert_all(conv_primitive: ConvPrimitive, units: &UnitDatabase) -> Vec<Conversion>
{
    let mut all_conversions = Vec::with_capacity(1);

    for value_expr in conv_primitive.input_vals
    {
        for output_unit in conv_primitive.output_units.iter()
        {
            //println!("{:?}", output_unit);
            all_conversions.push(
                convert(value_expr.value,
                        conv_primitive.input_unit.prefix, conv_primitive.input_unit.alias.clone().unwrap(), conv_primitive.input_unit.tag.clone(),
                        output_unit.clone().prefix, output_unit.clone().alias.unwrap(), output_unit.tag.clone(),
                        units)
            );
        }
    }

    all_conversions
}
