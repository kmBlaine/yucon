# Yucon Configuration
Central to Yucon is its flexible configuration, specifically the  units.cfg file
which allows units to be added the program simply editing a human-readable text
file. Of course for these new units to be understood by the program, they must
obey a certain syntax.

## The units.cfg File
The units.cfg file contains all of the unit declarations for Yucon, which
includes the standard units that it ships with and any user-defined units which
they add themselves. Although Yucon allows you to edit the default units which
simplifies the implementation, it is highly recommended that you do not.

### 1 - Units in Yucon
Before any discussion can be had about the specifics of the units.cfg file, it
is important to understand how Yucon handles units. First off, a unit, as
handled by Yucon, has three essential parts:
* **A common name** ie. inch, cubic centimeter, gallon, etc. This allows us to
  identify and look up a unit in a friendly, human-readable way.
* **A type** such as length, volume, mass, force, pressure, and so forth. This
  prevent nonsensical conversions like volume into pressure.
* **A conversion factor** or how many of the common reference unit composes one
  of this unit.

The conversion factor is the magic of Yucon which avoids the combinatorial
explosion of knowing the exact conversion ratio between every possible pair of
units. Instead, Yucon uses a common reference unit. The conversion factor is
the conversion ratio between the auxiliary unit and the refernce unit such that
1 auxiliary unit equals the conversion factor reference units. Converting is
then a simple matter of substitution into a predefined algebraic formula.

In addition to these basic three elements, units may also have additional
properties which must be considered during handling or that simply give us more
ways to talk about that unit. Optional properties are:
* **Aliases.** Alternate spellings or abbreviations used for the unit.
* **Zero Point.** The point on the reference unit scale that constitutes
  zero on this unit's scale.
* **Dimensions.** How many dimensions the unit has eg. a square meter has 2 while a
  cubic meter has 3 and a normal meter has 1
* **Inverse.** Determines if the unit is the inverse of the base unit.

Each of this gives us a little bit more info on our unit but some or all of it
may not be necessary. Not every unit has an abbreviation or alternate spelling for
instance. Thus, safe defaults can be assumed for these in most cases. But
there are instances where the defaults create unexpected behavior as we will
see.

#### 1.2 - Aliases:
Sometimes spellings are not agreed upon. "Centimetre" is commonly also
spelled "centimeter" with the r and e transposed. As either is correct 
depending on where you live, it would be nice if you could use whatever you
are used to. Aliasing units allows for this. Also, who wants to type
either of those every time when "cm" is universally recognized as the abreviation
for both? Aliasing also allows you to give a unit terse abbreviations.
It is important to note that Yucon allows no duplicate names. Thus while it
would be nice if it just knew that if we are in the US, "gallon" means
"US gallon" not "Imperial gallon", this feature is not quite implemented yet.
Gallon therefore must have unique names to disambiguate ie "gal-us" or "gal-uk".

#### 1.3 - Zero Point:
In most cases 0 units is 0 reference units; 0 inches is the same length
0 centimeters, 0 gallons is the same volume as 0 litres and so on. Temperature
is the notable exception to this rule; 0 F is **NOT** 0 C. In fact its actually
somewhere around -17.8 C. The zero point tells us where zero for this unit is on
the reference unit's scale.

**Ex.** Yucon uses Kelvin as the reference temperature unit. Lets see how Fahrenheit
and Celcius are defined:
    
    [celcius]
    aliases     = C
    type        = temperature
    conv_factor = 1           # Celcius and Kelvin have the same step size
    zero_point  = 273.15      # but 0 C is actually 273.15 K
    
    [fahrenheit]
    aliases     = F
    type        = temperature
    conv_factor = 0.555555555555556   # Fahrenheit is 5/9 of a degree Kelvin
    zero_point  = 255.372222222222222 # and 0 F is 255.372 K
    tags        = us,uk

