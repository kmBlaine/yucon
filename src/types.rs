//pub mod types;

pub static UNIT_TYPES: [&'static str; 3] = ["length", "volume", "area"];
pub static DEFAULT_NAME: &'static str = "DEFAULT";

pub struct Unit
{
    pub common_name: String,
    pub unit_type: &'static str, //life time is static because the type strings are embedded
    pub conv_factor: f64,
    pub zero_point: f64,
    pub dimensions: u8,
    pub inverse: bool
}

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

