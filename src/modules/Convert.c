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

/* File: Convert.c
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * This module of the program handles the actual unit conversions and
 * contains the functions for output formatting. This way, all modules
 * and methods needed to do conversion can do so without needing to
 * reimplement the conversions or formatting.
 */

#include "../H/Convert.h"

#include <stdlib.h>
#include <stdio.h>
#include <math.h>
#include <string.h>

/* get_conversion
 *
 * Purpose: given a numeral string and names of units to be converted
 *   converts them or returns error code as appropriate
 *
 * Parameters:
 *   char *number - input string with a value in valid double format
 *                  Ex. "65536", "3.141592654", "6.022E+23"
 *   char *unit_from_name - name of the unit to convert from
 *   char *unit_to_name - name of the unit to convert to
 *
 * Returns: Double - positive on conversion success. negative if error
 */
double get_conversion( char *number, char *unit_from_name, char *unit_to_name, UnitNode* units_list )
{
	char *input_end = NULL;
	double input = strtod( number, &input_end );

	//if trailing characters were found in the number
	if ( input_end && (input_end[1] == NULL_CHAR) )
	{
		return NONNUMERIC_INPUT;
	}

	//if negative, zero, or unrecognized input
	if ( input <= 0 || input == NAN || input == INFINITY )
	{
		return INVALID_INPUT;
	}

	Unit *unit_from = get_unit_by_name( unit_from_name, units_list );
	Unit *unit_to = get_unit_by_name( unit_to_name, units_list );

	//if the units weren't found, return appropriate error
	if ( unit_from == NULL )
	{
		return UNIT_FROM_NF;
	}

	if ( unit_to == NULL )
	{
		return UNIT_TO_NF;
	}

	//if the units types are mistmatched (ie converting volume to length), return error
	if ( unit_from->unit_type != unit_to->unit_type )
	{
		return INCOMPATIBLE_UNITS;
	}

	//else return conversion
	return input * ( unit_from->conversion_factor / unit_to->conversion_factor );
}

char *simple_output_str( double conversion )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g", conversion );

	return str;
}

char *descriptive_output_str( double conversion, char *unit_name )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g %s", conversion, unit_name );

	return str;
}

char *verbose_output_str( double conversion, char *orig_val, char *unit_from_name, char *unit_to_name )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g %s = %g %s", atof( orig_val ), unit_from_name, conversion, unit_to_name );

	return str;
}