#### 1.4 - Dimensions:
Most units are one-dimensional meaning that when a metric
prefix is applied, it scales in direct proportion to the prefix; 1 mL is
1/1000th of a litre. However, square and cubic units do not obey this wisdom;
1 cubic centimeter is actually 1/1,000,000th of 1 cubic meter. This is because
a cubic meter is composed of three individual meters and thus all of them must
be scaled, not just one. Consquently, the "centi" scalar which usually denotes
1/100th must now be raised to the power of three. The "dimensions" property
tells Yucon how to scale a unit properly. NOTE: Do not make the mistake of
thinking every area or volume type unit needs to have 2 or 3 dimensions
respectively as we see in the above example. As a rule of thumb, just the ones
that have "square" or "cubic" in the name need dimensioning.

#### 1.5 - Inverse:
Most units have the same exact conceptual dimension as the reference unit. For
instance, all pressure units are a force per area. Some unit types are less
strictly specified however, such as fuel economy. Fuel economy is sometimes
measured as distance travelled per unit of fuel consumed and other times it is
measured as fuel consumed per distance travelled. You might note that these are
the same metrics but they are simply inverted. Thus it is easy to convert between
say L/100km and MPG but one or the other must be inverted before Yucon can apply
the conversion factor. In this case, it would be MPG since Yucon uses L/100km as
the reference unit for fuel economies. If a unit is the inverse of the reference
unit, like MPG is relative to L/100km, you must inform Yucon of this so it can
perform the conversion properly.

### 2 - General Syntax and Formatting:
The units.cfg file essentially uses INI syntax and format which is simple,
easy to parse, and very readable. However its exact syntax and format differ
from defacto INI, so please consider the following:
1. **Yucon supports UTF8 so that everyone can enjoy their native character sets. This also applies to units.cfg** 
2. The following characters are reserved: [, ], =, #, \\, and ','. To be used as
   a literal, they MUST be escaped by a \\ no matter where they occur on a line.
   Ex:
   
       aliases = has\=,has\,,has\\

   yields:

       "has=", "has,", and "has\"

   upon tokenization.
   
   _Be wary of the \\ character. A single \\ is interpretted as an escape
   sequence but if the character following isn't one of our reserved characters
   a bad escape sequence error will occur_
   
   Ex.
   
       aliases = \break everything, now    # \b is not a valid escape sequence
   
3. Whitespace, apart from newline (LF, '\\n') as INI is a line-based format, is
   insignificant and may be used liberally for formatting. Empty / whitespace
   lines are effectively comments and will be completely ignored. A corrolary to this
   is that DOS/Windows style line endings (CR LF, "\\r\\n") have no effect on
   the syntactic validity or sematics of the file as the \\r will simply
   be ignored being a whitespace character. That being said, there are a couple
   of caveats:
   * Leading and trailing whitespace is discarded but enclosed whitespace is not
     Ex.
     
         "     [   cubic centimeter  ]          # header for the cm3 unit"
         "  aliases      =    cm3   ,  cubic centimetre   "
     
     yields:
     
         "", "[", "cubic centimeter", "]", ""
         "aliases", "=", "cm3", ",", "cubic centimetre"
     
     upon tokenization. Note that the whitespace leading and trailing "aliases"
     and the delimiters [, ], =, and ',' is discarded but whitespace, enclosed
     by "cubic" and "centimeter" is preserved.
     
     > But what if I want to use leading and trailing whitespace in something?
     
     This scenario has been considered and dismissed; such a feature is not and probably will
     not be implemented. Suffice it to say, its hard to imagine any situation
     where it would really be applicable.
   * It is frowned upon if DOS/Windows style line endings are used because as 
     everyone with an ounce of sense knows, you only need ONE character to
     terminate a line and that character is newline (LF, '\\n').
   * On that last note it is outright FORBIDDEN to edit this file with
     Notepad.exe. Atom, Notepad++, Sublime - they're out there. Just saying.
4. This file is case sensitive and favors lower case.

### 3 - Unit Declaration Syntax
Just as in INI there are essentially three categories of things in the units.cfg
file. They are:
* **Section headers** which are denoted by a token wrapped in square brackets
  **[ ]** and also define the unit's common name. Section headers tell Yucon to
  associate all following properties with a new unit.
  
  Ex.

      [ kilowatt ]
       
