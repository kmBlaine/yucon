# User Guide
Full instructions on usage of Yucon

## 1 - Running Yucon
Yucon is a console application and so it is usually run from a console like
Bash on Linux or Command Prompt / Powershell on Windows. The command to run
Yucon from the console takes one of these forms:

    1. $ yucon [options] <#> <input_unit> <output_unit>
    2. $ yucon [options]

In the first form, Yucon is run in **Single Use Mode**. In single use mode, a
single conversion is performed which is given in the command that invokes the
program. Yucon will perform the conversion and return to the console or return
an error. The full command will need to be entered for each desired
conversion.
- **[options]:** options to run Yucon with (if any)
- **<#>:** the value to be converted
- **<input_unit>:** the unit being converted from
- **<output_unit>:** the unit being converted to

In the second form, Yucon is run in **Interactive Mode**. In interactive mode,
Yucon takes over the console and interprets every command entered as conversion.
There is no need to type "yucon" and respecify options for each conversion in
interactive mode. In addition, interactive mode allows for recall of units and
values.

### 1.1 - Options
- **-s**\
  Simple formatting for the output. Only the number is displayed.

- **-l**\
  Long formatting for the output. Input value and unit is displayed alongside
  the output value and unit.

- **--version**\
  Displays version and license information and then exits.

- **--help**\
  Displays simple usage instructions and then exits.

## 2 - Conversion Syntax
All conversions, whether they are entered in single use mode or in interactive
mode follow this format:

    <#> <input_unit> <output_unit>

Where:

- **<#>:** a value or value expression to be converted
- **<input_unit>:** a unit or unit expression being converted from
- **<output_unit>:** a unit or unit expression being converted to

The reason there is a distinction between values / units and value / unit
expressions is because Yucon has several convenience features which allow for
things that would otherwise be very hard, redundant, or laborious to do or
incorporate into the program. These features are namely:

- **Recall**\
  Recall allows you to instantly reuse the last used value or units without
  retyping them.

- **Runtime Metric Prefixing**
  Runtime metric prefixing allows any unit to be prefixed with a metric prefix
  such as "milli" or "kilo" without needing an additional entry in the units.cfg
  file and associated aliases, even non-metric units. *Note that extremely common
  units such as centimeters or kilopascals are already included in units.cfg for
  convenience.*

Spaces are used to tokenize the conversion into values and units meaning that if
there is a space in the unit names, they must be escaped with a blackslash **\\**
Ex:

    > 428 cubic\ inch L
    7.013663392 L

### 2.1 - Recall
Recall allows for the reuse of the your last use value or unit without retyping
it. Recall is performed for values by typing a semicolon **;** instead of a number.
Recall is performed for a unit by typing a colon **:** instead of a unit. *NOTE:
the reason the recall character is different for values and units is in anticipation
of unimplemented features; using a different recall character will make parsing
significantly easier.*

When the program initially starts, the recalls are not set meaning that for
the very first conversion, attempting recall will result in an error. The
corollary to this is that recall is impossible when Yucon is run in single use
mode.

Recall for values is set whenever a valid literal number is entered even if the
overall conversion fails. The recall will persist indefinitely until a new
literal number is entered.

Recall for units is set whenever a corresponding entry in units.cfg is found for
a literal unit alias even if the overall conversion fails. The recall will
persist indefinitely until a new literal unit alias is successfully located in
the units.cfg file. It is important to note that the alias itself AND ONLY the
alias is used for performing recall which means that metric prefixing (if any)
is NOT preserved. This is intended to allow indefinite prefixing of the same
base unit without confusing accumulation of metric scalars. Ex:

    > 7790 m/s _cm/s
    779000 cm/s
    
    > ; : _k:
    7.79 km/s
    
    > ; : _m:
    7790000 mm/s

### 2.2 - Runtime Metric Prefixing
Runtime metric prefixing allows any unit to be scaled with a standard metric
prefix even if the scaled version is not present in the units.cfg file. This
means entries in the units.cfg file do not need to exist for every possible
combination of metric scalars. However, the most common ones such as centimeters
or kilograms are entered for convenience.

Runtime metric prefixing is done using the underscore **_** character. The
character immediately following the underscore is interpreted as the metric
prefix. The following metric prefixes are supported and mapped to the following
characters:

Big (descending):

    Yotta - Y
    Zeta  - Z
    Exa   - E
    Peta  - P
    Tera  - T
    Giga  - G
    Mega  - M
    Kilo  - k
    hecto - h
    deca  - D
    

Small (descending):

    deci  - d
    centi - c
    milli - m
    micro - u
    nano  - n
    pico  - p
    femto - f
    atto  - a
    zepto - z
    yocto - y

Metric prefixing may be used together with recall:

    > 7790 m/s _cm/s
    779000 cm/s
    
    > ; : _k:
    7.79 km/s

### 2.3 - Value Expressions
When entering values into Yucon, it is either a literal number or the recall
character:

    > 428
    ...
    > ;
    ...

Value recall is done using the semicolon **;**, as above. Remember that the
first conversion given cannot use value recall.

### 2.4 - Unit Expressions
When entering units into Yucon, they take one of the following forms:

    1. [_<prefix_char>]unit_alias
    2. [_<prefix_char>]:

In the first form, a literal unit alias / name is given to search the units.cfg
file for with an optional metric prefix given by an underscore **_** and the
character immediately following it. Errors will occur in the following cases:
- No prefix character or unit alias is given after an underscore
- The prefix character after an underscore is not recognized
- No unit alias is given after the metric prefix

Note that the underscore **_** character is a trigger for metric prefixing
meaning that if it is used anywhere in a literal unit alias it must be escaped
by a backslash **\\**:

    > 123 in example\_unit

In the second form, a recall of the last used unit is performed via the colon
**:** character with an optional metric prefix. The same rules for metric
prefixing above apply. Additionally, the colon **:** acts as a trigger for
recall. If it is used anywhere in a literal unit alias, it must be escaped by a
backslash **\\**:

    > 123 in example\:unit

## 3 - Program Commands
When Yucon is run in interactive mode, it understands several commands apart
from typical conversions. These commands modify the behavior or parameters of
the program such as the output format or what the recall units are. The commands
are as follows:

- **exit**\
  Exits the program
- **help**\
  Displays simple usage instructions
- **version**\
  Displays version and license info
- **\<var\> \[\<state\>\]**\
  Displays or sets a program variable

Note that ALL COMMANDS and PROGRAM VARIABLES are exclusively reserved keywords.
No unit name may EVER be one of these.

### 3.1 - Viewing and Setting Program Variables
Viewing and setting program variables is accomplished by typing the name of a
program variable to view it and following it up with a state to set it. If the
variable is not recognized, an error will occur. If the state is invalid, an
error will occur and the old state will be retained.

The following are recognized variables:

- **format**\
  Controls the output format. Valid states are \'s\', \'d\', and \'l\'
- **value**\
  The recall value for conversions. When setting, a literal number must be supplied
  as the state. Embedded recall is not allowed.
- **input_unit**\
  The recall input unit for conversions. When setting, a literal unit alias must be
  supplied as the state. Embedded recall and metric prefixing is not allowed.
- **output_unit**\
  The recall output unit for conversions. When setting, a literal unit alias must be
  supplied as the state. Embedded recall and metric prefixing is not allowed.

