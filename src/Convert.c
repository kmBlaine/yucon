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

//valid states for the finite state machine in check_escape_sequences()
#define RESET          0
#define RETURN_STATE   1
#define GET_PREFIX     2
#define PREFIX_SET     3
#define GET_LAST_UNIT  4

static double last_number;
static char *last_input_name;
static char *last_output_name;

static char *get_input_unit = "input unit";
static char *get_output_unit = "output unit";

/* get_prefix_value
 *
 * Purpose: returns the numerical constant associated with a valid metric prefix
 *
 * Paramters:
 *   char prefix - prefix character. CASE SENSITIVE!
 *
 * Returns: Numerical prefix on success. -1 on failure.
 */
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

/* check_escape_sequences
 *
 * Purpose: interprets escape sequences in the unit strings such as
 *   metric prefixing and recall last. adjusts str to ignore the metric
 *   prefix so it may be copied to the recall last storage later.
 *
 * Parameters:
 *   char **str - pointer to original str pointer. done this way so the
 *                original pointer may be modified to occlude the prefix
 *
 *   double *prefix - pointer to double
 *
 * Returns: 0 on exit success. Nonzero on error.
 */
int check_escape_sequences( char **str, double *prefix )
{
	int state = RESET;
	int error_status = 0;
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
				error_status = UNKNOWN_PREFIX;
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
				state = GET_LAST_UNIT;
			}
			else if ( str[0][pos] == NULL_CHAR )
			{
				error_status = NO_NAME_GIVEN;
				state = RETURN_STATE;
			}
			else
			{
				*str += 2; //change beginning of str to ommit prefix
				state = RETURN_STATE;
			}
			break;

		case GET_LAST_UNIT:
			error_status = RECALL_LAST;

			if ( str[0][pos] != NULL_CHAR )
			{
				error_status = NO_NAME_ALLOWED;
			}

			state = RETURN_STATE;
			break;
		}

		pos++;
	}

	return error_status;
}

void copy_name_for_recall( char *input, char **storage )
{
	if ( *storage )
	{
		free( *storage );
	}

	*storage = calloc( strlen(input) + 1, sizeof(char) );

	strcpy( *storage, input );
}

/* get_conversion
 *
 * Purpose: given a numeral string and names of units to be converted
 *   converts them or returns error code as appropriate
 *
 * Parameters:
 *   char *number - input string with a value in valid double format
 *                  Ex. "65536", "3.141592654", "6.022E+23"
 *                  May also use 'recall last' function ':'
 *
 *   char *input_unit_name - name of the unit to convert from
 *                  may optionally specify a metric prefix in form "_p"
 *                  and 'recall last' function ':'
 *                  these special sequences may be used together
 *                  Ex. "_d:" --> deci[last_unit]
 *
 *   char *output_unit_name - name of the unit to convert to
 *                  may optionally specify a metric prefix in form "_p"
 *                  and may use 'recall last' function ':'
 *                  these special sequences may be used together
 *                  Ex. "_d:" --> deci[last_unit]
 *
 *   double *conversion - pointer to double to write value into
 *
 * Returns: Int - 0 on success. Nonzero on error.
 */