* **Key-value Pairs** which are two tokens separated by an equals sign **=**.
  The left token is the key and the right token(s) are the value(s). Key-value
  pairs are how unit properties such as its type are described.

  Ex.
  
      type = power
      
  "Type" is the key and "power" is the value. A kilowatt is a unit of power.
* **Comments** which are anything following a hash **#**. Comments are
  information for a human reader. Unlike defacto INI (which first off uses **;** as
  the comment character), comments may appear on the same line as anything
  significant as well as anywhere on an empty line.
  
  Ex.
  
           # This is a mandatory field
      conv_factor = 1000  # base unit is watts

__The Catch-All Syntax Rule:__ If you write something that doesn't clearly look
like one of these three things, Yucon will complain LOUDLY on startup. And then
promptly discard the offending line whether it was significant or not. Yucon
will make _ITS BEST EFFORT_ to read units.cfg without panicing and crashing so be
wary of any errors it shouts at you and please heed them. Else you could have
units that are incorrectly specified.

#### 3.1 - Section Header Syntax
Some specifics on header syntax
* There MUST be some non-whitespace token between the brackets.
  
  Ex.
  
      [inch]  # This is okay
      [  ]    # This is intentionally broken and will raise an error
      []      # This is also broken

* Unescaped delimiters will cause errors but there is no need to fear leading
  and trailing whitespace. See __General Syntax and Formatting__: Rules 2-3.

  Ex.
  
          [  miles per hour  ]    # This is okay
      [litres per 100,0km]        # This is intentionally broken and will cause errors
      [litres per 100\,0km]       # This version will work however
      
#### 3.2 - Key-value Pair Syntax
Some specifics on key-value pair syntax:
* There must be at least 1 non-whitespace AND non-delimiter token after the **=**
  
  Ex.
  
      dimensions =      # This is intentionally broken and will cause errors
      dimensions = 3    # This is okay
      aliases    = , ,  # This is intentionally broken and will cause errors
      aliases    = cm3, cubic centimetre  # This is okay

* Unescaped delimiters will cause errors but there is no need to fear leading
  and trailing whitespace. See __General Syntax and Formatting__: Rules 2-3.
  
  **ONE IMPORTANT EXCEPTION!** When defining unit aliases, which as you might
  have guessed by now are comma-separated lists, sequential unescaped commas are
  **OK!** (believe it or not). This is because by definition delimiters separate
  two things even if one of those things is empty. Thus Yucon will interpolate the
  blank tokens but simply ignore them.
  
  Ex.
  
      aliases = cm3, cubic centimetre        # This is okay
      aliases = ,,cm3,,,,cubic centimetre,   # This is ALSO okay
      
  Both lines are valid syntax and also have exactly the same semantics.
  
### 4 - To Declare A Unit
Now that we've had our conversation on formatting, syntax, and how units are
handled, we can finally talk about declaring custom units. As discussed before,
a unit is composed of:
* **A Common Name / Section Header**
      
      [microgram]
      
* **A Set of Key-value Pairs.** Valid keys are as follows:

      aliases      : aliases for the unit
      conv_factor* : how many base units equals one of this unit
      dimensions   : how many dimensions this unit has
      inverse      : tells whether or not this unit is an inverse
      type*        : type of unit (length, volume, etc)
      zero_point   : where zero is on the base unit's scale

Starred fields are mandatory. All others are optional. Key-value pairs do not
have to appear in any particular order nor are they required to appear
sequentially without separation.

Yucon will make its best effort to add a unit. If the mandatory fields are
fulfilled, then the unit will be added.

#### 4.1 - aliases
By default, the unit is assumed to have no aliases. Aliases may be specified
by a comma-separated list.

    aliases=microgramme, ug
    
#### 4.2 - conv_factor
The conversion factor must be specified. May be any valid 64-bit floating point
number. Additional tokens or non-numeric tokens will cause errors.

    conv_factor = 1e-6      # base unit is gram. E notation supported.
    conv_factor = gibberish # This is not a number. Error
    conv_factor = 2, 6.3    # Only one token allowed. Error
    
