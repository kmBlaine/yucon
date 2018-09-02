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

pub mod config;

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
    pub has_tags: bool,
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
            has_tags: false,
        }
    }
}

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
    // TODO make default_namespace part of the namespaces tree
    default_namespace: BTreeMap<Rc<String>, Rc<Unit>>,
    namespaces: BTreeMap<Rc<String>, BTreeMap<Rc<String>, Rc<Unit>>>,
    units: Vec<Rc<Unit>>,
    preferred_namespace: Rc<String>,
    //default_namespace_: Rc<String>
}

impl UnitDatabase
{
    pub fn new() -> UnitDatabase
    {
        let preferred = Rc::new("us".to_string());
        //let default = Rc::new("default".to_string());
        let mut namespaces_ = BTreeMap::new();
        namespaces_.insert(preferred.clone(), BTreeMap::new());
        //namespaces_.insert(default.clone(), BTreeMap::new());

        UnitDatabase { default_namespace: BTreeMap::new(),
                       namespaces: namespaces_,
                       units: Vec::new(),
                       preferred_namespace: preferred,
                       /*default_namespace_: default,*/ }
    }

    /*
    Checks if a given set of unit aliases will collide with any others in its
    tags / namespaces. If a collision is detected, the first namespace and alias
    that caused a collision are returned.
     */
    fn check_collisions(&self,
                        unit: &Unit,
                        aliases: &Vec<Rc<String>>,
                        tags: &Vec<Rc<String>>) -> Option<(Rc<String>, Rc<String>)>
    {
        if !unit.has_tags
        {
            if self.default_namespace.contains_key(&unit.common_name)
            {
                return Some(
                    (Rc::new("default".to_string()), unit.common_name.clone())
                );
            }
            for alias in aliases.iter()
            {
                if self.default_namespace.contains_key(alias)
                {
                    return Some(
                        (Rc::new("default".to_string()), alias.clone())
                    );
                }
            }

            return None;
        }

        for tag in tags.iter()
        {
            if let Some(namespace) = self.namespaces.get(tag)
            {
                if namespace.contains_key(&unit.common_name)
                {
                    return Some(
                        (tag.clone(), unit.common_name.clone())
                    );
                }

                for alias in aliases.iter()
                {
                    if namespace.contains_key(alias)
                    {
                        return Some(
                            (tag.clone(), alias.clone())
                        );
                    }
                }
            }
            else if tag.as_ref().as_str().eq("default")
            {
                for alias in aliases.iter()
                {
                    if self.default_namespace.contains_key(alias)
                    {
                        return Some(
                            (tag.clone(), alias.clone())
                        );
                    }
                }
            }
        }

        None
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
    pub fn add(&mut self, unit: Unit, aliases: &Vec<Rc<String>>, tags: &Vec<Rc<String>>) -> Option<Unit>
    {
        if let Some(collision) = self.check_collisions(&unit, aliases, tags)
        {
            let (tag, name) = collision;
            println!("UnitsDatabase: Unit is already registered in one or more of its tags. Unit will not be added.\n\
                      Colliding tag: '{}'    Colliding name: '{}'",
                    tag,
                    name
            );

            return Some(unit);
        }

        let unit_rc = Rc::new(unit);
        self.units.push(unit_rc.clone());

        if unit_rc.has_tags
        {
            for tag in tags.iter()
            {
                let mut namespace = if self.namespaces.contains_key(tag)
                {
                    self.namespaces.get_mut(tag).unwrap()
                }
                else if tag.as_str().eq("default")
                {
                    &mut self.default_namespace
                }
                else
                {
                    self.namespaces.insert(tag.clone(), BTreeMap::new());
                    self.namespaces.get_mut(tag).unwrap()
                };

                namespace.insert(unit_rc.common_name.clone(), unit_rc.clone());

                for alias in aliases.iter()
                {
                    namespace.insert(alias.clone(), unit_rc.clone());
                }
            }
        }
        else
        {
            self.default_namespace.insert(unit_rc.common_name.clone(), unit_rc.clone());

            for alias in aliases.iter()
            {
                self.default_namespace.insert(alias.clone(), unit_rc.clone());
            }
        }

        None
    }

    pub fn query(&self, name: &String, tag: Option<&String>) -> Option<Rc<Unit>>
    {
        //println!("name: {:?}    tag: {:?}", name, tag);
        let unit_result = if tag.is_some()
        {
            // if the unit was tagged, search only in the tagged namespace
            if let Some(namespace) = self.namespaces.get(tag.unwrap())
            {
                if let Some(unit_rc) = namespace.get(&Rc::new(name.clone()))
                {
                    Some(unit_rc.clone())
                }
                else
                {
                    None
                }
            }
            else
            {
                None
            }
        }
        else
        {
            // if the unit was not tagged search internal namespaces in the following order
            // 1. Preferred tag
            // 2. Default namespace
            // 3. All registered namespaces in alphabetical order
            let mut inner_result = if let Some(unit) = self.namespaces.get(&self.preferred_namespace).unwrap().get(name)
            {
                Some(unit.clone())
            }
            else
            {
                None
            };

            if inner_result.is_none()
            {
                inner_result = if let Some(unit) = self.default_namespace.get(name)
                {
                    Some(unit.clone())
                }
                else
                {
                    None
                }
            }

            if inner_result.is_none()
            {
                for (registered_tag, namespace) in self.namespaces.iter()
                {
                    if registered_tag.eq(&self.preferred_namespace)
                    {
                        continue;
                    }
                    if let Some(unit) = namespace.get(name)
                    {
                        inner_result = Some(unit.clone());
                        break;
                    }
                }
            }

            inner_result
        };
        /*
        if let Some(unit_rc) = self.default_namespace.get(&Rc::new(name.clone()))
        {
            return Some(unit_rc.clone());
        }
        */

        unit_result
    }
}

// TODO refactor to make unit field private to ensure no initialization occurs without proper tracking
pub struct UnitInit
{
    pub unit: Unit,
    default_name: bool,
    default_conv: bool,
    default_dims: bool,
    default_inv: bool,
    default_type: bool,
    default_zpt: bool,
}

impl UnitInit
{
    pub fn new() -> UnitInit
    {
        UnitInit
        {
            unit: Unit::new(),
            default_name: true,
            default_conv: true,
            default_dims: true,
            default_inv: true,
            default_type: true,
            default_zpt: true
        }
    }