int get_conversion( char *number, char *input_unit_name, char *output_unit_name, double *conversion )
{
	double input = last_number;

	if ( number[0] != ':' )
	{
		input = strtod( number, NULL );
		last_number = input;
	}

	//if input is out of range
	if ( (input == NAN) || (input == INFINITY) )
	{
		return INVALID_INPUT;
	}

	double input_prefix = 1;
	double output_prefix = 1;
	Unit *input_unit = NULL;
	Unit *output_unit = NULL;

	error_code = check_escape_sequences( &input_unit_name, &input_prefix );
	if ( error_code == RECALL_LAST )
	{
		input_unit = get_unit_by_name( input_unit_name, INPUT_UNIT );

		if ( input_unit == NULL )
		{
			error_msg = get_input_unit; //preemptively set error point for external help function
			return RECALL_UNSET;
		}
	}
	else if ( error_code )
	{
		error_msg = input_unit_name;
		return error_code;
	}
	else
	{
		input_unit = get_unit_by_name( input_unit_name, INPUT_UNIT );
	}
	//if the units weren't found, return appropriate error
	if ( input_unit == NULL )
	{
		error_msg = input_unit_name;
		return UNIT_NF;
	}
	if ( error_code != RECALL_LAST )
	{
		copy_name_for_recall( input_unit_name, &last_input_name );
	}


	error_code = check_escape_sequences( &output_unit_name, &output_prefix );
	if ( error_code == RECALL_LAST )
	{
		output_unit = get_unit_by_name( output_unit_name, OUTPUT_UNIT );

		if ( output_unit == NULL )
		{
			error_msg = get_output_unit;
			return RECALL_UNSET;
		}
	}
	else if ( error_code )
	{
		error_msg = output_unit_name;
		return error_code;
	}
	else
	{
		output_unit = get_unit_by_name( output_unit_name, OUTPUT_UNIT );
	}
	//if unit was not found, return an error
	if ( output_unit == NULL )
	{
		error_msg = output_unit_name;
		return UNIT_NF;
	}
	if ( error_code != RECALL_LAST )
	{
		copy_name_for_recall( output_unit_name, &last_output_name );
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

	return EXIT_SUCCESS;
}

/* build_unit_str
 *
 * Purpose: when using special sequence interpolation, the input and output
 *   unit names will be cryptic. This function adjusts or builds new output
 *   name strings to eliminate the escape sequences. Eliminates the need to
 *   do complicated format logic in the output string generator functions to
 *   account for the escape sequences.
 *   WARNING: FUNCTION USES CALLOC! WHEN THIS FUNCTION RETURNS A VALID POINTER
 *   IT MUST BE FREED AFTER USE!
 *   Example of adjusting a string:
 *     unit_str points "_dm" in this string
 *                      ^here
 *     build_unit_str will adjust unit_str to point "_dm"
 *                                                     ^here
 *     returns NULL in this case
 *
 *   Example of rebuilding a string:
 *     unit_str is "_k:" and last unit was "newton"
 *     build_unit_str will create new string "knewton" and return that
 *
 * Parameters:
 *   char **unit_str - pointer to string. this allows function to adjust the
 *                     original pointer to eliminate the _ escape sequence
 *
 *   int which - determines wether the function is adjusting the input unit or
 *                     output unit string. Use macros in UnitList.h
 *
 * Returns: Pointer to string if was completely rebuilt. NULL if it was only adjusted
 */
char *build_unit_str( char **unit_str, int which )
{
	char *last_unit = NULL;

	//determine which of the units to recall
	if ( which == INPUT_UNIT )
	{
		last_unit = last_input_name;
	}
	else
	{
		last_unit = last_output_name;
	}

	//occlude underscore when using metric prefixing
	if ( unit_str[0][0] == '_' )
	{
		*unit_str += 1;
	}

	//simply recalling without prefix, set unit_str to the recall str
	if ( unit_str[0][0] == ':' )
	{
		*unit_str = last_unit;
	}
	//if there is a prefix, rebuild string with prefix and return it
	else if ( (unit_str[0][1] == ':') )
	{
		//additional space for null terminator and prefix
		char *new_unit_str = calloc( strlen(last_unit) + 2, sizeof(char) );

		new_unit_str[0] = unit_str[0][0]; //copy over prefix

		//don't copy over the prefix
		strcpy( new_unit_str + 1, last_unit );

		return new_unit_str;
	}

	return NULL;
}

/* simple_output_str
 *
 * Purpose: builds a console output string in the simple format (no units)
 *   WARNING: FUNCTION USES CALLOC. Generated string MUST BE FREED AFTER USE
 *
 * Parameters:
 *   double conversion - final conversion value
 *
 * Returns: Pointer to output str
 */
char *simple_output_str( double conversion )
{
	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g\n", conversion );

	return str;
}

/* descriptive_output_str
 *
 * Purpose: builds a console output string in the descriptive format
 *   (including output unit). WARNING: FUNCTION USES CALLOC! Generated
 *   string MUST BE FREED AFTER USE!
 *
 * Parameters:
 *   double conversion - final conversion value
 *   char *unit_name - raw output unit name string (include escape sequences)
 *
 * Returns: Pointer to output str
 */
char *descriptive_output_str( double conversion, char *unit_name )
{
	char *output_name_rebuilt = build_unit_str( &unit_name, OUTPUT_UNIT );

	//if the output unit had to be interpolated for escape sequences
	if ( output_name_rebuilt )
	{
		unit_name = output_name_rebuilt;
	}

	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g %s\n", conversion, unit_name );

	if ( output_name_rebuilt )
	{
		free( output_name_rebuilt );
	}

	return str;
}

/* verbose_output_str
 *
 * Purpose: builds a console output string in the verbose format
 *   (including input and output unit). WARNING: FUNCTION USES CALLOC! Generated
 *   string MUST BE FREED AFTER USE!
 *
 * Parameters:
 *   double conversion - final conversion value
 *   char *input_unit_name - raw input unit name string (including escape sequences)
 *   char *input_unit_name - raw output unit name string (including escape sequences)
 *
 * Returns: Pointer to output str
 */
char *verbose_output_str( double conversion, char *orig_val, char *input_unit_name, char *output_unit_name )
{
	double number = last_number;

	if ( orig_val[0] != ':' )
	{
		number = atof( orig_val );
	}

	char *input_name_rebuilt = build_unit_str( &input_unit_name, INPUT_UNIT );
	char *output_name_rebuilt = build_unit_str( &output_unit_name, OUTPUT_UNIT );

	if ( input_name_rebuilt )
	{
		input_unit_name = input_name_rebuilt;
	}

	if ( output_name_rebuilt )
	{
		output_unit_name = output_name_rebuilt;
	}

	char *str = calloc( OUTPUT_STR_SIZE, sizeof(char) );

	sprintf( str, "%g %s = %g %s\n", number, input_unit_name, conversion, output_unit_name );

	if ( input_name_rebuilt )
	{
		free( input_name_rebuilt );
	}

	if ( output_name_rebuilt )
	{
		free( output_name_rebuilt );
	}

	return str;
}

/* delete_recall_data
 *
 * Purpose: deletes the stored name strings for use with 'recall last' function
 *   THIS FUNCTION SHOULD BE CALLED WHEN THE PROGRAM EXITS TO PREVENT MEMORY LEAK!
 *
 * Parameters: none
 *
 * Returns: nothing
 */
void delete_recall_data()
{
	if ( last_input_name ) free( last_input_name );
	if ( last_output_name ) free( last_output_name );
}
