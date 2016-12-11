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

#include "GlobalDefines.h"
#include "Convert.h"
#include "UnitList.h"

#include <stdlib.h>
#include <stdio.h>
#include <math.h>
#include <string.h>

#define RESET          0
#define RETURN_STATE   1
#define GET_PREFIX     2
#define PREFIX_SET     3
#define GET_LAST_UNIT  4

double get_prefix_value( char prefix )
{
	switch ( prefix )
	{
	case 'Y':
		return 1e+24;

	case 'Z':
		return 1e+21;

	case 'E':
		return 1e+18;

	case 'P':
		return 1e+15;

	case 'T':
		return 1e+12;

	case 'G':
		return 1e+9;

	case 'M':
		return 1e+6;

	case 'k':
		return 1e+3;

	case 'h':
		return 1e+2;

	case 'D':
		return 1e+1;

	case 'd':
		return 1e-1;

	case 'c':
		return 1e-2;

	case 'm':
		return 1e-3;

	case 'u':
		return 1e-6;

	case 'n':
		return 1e-9;

	case 'p':
		return 1e-12;

	case 'f':
		return 1e-15;

	case 'a':
		return 1e-18;

	case 'z':
		return 1e-21;

	case 'y':
		return 1e-24;

	default:
		return -1;
	}
}


int check_escape_sequences( char **str, double *prefix )
{
	int state = RESET;
	int error_code = 0;
	int pos = 0;

	while ( state != RETURN_STATE )
	{
		switch ( state )
		{
		case RESET:
			if ( str[0][pos] == ':' )
			{
				state = GET_LAST_UNIT;
			}
			else if ( str[0][pos] == '_' )
			{
				state = GET_PREFIX;
			}
			else
			{
				state = RETURN_STATE;
			}
			break;

		case GET_PREFIX:
			*prefix = get_prefix_value( str[0][pos] );

			if ( *prefix < 0 )
			{
				error_code = UNKNOWN_PREFIX;
				state = RETURN_STATE;
			}
			else
			{
				state = PREFIX_SET;
			}
			break;

		case PREFIX_SET:
			if ( str[0][pos] == ':' )
			{
				state = GET_PREFIX;
			}
			else if ( str[0][pos] == NULL_CHAR )
			{
				error_code = NO_NAME_GIVEN;
				state = RETURN_STATE;
			}
			else
			{
				state = RETURN_STATE;
			}
			break;

		case GET_LAST_UNIT:
			error_code = RECALL_LAST;

			if ( str[0][pos] != NULL_CHAR )
			{
				error_code = NO_NAME_ALLOWED;
			}

			state = RETURN_STATE;
			break;
		}

		pos++;
	}

	*str += --pos; //skip the prefix so the name may be passed directly to get_unit_by_name()

	return error_code;
}

/* get_conversion
 *
 * Purpose: given a numeral string and names of units to be converted
 *   converts them or returns error code as appropriate
 *
 * Parameters:
 *   char *number - input string with a value in valid double format
 *                  Ex. "65536", "3.141592654", "6.022E+23"
 *   char *input_unit_name - name of the unit to convert from
 *   char *output_unit_name - name of the unit to convert to
 *   double *conversion - pointer to double to write value into
 *
 * Returns: Int - 0 on success. Nonzero on error.
 */
int get_conversion( char *number, char *input_unit_name, char *output_unit_name, double *conversion )
{
	static double last_number;
	static Unit *last_input_unit;
	static Unit *last_output_unit;

	//char *input_end = NULL;
	double input = strtod( number, NULL );

	//if negative, zero, out of range or unrecognized input
	if ( (input == NAN) || (input == INFINITY) )
	{
		return INVALID_INPUT;
	}

	double input_prefix = 1;
	double output_prefix = 1;
	Unit *input_unit = NULL;
	Unit *output_unit = NULL;

	int error_code = check_escape_sequences( &input_unit_name, &input_prefix );
	if ( error_code == RECALL_LAST )
	{
		if ( last_input_unit == NULL )
		{
			return INPUT_UNIT_UNSET;
		}

		input_unit = last_input_unit;
	}
	else if ( error_code )
	{
		return error_code;
	}
	else
	{
		input_unit = get_unit_by_name( input_unit_name );
	}

	error_code = check_escape_sequences( &output_unit_name, &output_prefix );
	if ( error_code == RECALL_LAST )
	{
		if ( last_output_unit == NULL )
		{
			return OUTPUT_UNIT_UNSET;
		}

		output_unit = last_output_unit;
	}
	else if ( error_code )
	{
		return error_code;
	}
	else
	{
		output_unit = get_unit_by_name( output_unit_name );
	}

	//if the units weren't found, return appropriate error
	if ( input_unit == NULL )
	{
		return UNIT_FROM_NF;
	}

	if ( output_unit == NULL )
	{
		return UNIT_TO_NF;
	}

	//if the units types are mistmatched (ie converting volume to length), return error
	if ( input_unit->unit_type != output_unit->unit_type )
	{
		return INCOMPATIBLE_UNITS;
	}

	//else return conversion
	*conversion = (( input * input_prefix + input_unit->offset )
			* ( input_unit->conversion_factor / output_unit->conversion_factor )
			- (output_unit->offset))
			/ output_prefix;

	last_input_unit = input_unit;
	last_output_unit = output_unit;

	return EXIT_SUCCESS;
}

char *simple_output_str( double conversion )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g\n", conversion );

	return str;
}

char *descriptive_output_str( double conversion, char *unit_name )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g %s\n", conversion, unit_name );

	return str;
}

char *verbose_output_str( double conversion, char *orig_val, char *input_unit_name, char *output_unit_name )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g %s = %g %s\n", atof( orig_val ), input_unit_name, conversion, output_unit_name );

	return str;
}
