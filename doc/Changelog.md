# Change Log
Version history of Yucon.

## Beta Releases

---
### **v0.2**
Complete rewrite in the Rust programming language. Released xx Nov 2017

#### Summary
The C lang version was more of a hack than a well-designed application. A rewrite
was necessary in order to make the project work correctly even in its current form.
Further, C lacks memory safety, high level abstractions / data structures, good
platform agnosticism, and Unicode support, all of which Rust supports by its
specification. Transition to Rust was only logical. (also, I wanted to learn a
new programming language xD --kmBlaine)

In the process, the syntaxes were strictly defined, improvements were made to the
usability, and the parsing was vastly improved. The biggest improvement on this
front was the implementation of escape sequences allowing for all characters to
be used anywhere in unit names.

#### Fixes:
* Fixed a bug where units square and cubic units did not scale correctly when
  given runtime metric prefixes. Ex:

      > 1 _cm3 cm3
        10000 cm3   # the answer should be 1

#### Changes:
* Far more robust syntax and parsing for units.cfg file and command interpreter
* units.cfg file syntax completely redefined. See yucon/doc/UnitsCFG.md for more
  details. Highlights:
  * Spaces may now be used in unit names
  * Yucon control characters (ie : , _ = ;) may be used unit names
  * Order of tag fields like "type" or "conv_factor" no longer matters
  * Inline comments with the \'#\' character
* Recalling last used value now done with the semicolon \';\' character. Done in
  anticipation of future features
* \'set\' and \'view\' commands in the interpreter deprecated. Now typing the
  name of a variable will simply display it and the name of a variable followed
  by a new state changes it. Ex:

      > format
        d: descriptive / value and output unit
      
      > format s
        Okay.

* Lots of back-end changes for performance

#### Additions:
* Full support for inverse units like fuel economies. Conversion between miles
  per gallon and litres per 100km possible

#### Removals:
* -d command line option since \'descriptive\' is now the default
* Batch processing via piping or files and by extension the -b and -o options.
  *The way that this is planned to be reimplemented differs significantly from
  v0.1 and requires a lot of forethought and coding. A straight port would just
  result in many lines of code being ripped up one release later. Thus, this
  feature was postponed.*
* Shorthand --help option (-h)

---
### **v0.1.1**
Incremental feature update. Released 22 Apr 2017

#### Changes:
* Default output format is now 'descriptive' (-d)
* Help messages revamped to deliver more informative error messages

#### Additions:
* Option for simple output format: -s
* Partial support for fuel economy; calculable in units with dimension of
  [volume/distance] such as L/100km or gal/mi. IMPORTANT: FUEL ECONOMIES
  IN DIMENSION [distance/volume] SUCH AS MPG ARE NOT YET SUPPORTED! (sorry
  U.S.)
* 'set' and 'view' commands in Interactive mode which allow setting and
  viewing of certain program variables like output format or recall units
  dynamically.

---
### **v0.1**
Initial Beta release. Released 12 Dec 2016

#### Base feature set implemented:
* Direct conversion of units on the command line
* Batch conversion of units from file or pipe
* Interactive mode for converting units
* Loads all units from units.dat file
* Support for following unit types: length, volume, area, energy, power,
  mass, force, torque, speed, pressure, temperature
* Support for metric prefixing
* Support for recall when using batch and interactive mode