    pub fn set_common_name(&mut self, name: String)
    {
        if self.default_name
        {
            self.unit.common_name = Rc::new(name);
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
            self.unit.conv_factor = conv_factor;
            self.default_conv = false;
        }
        else
        {
            println!("\n*** WARNING ***\n\
                  For unit {}: attemtped to assign conv_factor twice. Ignoring this attempt.\n",
                     self.unit.common_name);
        }
    }

    pub fn set_dimensions(&mut self, dimensions: u8)
    {
        if self.default_dims
        {
            self.unit.dimensions = dimensions;
            self.default_dims = false;
        }
        else
        {
            println!("\n*** WARNING ***\n\
                  For unit {}: attemtped to assign dimensions twice. Ignoring this attempt.\n",
                     self.unit.common_name);
        }
    }

    pub fn set_inverse(&mut self, inverse: bool)
    {
        if self.default_inv
        {
            self.unit.inverse = inverse;
            self.default_inv = false;
        }
        else
        {
            println!("\n*** WARNING ***\n\
                  For unit {}: attemtped to assign inverse twice. Ignoring this attempt.\n",
                     self.unit.common_name);
        }
    }

    pub fn set_unit_type(&mut self, unit_type: &'static str)
    {
        if self.default_type
        {
            self.unit.unit_type = unit_type;
            self.default_type = false;
        }
        else
        {
            println!("\n*** WARNING ***\n\
                  For unit {}: attemtped to assign unit_type twice. Ignoring this attempt.\n",
                     self.unit.common_name);
        }
    }

    pub fn set_zero_point(&mut self, zero_point: f64)
    {
        if self.default_zpt
        {
            self.unit.zero_point = zero_point;
            self.default_zpt = false;
        }
        else
        {
            println!("\n*** WARNING ***\n\
                  For unit {}: attemtped to assign zero_point twice. Ignoring this attempt.\n",
                     self.unit.common_name);
        }
    }

    pub fn is_well_formed(&self) -> bool
    {
        !(self.default_name || self.default_conv || self.default_type)
    }
}