# Yucon - General Purpose Unit Converter
Yucon is a dedicated, feature-rich unit conversion utility built especially for
use on the command line. Why chose Yucon? Here's why:
1. **Easy to use** No tedious hunting and pecking through tiny menus like other
  conversion utilities. Just type what you want.

2. **Extensible** Also unlike other conversion utilities, Yucon's units are not
  embedded. Units can be added to the program by editing a config file.

3. **Powerful** Yucon also provides advanced syntaxes for power users to such
  as recall and runtime metric prefixing.

4. **Freedom** Yucon is free and open source to make it as useful as possible.

Here are some examples of Yucon usage to show you what it's all about:
* Perform conversions just with the program's invocation:

      $ yucon 1 in mm
        25.4 mm

* Run an interactive session to use it like a desktop calculator:

      $ yucon
      YUCON - General Purpose Unit Converter - v0.2
      ====
      This is free software licensed under the GNU General Public License v3
      Type 'version' for more details
      Copyright (C) 2016-2017 Blaine Murphy
      
      Enter a conversion or a command. Type 'help' for assistance.
      
      > 350 cid L
      5.7354724 L
      
      > 707 hp kW
      527.2099000000001 kW
      
      > 222.6 in m
      5.65404 m
      
      > 63 gr _ug
      4082331.33 ug
      
      > format l
      Okay.
      
      > 24 : _m:
      24 gr = 1555.1738400000002 mg

Yucon is still in a Beta stage. More features are planned for future releases.
These include:
* Multiple multiple input values and output units on the same line
* History buffers for recall
* Robust, large scale batch processing

**Yucon officially supports the following environments:**
* Linux
* Windows

## Installation

To install, follow the instructions below.

### For Linux users:
1. Download the 'yucon_v[version#].tar.gz' file (where [version#] is the current
   version or whichever version you want) from the Releases page:
   https://github.com/kmBlaine/yucon/releases/

2. Open up a terminal session, unzip the files, and 'cd' into the
   'yucon_v[version#]' folder.

       $ cd /path/to/yucon_v[version#].tar.gz
       $ tar -xzf yucon_v[version#].tar.gz
       $ cd yucon_v[version#]

3. Run 'install-linux.sh' as root:

       $ sudo ./install-linux.sh

4. Your finished! Type 'yucon' in your terminal to start using.

### For Windows users:
1. Download the 'yucon_v[version#].zip' file (where [version#] is the current
   version or whichever version you want) from the Releases page:
   https://github.com/kmBlaine/yucon/releases/

2. Navigate to the download location. Unzip the files and open the 
   'yucon_v[version#]' folder.

3. Create a folder for Yucon in your preferred location. Suggestions:

       C:\Program Files\yucon\
       C:\Users\[your username]\Desktop\yucon\
       C:\Users\[your username]\yucon\

4. Copy the 'yucon.exe' file in the 'yucon_v[version#]' folder into
   the folder you just created.

5. Copy the 'units.cfg' file in the 'yucon_v[version#]' folder into the same
   folder as yucon.exe.

6. Your finished! Simply double-click 'yucon.exe' to start using.
   * OR open a command prompt or Powershell session cd to where you copied Yucon to:

         C:\Users\John Doe\> cd path\to\yucon\

   * Type 'yucon' to start using.

         C:\path\to\yucon> yucon


## Usage
Basic pointers on usage. This is NOT full documentation. [See the user guide at doc/UserGuide.md for full instructions on use.](https://github.com/kmBlaine/yucon/tree/master/doc/UserGuide.md)

If you would like to add new units to your Yucon install, [see the units.cfg guide at doc/UnitsCFG.md for instructions.](https://github.com/kmBlaine/yucon/tree/master/doc/UnitsCFG.md)

To run:

    yucon [options]
    yucon [options] <#> <input_unit> <output_unit>

In the first form, start an interactive session to use Yucon like a desktop
calculator.

In the second form, perform the conversion specified on the command line and
immediately exit.

Conversion follow a basic format:

    <#> <input_unit> <output_unit>

Where:

- **<#>** is the value to covert
- **<input_unit>** is the unit you know
- **<output_unit>** is the unit you want

You can recall the last value used with the semicolon **;** character:

    > ; in mm

You can recall the last input or output unit sed with the colo **:** character:

    > 6 psi :

You can actively prefix any unit with a metric prefix using the underscore **_**
character:

    > 63 gr _ug
    4082331.33 ug

Program options:
- **-s**\
  Simple formatting for the output. Only the number is displayed.

- **-l**\
  Long formatting for the output. Input value and unit is displayed alongside
  the output value and unit.

- **--version**\
  Displays version and license information and then exits.

- **--help**\
  Displays simple usage instructions and then exits.

## License
**Yucon is licensed under the GNU Public License Version 3**

Copyright (C) 2016-2017 Blaine Murphy

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. [See the GNU General Public License for more details.](https://gnu.org/licenses/gpl.html)

