/*
Yucon - General purpose unit converter
    Copyright (C) 2016 - Blaine Murphy

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

/* File: GlobalDefines.h
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * This file contains the global definitions of the program that are shared
 * across nearly all modules. This includes things like definitions,
 * simple macros, and constants. The intent is to maintain consistency for commonly
 * used definitions and constants like error codes or names across all files.
 * However, it should not be used for variables and type definitions. These should be
 * included in the header/file for the module that they are most closely associated
 * with.
 */

#ifndef H_GLOBALDEFINES_H_
#define H_GLOBALDEFINES_H_

//main program operating modes
#define ONE_TIME_MODE      0
#define BATCH_MODE         1
#define INTERACTIVE_MODE   2

#define NULL_CHAR         '\0'
#define VERSION           "v0.1.1"
#define PROGRAM_TITLE     "YUCON - General Purpose Unit Converter - "VERSION"\n"
#define RELEASE_DATE      "24 Dec 2016"
#define COPYRIGHT_NOTICE  "Copyright (C) 2016 Blaine Murphy\n"

/* Error Code defines
 *
 * NOT_ENOUGH_ARGS  - not enough arguments supplied
 * UNRECOGNIZED_ARG - invalid argument / option
 * TOO_MANY_ARGS    - too many args for operation mode
 * HELP_REQUESTED   - user supplied the -h / --help option
 * NONNUMERIC_INPUT - user supplied a nonnumeric input value
 * INVALID_INPUT    - user supplied a zero, negative, or nonnumeric input value
 * UNIT_FROM_NF     - converting from unknown unit
 * UNTI_TO_NF       - converting to unknown unit
 * INCOMPATIBLE_UNITS - unit types are mismatched ie attempting to convert volume to length
 * OUTPUT_FILE_ERR    - output file specified could not be opened
 * UNITS_FILE_MISSING - units.dat file not found in /etc/yucon/ or is corrupt
 * INPUT_FILE_ERR     - input file not found or unreadable
 * FILE_OUTPUT_NOT_ALLOWED - user attempted to enable file output in interactive mode
 * UNKNOWN_PREFIX     - user specified an unknown metric prefix for a unit
 * NO_NAME_GIVEN      - no unit name was given after prefix
 * NO_NAME_ALLOWED    - a unit name was specified after the 'recall last' character ':'
 */
#define NOT_ENOUGH_ARGS    1
#define UNRECOGNIZED_ARG   2
#define TOO_MANY_ARGS      3
#define NONNUMERIC_INPUT   4
#define INVALID_INPUT      5
#define UNIT_NF            6
#define INCOMPATIBLE_UNITS 7
#define OUTPUT_FILE_ERR    8
#define UNITS_FILE_MISSING 9
#define INPUT_FILE_ERR     10
#define FILE_OUTPUT_NOT_ALLOWED 11
#define UNKNOWN_PREFIX     12
#define NO_NAME_GIVEN      13
#define NO_NAME_ALLOWED    14
#define RECALL_UNSET       15

/* INTERNAL COMMAND DEFINES
 *
 * THe program uses a set of internal commands at various points
 * particularly for the interactive mode. These will be negative
 * by convention to distinguish them from error codes which are
 * positive.
 */
#define HELP_REQUESTED    -1
#define EXIT_PROGRAM      -2
#define TRY_ARGS_CONVERT  -3
#define VERSION_REQUESTED -4
#define RECALL_LAST       -5

//NUMERIC CONSTANTS FOR STANDARD UNIT TYPES
#define LENGTH   0
#define VOLUME   1
#define AREA     3
#define ENERGY   4
#define POWER    5
#define MASS     6
#define FORCE    7
#define TORQUE   8
#define SPEED    9
#define PRESSURE 10
#define TEMP     11

extern int error_code;
extern char *error_msg;

//STRING CONSTANTS FOR STANDARD UNIT TYPE NAMES
//GCC will issue warnings for these definitions although they are valid
//and used in other modules. Temporarily disable unused variable warnings
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
extern const char *length;
extern const char *volume;
extern const char *area;
extern const char *energy;
extern const char *power;
extern const char *mass;
extern const char *force;
extern const char *torque;
extern const char *speed;
extern const char *pressure;
extern const char *temp;
//enable unused variable warnings
#pragma GCC diagnostic pop

//GENERIC NOT IMPLEMENTED WARNING
#define FUNCTION_NOT_IMPLEMENTED(NAME) printf("%s: This function is not implemented yet.",NAME)

#endif /* H_GLOBALDEFINES_H_ */
