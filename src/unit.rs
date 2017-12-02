/* unit.rs
 * ===
 * Contains the internal unit representation struct and the internal units database.
 * 
 * This file is a part of:
 * 
 * Yucon - General Purpose Unit Converter
 * Copyright (C) 2016-2017  Blaine Murphy
 *
 * This program is free software: you can redistribute it and/or modify it under the terms
 * of the GNU General Public License as published by the Free Software Foundation, either
 * version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
 * without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::BTreeMap;
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

// TODO: tracking defaults is only necessary during allocation. add an initilization wrapper
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

/* struct UnitDatabase
 * 
 * This struct is for containing the units that are read from the units.cfg file
 * It is composed of two parts: a B-Tree map for O(log n) search of units by name
 * and a vector for easy listing of all available units. Units are
 * stored using reference counts to avoid dual allocation overhead and so that
 * online removal of units is a more straightforward process if it is ever
 * implemented in the future.
 *
 * Fields:
 *   - aliases: associative map between all unit names / aliases in the program\
 *       and the correct unit
 *
 *   - units: linear container for all units in the program so that they may
 *       be easily listed at user's request.
 * 
 */
pub struct UnitDatabase
{
	aliases: BTreeMap<Rc<String>, Rc<Unit>>,
	units: Vec<Rc<Unit>>
}

impl UnitDatabase
{
	pub fn new() -> UnitDatabase
	{
		UnitDatabase { aliases: BTreeMap::new(),
		               units: Vec::new() }
	}

	/**
	# add()

	# Description:
	This method adds a unit the database if it does already exist in the
	database. To avoid expensive heap copy, this method takes ownership of its
	arguments under the assumption that they will be successfully added. If
	adding was unsuccsessful, they returned back in a tuple. Otherwise
	Option::None is returned.

	Def. duplicate unit: has an alias that already exists in the namespaces
	selected for it.

	When a duplicate unit is detected, THE UNIT IS NOT ADDED IN ANY NAMESPACE!
	Effectively discarded.

	# Parameters:
	unit - a unit to be added
	aliases - a vector of names for the unit

	# Returns: Option<(Unit, Vec<String>)>
	Success: None
	Failure: Some
	*/
	pub fn add(&mut self, unit: Unit, aliases: &Vec<Rc<String>>) -> Option<Unit>
	{
		let mut exists = false;

		if self.aliases.contains_key(&unit.common_name)
		{
			exists = true;
		}
		
		if unit.has_aliases
		{
			for alias in aliases
			{
				if self.aliases.contains_key(alias)
				{
					exists = true;
					break;
				}
			}
		}

		if !exists
		{
			let common_name = unit.common_name.clone();
			let has_aliases = unit.has_aliases;
			let unit_rc = Rc::new(unit);

			self.units.push(unit_rc.clone());
			self.aliases.insert(common_name, unit_rc.clone());

			if has_aliases
			{
				for alias in aliases
				{
					self.aliases.insert(alias.clone(), unit_rc.clone());
				}
			}

			return None; //if adding was sucssesful, we don't need to bother handing back the values
		}

		Some(unit) //if adding was unsucssesful, we need to hand back the values
	}

	pub fn query(&self, name: &String) -> Option<Rc<Unit>>
	{
		if let Some(unit_rc) = self.aliases.get(&Rc::new(name.clone()))
		{ 
			return Some(unit_rc.clone());
		}

		None
	}
}
