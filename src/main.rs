mod types;

use types::*;

fn main() {
    let mut gallon = Unit { common_name: types::DEFAULT_NAME.to_string(),
                            unit_type: types::DEFAULT_NAME,
                            conv_factor: 1.0,
                            zero_point: 0.0 };

    gallon.common_name = "gallon".to_string();
    gallon.unit_type = types::UNIT_TYPES[1];
    gallon.conv_factor = 3785.411784;
    gallon.zero_point = 0.0;

    let mut liter = Unit{ common_name: types::DEFAULT_NAME.to_string(),
                          unit_type: types::DEFAULT_NAME,
                          conv_factor: 1.0,
                          zero_point: 0.0 };

    liter.common_name = "litre".to_string();
    liter.unit_type = types::UNIT_TYPES[1];
    liter.conv_factor = 1000.0;
    liter.zero_point = 0.0;

    let gallon = gallon;
    let liter = liter;

    let orig_unit = UnitScalar { unit: &gallon,
                                 scalar: 1.0,
                                 prefix: 1.0 };

    let conv_unit = orig_unit.convert_to( &liter, 1.0 );

    println!("Common Name: {}\nUnit Type: {}\nConversion Factor: {}\nZero Point: {}\n",
              gallon.common_name,
              gallon.unit_type,
              gallon.conv_factor,
              gallon.zero_point );

    println!("Common Name: {}\nUnit Type: {}\nConversion Factor: {}\nZero Point: {}\n",
              liter.common_name,
              liter.unit_type,
              liter.conv_factor,
              liter.zero_point );

    println!("{} {}(s) is {} {}s",
              orig_unit.scalar,
              orig_unit.unit.common_name,
              conv_unit.scalar,
              conv_unit.unit.common_name );
}
