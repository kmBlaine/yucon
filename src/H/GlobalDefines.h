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
/*
 * GlobalDefines.h
 *
 *  Created on: Nov 19, 2016
 *      Author: kbm1271
 */

#ifndef H_GLOBALDEFINES_H_
#define H_GLOBALDEFINES_H_

#define LENGTH 0
#define VOLUME 1
#define AREA   3
#define ENERGY 4
#define POWER  5
#define MASS   6
#define FORCE  7
#define TORQUE 8

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
 */
#define NOT_ENOUGH_ARGS    1
#define UNRECOGNIZED_ARG   2
#define TOO_MANY_ARGS      3
#define HELP_REQUESTED     4
#define NONNUMERIC_INPUT   -1
#define INVALID_INPUT      -2
#define UNIT_FROM_NF       -3
#define UNIT_TO_NF         -4
#define INCOMPATIBLE_UNITS -5
#define OUTPUT_FILE_ERR    5

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
//standard unit type names
static const char *length = "length";
static const char *volume = "volume";
static const char *area = "area";
static const char *energy = "energy";
static const char *power = "power";
static const char *mass = "mass";
static const char *force = "force";
static const char *torque = "torque";
#pragma GCC diagnostic pop

#endif /* H_GLOBALDEFINES_H_ */
