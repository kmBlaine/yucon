use types::Unit;
use std::collections::BTreeMap;
use std::rc::Rc;

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
	pub fn add(&mut self, unit: Unit, mut aliases: Vec<Rc<String>>) -> Option<(Unit, Vec<Rc<String>>)>
	{
		let mut exists = false;

		for alias in &aliases
		{
			if self.aliases.contains_key(alias)
			{
				exists = true;
				break;
			}
		}
		
		if self.aliases.contains_key(&unit.common_name)
		{
			exists = true;
		}

		if !exists
		{
			let common_name = unit.common_name.clone();
			let unit_rc = Rc::new(unit);

			self.units.push(unit_rc.clone());
			self.aliases.insert(common_name, unit_rc.clone());
			
			for alias in aliases
			{
				self.aliases.insert(alias, unit_rc.clone());
			}

			return None; //if adding was sucssesful, we don't need to bother handing back the values
		}

		Some((unit, aliases)) //if adding was unsucssesful, we need to hand back the values
	}
}
