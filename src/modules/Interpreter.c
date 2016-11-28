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

/* File: Interpreter.c
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRITPTION:
 *
 * This module handles the interpretation of arguments given to the program
 * on the command line and implements conversion routines to match. This
 * partially overlaps with the task of the main() method but
 * the Interpreter is both the back end and the principle determinant of the
 * program's behavior. The intent is mainly to keep the main file as clean
 * as possible and make the program easily extensible.
 */

#include "../H/Interpreter.h"
#include "../H/UnitList.h"
#include "../H/Convert.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#define STDOUT_MODE        0
#define VERBOSE_MODE       1
#define QUIET_MODE         2
#define SIMPLE_FORMAT      0
#define DESCRIPTIVE_FORMAT 1
#define VERBOSE_FORMAT     2

/* get_type_str
 *
 * Purpose: given the internal type code for a unit, returns a string
 *   with the English name of the unit type
 *
 * Parameters:
 *   int unit_type - internal type code for a unit
 *
 * Returns: char* - pointer to read-only string containing english name
 */
const char *get_type_str( int unit_type )
{
	switch ( unit_type )
	{
	case LENGTH:
		return length;

	case VOLUME:
		return volume;

	case AREA:
		return area;

	case ENERGY:
		return energy;

	case POWER:
		return power;

	case MASS:
		return mass;

	case FORCE:
		return force;

	case TORQUE:
		return torque;

	case SPEED:
		return speed;

	case PRESSURE:
		return pressure;

	case TEMP:
		return temp;

	default:
		return length;
	}
}


/* set_program_options
 *
 * Purpose: parses the program options from the command line args and puts
 *   them into the ProgramOptions struct. returns error code if args are
 *   invalid
 *
 * Parameters:
 *   ProgramOptions *options - options struct to write into
 *   int argc - arguments count
 *   char *argv[] - array of command line arguments
 *
 * Valid Options:
 *   -b   - batch mode
 *   -o   - output to file and stdout (default behavior)
 *     q  - suboption. output to file only (quiet behavior)
 *     whatever arg follows is the name of the output file
 *   -d   - descriptive outputs ( 1.5mm instead of 1.5 )
 *   -v   - verbose outputs ( 1 in = 25.4 mm )
 *   -h,--help
 *
 * Returns: Int:
 *   0 - success. no errors
 *   NONZERO - errors in input and options. see Interpreter.h for details
 */
int set_program_options( ProgramOptions *options, int argc, char *argv[] )
{
	//assume default options
	options->argc = argc;
	options->argv = argv;
	options->input_mode = ONE_TIME_MODE;
	options->output_mode = STDOUT_MODE;
	options->format = SIMPLE_FORMAT;

	//if any option is -h or --help, return HELP error code
	for ( int arg = 1; arg < argc; arg++ )
	{
		if ( (strcmp( argv[arg], "-h" ) == 0) || (strcmp( argv[arg], "--help") == 0) )
		{
			return HELP_REQUESTED;
		}
	}

	//check all args
	for ( int arg = 1; arg < argc; arg++ )
	{
		options->last_arg = argv[arg];

		//filter out dash args
		if ( argv[arg][0] == '-' )
		{
			//if batch mode, set batch option
			if ( strcmp( argv[arg], "-b" ) == 0 )
			{
				options->input_mode = BATCH_MODE;
			}
			//if verbose output file mode
			else if ( strcmp( argv[arg], "-o") == 0 )
			{
				options->output_mode = VERBOSE_MODE;
				//if there are enough args, set next arg to filename
				if ( arg < (argc - 1) ){ options->input_file = argv[++arg]; }
				else { return NOT_ENOUGH_ARGS; } //else return error
			}
			//if quiet output file mode
			else if ( strcmp( argv[arg], "-oq" ) == 0 )
			{
				options->output_mode = QUIET_MODE;
				//if there are enough args, set next arg to filename
				if ( arg < (argc - 1) ){ options->input_file = argv[++arg]; }
				else { return NOT_ENOUGH_ARGS; }
			}
			//if descriptive format, set descriptive format option
			else if ( strcmp( argv[arg], "-d" ) == 0 )
			{
				options->format = DESCRIPTIVE_FORMAT;
			}
			//if verbose format, set verbose format option
			else if ( strcmp( argv[arg], "-v" ) == 0 )
			{
				options->format = VERBOSE_FORMAT;
			}
			else
			{
				return UNRECOGNIZED_ARG; //else unrecognized arg. error
			}
		}
		else //else if non dash argument
		{
			if ( options->input_mode == BATCH_MODE )
			{
				//if exactly one argument left, interpret as input file name
				if ( arg == (argc - 1) )
				{
					options->input_file = argv[arg];
				}
				//loop will not reach this point if args left = 0
				//if we reached this point, too many args
				else
				{
					return TOO_MANY_ARGS;
				}
			}
			//if in one time mode, and three args left, try to convert
			else if ( (argc - arg) == 3 )
			{
				break;
			}
			//if there are more than three args, not possible. too many args
			else if ( (argc - arg) > 3 )
			{
				return UNRECOGNIZED_ARG;
			}
			else
			{
				return NOT_ENOUGH_ARGS;
			}
		}
	}

	return EXIT_SUCCESS;
}

/* help
 *
 * Purpose: provides the user with basic information on the
 *   program's operation and any errors that arise when in
 *   use.
 *
 * Parameters:
 *   int error_code - internal code for the runtime error. see GlobalDefines.h
 *   ProgramOptions *options - options that the program was run with
 *   UnitNode *units_list - head of the list of units
 *
 * Returns: nothing
 */
