//pub mod types;
use std::rc::Rc;

// unit types Yucon recognizes
// statically allocated so that we do not waste memory storing duplicate data
pub static UNIT_TYPES: [&'static str; 12] = ["area",
                                             "energy",
                                             "force",
                                             "fuel economy",
                                             "length",
                                             "mass",
                                             "power",
                                             "pressure",
                                             "speed",
                                             "temperature",
                                             "torque",
                                             "volume",];

#[derive(Debug)]
pub struct Unit
{
	pub common_name: Rc<String>,
	pub conv_factor: f64,
	pub dimensions: u8,
	pub inverse: bool,
	pub unit_type: &'static str, //life time is static because the type strings are embedded
	pub zero_point: f64,
	pub has_aliases: bool,
	default_name: bool,
	default_conv: bool,
	default_dims: bool,
	default_inv: bool,
	default_type: bool,
	default_zpt: bool,
}

impl Unit
{
	pub fn new() -> Unit
	{
		Unit {
			common_name: Rc::new(String::new()),
			conv_factor: 1.0,
			dimensions: 1,
			inverse: false,
			unit_type: UNIT_TYPES[0],
			zero_point: 0.0,
			has_aliases: false,
			default_name: true,
			default_conv: true,
			default_dims: true,
			default_inv: true,
			default_type: true,
			default_zpt: true,
		}
	}
	
	pub fn set_common_name(&mut self, name: String)
	{
		if self.default_name
		{
			self.common_name = Rc::new(name);
			self.default_name = false;
		}
		else
		{
			unreachable!();
			// the code is written such that there should never be an attempt
			// to assign a common_name twice. encountering a new common name
			// in config triggers a flush of the current unit and starts a new one.
		}
	}
	
	pub fn set_conv_factor(&mut self, conv_factor: f64)
	{
		if self.default_conv
		{
			self.conv_factor = conv_factor;
			self.default_conv = false;
		}
		else
		{
			println!("\n*** WARNING ***\n\
			          For unit {}: attemtped to assign conv_factor twice. Ignoring this attempt.\n",
			          self.common_name);
		}
	}
	
	pub fn set_dimensions(&mut self, dimensions: u8)
	{
		if self.default_dims
		{
			self.dimensions = dimensions;
			self.default_dims = false;
		}
		else
		{
			println!("\n*** WARNING ***\n\
			          For unit {}: attemtped to assign dimensions twice. Ignoring this attempt.\n",
			          self.common_name);
		}
	}
	
	pub fn set_inverse(&mut self, inverse: bool)
	{
		if self.default_inv
		{
			self.inverse = inverse;
			self.default_inv = false;
		}
		else
		{
			println!("\n*** WARNING ***\n\
			          For unit {}: attemtped to assign inverse twice. Ignoring this attempt.\n",
			          self.common_name);
		}
	}
	
	pub fn set_unit_type(&mut self, unit_type: &'static str)
	{
		if self.default_type
		{
			self.unit_type = unit_type;
			self.default_type = false;
		}
		else
		{
			println!("\n*** WARNING ***\n\
			          For unit {}: attemtped to assign unit_type twice. Ignoring this attempt.\n",
			          self.common_name);
		}
	}
	
	pub fn set_zero_point(&mut self, zero_point: f64)
	{
		if self.default_zpt
		{
			self.zero_point = zero_point;
			self.default_zpt = false;
		}
		else
		{
			println!("\n*** WARNING ***\n\
			          For unit {}: attemtped to assign zero_point twice. Ignoring this attempt.\n",
			          self.common_name);
		}
	}
	
	pub fn is_well_formed(&self) -> bool
	{
		!(self.default_name || self.default_conv || self.default_type)
	}
}

/*
pub struct UnitScalar<'a>
{
	pub unit: &'a Unit,
	pub scalar: f64,
	pub prefix: f64
}

impl<'a> UnitScalar<'a>
{
    pub fn convert_to( &self, to: &'a Unit, prefix: f64 ) -> UnitScalar
    {
        let mut converted = UnitScalar { unit: to,
                                         scalar: 1.0,
                                         prefix: 1.0 };

        converted.scalar = ((self.scalar * self.prefix + self.unit.zero_point)
                           *(self.unit.conv_factor / to.conv_factor)
                           - to.zero_point
                           )
                           / prefix;
        converted.prefix = prefix;
        converted.unit = to;

        let converted = converted;

        converted
    }
}
*/
