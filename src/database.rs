use types::Unit;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct UnitDatabase
{
    aliases: BTreeMap<String, Rc<Unit>>,
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
    pub fn add( &mut self, unit: Unit, aliases: Vec<String> ) -> Option<(Unit, Vec<String>)>
    {
        let mut exists = false;
        let aliases_num = aliases.len();
        let mut index: usize = 0;

        //check for duplicates in selected namespaces before continuing
        while index < aliases_num
        {
            if self.aliases.contains_key( &aliases[index] )
            {
                exists = true;
                break;
            }

            index += 1;
        }

        if exists == false
        {
            let unit_ref = Rc::new( unit );

            self.units.push( unit_ref.clone() );
            for alias in aliases
            {
                self.aliases.insert( alias, unit_ref.clone() );
            }

            return None; //if adding was sucssesful, we don't need to bother handing back the values
        }

        Some((unit, aliases)) //if adding was unsucssesful, we need to hand back the values
    }
}