void help( int error_code, ProgramOptions *options )
{
	if ( error_code != HELP_REQUESTED )
	{
		printf( "Error: ");
	}

	switch ( error_code )
	{
	case NOT_ENOUGH_ARGS:
		if ( options->output_mode )
		{
			printf( "%s: expected output file name\n\n", options->last_arg );
		}
		else
		{
			printf( "expected a unit conversion. Not enough arguments\n\n" );
		}
		break;

	case UNRECOGNIZED_ARG:
		printf( "unrecognized option: %s\n\n", options->last_arg );
		break;

	case TOO_MANY_ARGS:
		printf( "-b: input file name expected as last argument\n\n" );
		break;

	case NONNUMERIC_INPUT:
		printf( "unrecognized value: %s\n\n", options->argv[options->argc-3] );
		break;

	case INVALID_INPUT:
		printf( "out of range or unrecognized value: %s\n\n", options->argv[options->argc-3] );
		break;

	case UNIT_FROM_NF:
		printf( "converting from unknown unit: %s\n\n", options->argv[options->argc-2] );
		break;

	case UNIT_TO_NF:
		printf( "converting to unknown unit: %s\n\n", options->argv[options->argc-1] );
		break;

	case INCOMPATIBLE_UNITS:
		printf( "incompatible unit types. Attempted to convert %s to %s\n\n",
				get_type_str( get_unit_by_name( options->argv[options->argc-2] )->unit_type ),
				get_type_str( get_unit_by_name( options->argv[options->argc-1] )->unit_type )
		);
		break;

	case OUTPUT_FILE_ERR:
		printf( "unable to write output file\n\n" );
		break;

	case UNITS_FILE_MISSING:
		printf( "units.dat file missing or corrupt\n\n" );
		break;

	default:
		break;
	}

	printf( "YUCON - General Purpose Unit Converter - ALPHA\n"
			"Usage:\n"
			"\tyucon\n"
			"\tyucon [options] #### original_unit converted_unit\n"
			"\tyucon -b [options] [input file]\n\n"
	);

	if ( error_code == HELP_REQUESTED )
	{
		printf( "MODES:\n"
				"\tNormal Mode      - converts from command line args or input file\n"
				"\tInteractive Mode - interactive conversion console. launched when ucon is given no arguments\n\n"
				"OPTIONS:\n"
				"\t-b          - batch conversion. convert units from input file.\n"
				"\t              last argument is expected to be input file. if no\n"
				"\t              file is specified, standard input is used\n\n"
				"\t-o[q] name  - output to file specified. q suboption cancels\n"
				"\t              console output\n\n"
				"\t-d          - descriptive. includes unit\n\n"
				"\t-v          - verbose. prints input+output values and units together\n\n"
				"\t-h, --help  - prints this help message\n\n"
				"Examples:\n"
				"\tyucon -v 1 in mm\n"
				"\tConverts 1 in to mm. Output: 1 in = 25.4 mm\n\n"
				"\tyucon -b -oq output.txt input.txt\n"
				"\tPerforms conversions in input.txt and writes results to output.txt. No console output\n\n"
				"THIS IS FREE SOFTWARE LICENSED UNDER GPLv3\n"
				"Copyright (C) 2016 - Blaine Murphy\n"
		);
	}
	else
	{
		printf( "Try \'-h\' or \'--help\' options for more details" );
	}
}

/* batch_convert
 *
 * Purpose: performs a batch conversion on a specified input file
 *   Entries in input file expected to be formatted as a standard command line
 *   conversion, one per line, like so:
 *
 *     25.4 mm in
 *     3.78 liter gal
 *     ...
 *
 *   any lines that cannot be interpreted will be ignored and will
 *   result in an "Error converting this line" in the corresponding output
 *
 * Parameters:
 *   ProgramOptions *options - pointer to options struct containing program
 *                             options
 *   UnitNode *units_list - pointer to head of units list
 *
 * Returns: nothing
 */
void batch_convert( ProgramOptions *options )
{
	FUNCTION_NOT_IMPLEMENTED("batch_convert");
}

/* args_convert
 *
 * Purpose: performs a unit conversion specified on the command line
 *
 * Parameters:
 *   ProgramOptions *options - options that the program was run with
 *   UnitNode *units_list - head of the units list
 *
 * Returns: nothing
 */
void args_convert( ProgramOptions *options )
{
	int argc = options->argc;
	char **argv = options->argv;

	double conversion = 0;
	int error_code = get_conversion( argv[argc-3], argv[argc-2], argv[argc-1], &conversion );

	if ( error_code )
	{
		help( error_code, options );
		return;
	}

	char *output_str = NULL;

	switch ( options->format )
	{
	case SIMPLE_FORMAT:
		output_str = simple_output_str( conversion );
		break;

	case DESCRIPTIVE_FORMAT:
		output_str = descriptive_output_str( conversion, argv[argc-1] );
		break;

	case VERBOSE_FORMAT:
		output_str = verbose_output_str( conversion, argv[argc-3], argv[argc-2], argv[argc-1] );
		break;

	default:
		break;
	}

	if ( options->output_mode < 2 )
	{
		printf( "%s", output_str );
	}

	if ( options->output_mode > 0 )
	{
		FILE *output_file = fopen( options->input_file, "w" );

		if ( output_file == NULL )
		{
			help( OUTPUT_FILE_ERR, options );
			return;
		}

		if ( fputs( output_str, output_file ) == EOF )
		{
			help( OUTPUT_FILE_ERR, options );
		}

		fclose( output_file );
	}

	free( output_str );
}

/* interactive_mode
 *
 * Purpose: runs an interactive terminal session for unit conversion
 *
 * Parameters:
 *   none at this time
 *
 * Returns: Int - 0 to stop. Nonzero to continue
 */
int interactive_mode()
{
	FUNCTION_NOT_IMPLEMENTED("interactive_mode");
	return 0;
}
