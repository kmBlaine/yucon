use std::{
    convert::From,
    collections::BTreeMap,
    ops::Deref,
    rc::Rc,
    fmt::{self, Display, Formatter},
    fs::File,
    io::Read,
};

#[derive(Deserialize, Debug)]
pub enum UnitType {
    Length,
    Volume,
}

#[derive(Deserialize, Debug)]
struct ConfigFileUnits {
    units: Vec<ConfigFileUnit>,
}

#[derive(Deserialize, Debug)]
struct ConfigFileUnit {
    unit: UnitParams
}

impl Deref for ConfigFileUnit {
    type Target = UnitParams;

    fn deref(&self) -> &UnitParams {
        &self.unit
    }
}

#[derive(Deserialize, Debug)]
struct UnitParams {
    name: String,
    unit_type: UnitType,
    conversion_factor: f64,
    aliases: Option<Vec<String>>,
    dimensions: Option<u32>,
    tags: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Unit {
    pub name: Rc<String>,
    pub unit_type: UnitType,
    pub conversion_factor: f64,
    pub aliases: Vec<Rc<String>>,
    pub dimensions: u32,
    pub tags: Vec<Rc<String>>,
}

impl Unit {
    /// Convenience method for testing if this unit has aliases.
    /// Mainly to make code more readable instead of typing `unit.aliases.len() > 0` all the time.
    pub fn has_aliases(&self) -> bool {
        self.aliases.len() > 0
    }

    /// Convenience method for testing if this unit has tags.
    /// Mainly to make code more readable instead of typing `unit.tags.len() > 0` all the time.
    pub fn has_tags(&self) -> bool {
        self.tags.len() > 0
    }
}

impl From<ConfigFileUnit> for Unit {
    fn from(cfg_unit: ConfigFileUnit) -> Self {
        let mut unit = Unit {
            name: Rc::new(cfg_unit.unit.name),
            unit_type: cfg_unit.unit.unit_type,
            conversion_factor: cfg_unit.unit.conversion_factor,
            aliases: cfg_unit
                .unit
                .aliases
                .unwrap_or(Vec::<String>::new())
                .into_iter()
                .map(|alias| Rc::new(alias))
                .collect(),
            dimensions: cfg_unit.unit.dimensions.unwrap_or(1u32),
            tags: cfg_unit
                .unit
                .tags
                .unwrap_or(Vec::<String>::new())
                .into_iter()
                .map(|tag| Rc::new(tag))
                .collect(),
        };

        // add the unit's common name into its aliases vector
        // this significantly simplifies logic later dealing with collisions and adding units
        unit.aliases.push(unit.name.clone());

        unit
    }
}

#[derive(Debug)]
pub struct UnitDatabase {
    namespaces: BTreeMap<Rc<String>, BTreeMap<Rc<String>, Rc<Unit>>>,
    units: Vec<Rc<Unit>>,
    pub preferred_namespace: Rc<String>,
    pub default_namespace: Rc<String>
}

pub struct NameCollision {
    pub namespace: Rc<String>,
    pub alias: Rc<String>
}

impl Display for NameCollision {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Alias '{}' already exists in tag '{}'", self.alias, self.namespace)
    }
}

impl UnitDatabase {
    pub const DEFAULT_NAMESPACE: &'static str = "default";

    fn parse_file(mut self, path: String) -> Option<Self> {
        const FILE_BUFFER_SIZE: usize = 131072; // this should be plenty barring stupid tier modifications
        let mut file_as_string = String::with_capacity(FILE_BUFFER_SIZE);
        let mut cfg_file = match File::open(path) {
            Err(err) => {
                error!("Could not load 'units.yaml': {}", err);
                return None;
            },
            Ok(file) => file,
        };

        match cfg_file.read_to_string(&mut file_as_string) {
            Err(err) => {
                error!("Failed to read 'units.yaml': {}", err);
                return None;
            },
            _ => (),
        }

        let units: ConfigFileUnits = match serde_yaml::from_str(&file_as_string) {
            Err(err) => {
                error!("Failed to deserialize 'units.yaml': {}", err);
                return None;
            },
            Ok(parsed_yaml) => parsed_yaml,
        };
        let units: Vec<Rc<Unit>> = units.units.into_iter().map(|unit| Rc::new(Unit::from(unit))).collect();

        units.into_iter().for_each(|unit| {
            self.add(unit);
        });

        Some(self)
    }

    pub fn load_from_file(units_cfg: String, preferred_namespace: Option<String>) -> Option<Self> {
        let default_namespace = Rc::new(Self::DEFAULT_NAMESPACE.to_string());
        let preferred_namespace = preferred_namespace
            .map(|namespace| Rc::new(namespace))
            .unwrap_or(default_namespace.clone());
        
        let mut namespaces = BTreeMap::new();
        namespaces.insert(default_namespace.clone(), BTreeMap::new());
        
        if default_namespace != preferred_namespace {
            namespaces.insert(preferred_namespace.clone(), BTreeMap::new());
        }

        UnitDatabase {
            default_namespace,
            namespaces,
            units: Vec::new(),
            preferred_namespace,
        }
        .parse_file(units_cfg)
    }