#### 4.3 - dimensions
By default units are assumed to be 1D. The number of dimensions may be any
floating point value between 1 and 255 (though it is highly unlikely you will
ever need more than 3) and will be truncated to the whole number. Additional
tokens or non-numeric tokens will cause errors.

    dimensions = 1          # optional. 1 is the default
    dimensions = gibberish  # This is not a number. Error
    dimensions = 1, 2       # Only one token allowed. Error
    dimensions = 0          # Impossible. Error
    dimensions = 729        # Why do you need this many? Seriously? Error
    
#### 4.4 - inverse
By default units are assumed to be consistent with the corresponding base unit (inverse = 0).
This can be overridden by specifying inverse to be non-zero. Like C, Yucon
defines FALSE as 0 and TRUE is simply anything that isn't false. Thus technically
you can specify inverse to be 42 or even -2147483648.001 and it would have the same
semantics. But for sanity's sake, please just use 0 and 1. Additional tokens or
non-numeric tokens will cause errors.

    inverse = 0          # This is okay. Unit is NOT inverse
    inverse = 1          # This is okay. Unit IS inverse
    inverse = -1337.666  # This is ALSO okay but please, no. Unit IS inverse.
    inverse = gibberish  # This is not a number. Error
    inverse = 0, 0.0     # Only one token allowed. Error
    
#### 4.5 - type
The unit type must be specified. May be one of the following (standard base units
are given along with:

    Type         : Base Unit Used by Yucon
    _____________:_______________________________
    length       : millimetre
    volume       : millilitre
    area         : square centimetre
    energy       : joule
    power        : watt
    mass         : gram
    force        : newton
    torque       : newton meter
    speed        : centimeters per second
    pressure     : pascals
    temperature  : kelvin
    fuel economy : litres per 100 kilometers

Additional tokens or undefined types will cause errors.

    type = mass         # okay
    type = mass, force  # Only one token allowed. Error
    type = notatype     # unrecognized type. Error

#### 4.6 - zero_point
The default zero point is 0 as for most units, 0 is just 0: absence of value.
As we saw with temperatures though, this isn't always the case. May be any valid
64-bit floating point number. Additional tokens or non-numeric tokens will cause
errors.

    zero_point = 0          # optional. default is 0
    zero_point = 0, 1       # only one token allowed. Error
    zero_point = gibberish  # This is not a number. Error
    
#### 4.7 - Our Final Unit
We have been working our way towards defining the microgram unit. We have now
done so.

    [microgram]
    aliases=microgramme, ug
    conv_factor = 1e-6      # base unit is gram. E notation supported.
    dimensions = 1          # optional. 1 is the default
    inverse = 0             # optional. 0 is default. Unit is NOT inverse
    type = mass
    zero_point = 0          # optional. default is 0
    
A sparse definition which ommits the optional fields is equally valid

    [microgram]
    aliases=microgramme, ug
    conv_factor = 1e-6      # base unit is gram. E notation supported.
    type = mass
    
As is one where the key-value pairs are mixed up.

    [microgram]
    zero_point = 0          # optional. default is 0
    conv_factor = 1e-6      # base unit is gram. E notation supported.
    aliases=microgramme, ug
    dimensions = 1          # optional. 1 is the default
    type = mass
    inverse = 0             # optional. 0 is default. Unit is NOT inverse
    
#### 4.8 - Final Notes
* When in doubt, Read **Section 2: General Syntax and Formatting** again and review
  **The Catch-All Syntax Rule**

* If a field is malformed it will be ignored and a warning will be printed to
  to the console. Because Yucon makes best effort, the unit will still be
  considered valid unless we reacha new section and a mandatory field is
  ommitted at which point the unit will be discarded.
  
* Duplicate fields will be rejected and a warning printed to the console.
  Whichever came first is assumed to be correct.
  
  Ex.
  
      [microgram]
      aliases=microgramme, ug
      conv_factor = 1e-6
      type = mass
      type = length           # this will be ignored. Warning printed