    /// Checks if the given set of unit aliases will collide with any others
    /// in its tags / namespaces. If a collision is detected, the a vector of the
    /// collisions will be returned. Else, `None` will be returned.
    fn check_collisions(&self, unit: Rc<Unit>) -> Option<Vec<NameCollision>>
    {
        let aliases = &unit.aliases;
        let tags = &unit.tags;
        let collision: Vec<NameCollision> = if !unit.has_tags() {
            let default_namespace_units = self
                .namespaces
                .get(&self.default_namespace)
                .expect("the default namespace should always be present");
            
            aliases.iter().filter_map(|alias| {
                if default_namespace_units.contains_key(alias) {
                    Some(NameCollision {
                        namespace: self.default_namespace.clone(),
                        alias: alias.clone()
                    })
                } else {
                    None
                }
            })
            .collect()
        } else {
            tags
                .iter()
                .filter_map(|tag| {
                    if let Some(units) = self.namespaces.get(tag) {
                        Some((tag, units))
                    } else {
                        None
                    }
                })
                .flat_map(|(tag, units)| {
                    // this really shouldn't be necessary to collect into a type annotated Vec but...
                    // the borrow checker and type inference systems aren't quite robust enough to see what's happening =)
                    let collisions: Vec<NameCollision> = aliases.iter().filter_map(|alias| {
                        if units.contains_key(alias) {
                            Some(NameCollision {
                                namespace: tag.clone(),
                                alias: alias.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                    collisions
                })
                .collect()
        };

        let collision = if collision.len() > 0 {
            Some(collision)
        } else {
            None
        };

        collision
    }

    /// Adds a unit the database if neither its name nor any of its aliases exist
    /// in the database under any of its listed tags. Returns `true` on success.
    /// Otherwise `false` will be returned to indicate failure.
    pub fn add(&mut self, unit: Rc<Unit>) -> bool {
        if let Some(collisions) = self.check_collisions(unit.clone()) {
            warn!("Unit with name '{}' will not be added. One or more of the unit's aliases is already registered in the database", unit.name);
            collisions.iter().for_each(|collision| info!("{}", collision));

            return false;
        }

        self.units.push(unit.clone());

        if unit.has_tags() {
            unit.tags.iter().for_each(|tag| {
                // let namespace = if !self.namespaces.contains_key(tag) {
                //     self.namespaces.insert(tag.clone(), BTreeMap::new());
                //     self.namespaces.get_mut(tag).unwrap()
                // } else {
                //     self.namespaces.get_mut(tag).unwrap()
                // };
                let namespace = self.namespaces.entry(tag.clone()).or_insert(BTreeMap::new());

                unit.aliases.iter().for_each(|alias| {
                    namespace.insert(alias.clone(), unit.clone());
                });
            });
        } else {
            let namespace = self.namespaces.get_mut(&self.default_namespace).expect("default namespace is always present");
            
            unit.aliases.iter().for_each(|alias| {
                namespace.insert(alias.clone(), unit.clone());
            });
        }

        true
    }

    pub fn query(&self, name: &String, tag: Option<&String>) -> Option<Rc<Unit>> {
        //println!("name: {:?}    tag: {:?}", name, tag);
        let unit_result = if let Some(tag) = tag {
            // if the unit was tagged, search only in the tagged namespace
            if let Some(namespace) = self.namespaces.get(tag) {
                namespace.get(name).map(|unit| unit.clone())
            } else {
                None
            }
        } else {
            // if the unit was not tagged search internal namespaces in the following order
            // 1. Preferred tag
            // 2. Default namespace
            // 3. All registered namespaces in alphabetical order
            let mut unit = self
                .namespaces
                .get(&self.preferred_namespace)
                .expect("preferred namespace is always present")
                .get(name)
                .map(|unit| unit.clone());

            if unit.is_none() {
                unit = self
                    .namespaces
                    .get(&self.default_namespace)
                    .expect("default namespace is always present")
                    .get(name)
                    .map(|unit| unit.clone());
            }

            if unit.is_none() {
                for (registered_tag, namespace) in self.namespaces.iter() {
                    if registered_tag.eq(&self.preferred_namespace) || registered_tag.eq(&self.default_namespace) {
                        // make sure to skip the default and preferred. we already checked these
                        continue;
                    }
                    
                    if let Some(retrieved_unit) = namespace.get(name) {
                        unit = Some(retrieved_unit.clone());
                        break;
                    }
                }
            }

            unit
        };

        unit_result
    }
}
