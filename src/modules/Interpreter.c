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
 * Interpreter.c
 *
 *  Created on: Nov 19, 2016
 *      Author: kbm1271
 */

#include "H/Interpreter.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "H/GlobalDefines.h"

#define ONE_TIME_MODE      0
#define BATCH_MODE         1
#define STDOUT_MODE        0
#define VERBOSE_MODE       1
#define QUIET_MODE         2
#define SIMPLE_FORMAT      0
#define DESCRIPTIVE_FORMAT 1
#define VERBOSE_FORMAT     2

/* struct ProgramOptions
 *
 * Purpose: holds the options specified by the args on the command line
 *   for easy passage to through methods
 *
 * Fields:
 *   char input_mode - specifies where to take input from
 *     0 - one time conversion. convert from command line args
 *     1 - batch mode. convert from input file (list of conversions)
 *
 *   char *batch_file - specifies the name of the input file to read from when in batch mode
 *     NULL - read from stdin
 *     ptr  - read from file with specified name
 *
 *   char output_mode - specifies where to write output
 *     0 - stdout mode. write output only to stdout
 *     1 - verbose mode. write output to stdout and to output file specified
 *     2 - quiet mode. write output only to output file
 *
 *   char *output_file - specifies the name of the output file to write to when in verbose or quiet mode
 *     NULL - N/A
 *     ptr  - write to file with specified name
 *
 *   char format - specifies the format of outputs
 *     0 - simple. writes number only eg. 1.1, 5, 25.4
 *     1 - descriptive. writes number and associated unit eg. 1.1 qt, 5 cm2, 25.4 mm
 *     2 - verbose. writes input and output values with units eg. 1 in = 25.4 mm, 1 in3 = 16.38 cc
 *
 *   char *last_arg - argument where options parsing left off
 *
 *   int argc - raw args count
 *
 *   char *argv[] - raw args array
 */
typedef struct ProgramOptions ProgramOptions;
struct ProgramOptions
{
	char input_mode;
	char *input_file;
	char output_mode;
	char *output_file;
	char format;
	char *last_arg;
	int argc;
	char **argv;
};

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

void help( int error_code, ProgramOptions *options, UnitNode* units_list )
{
	int argc = options->argc;
	char **argv = options->argv;

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
		printf( "unrecognized value: %s\n\n", argv[argc-3] );
		break;

	case INVALID_INPUT:
		printf( "out of range or unrecognized value: %s\n\n", argv[argc-3] );
		break;

	case UNIT_FROM_NF:
		printf( "converting from unknown unit: %s\n\n", argv[argc-2] );
		break;

	case UNIT_TO_NF:
		printf( "converting to unknown unit: %s\n\n", argv[argc-1] );
		break;

	case INCOMPATIBLE_UNITS:
		printf( "incompatible unit types. Attempted to convert %s to %s\n\n",
				get_type_str( get_unit_by_name( argv[argc-2], units_list )->unit_type ),
				get_type_str( get_unit_by_name( argv[argc-1], units_list )->unit_type )
		);
		break;

	case OUTPUT_FILE_ERR:
		printf( "unable to write output file\n\n" );
		break;

	default:
		break;
	}

	printf( "UCON - General Purpose Unit Converter - ALPHA\n"
			"Usage:\n"
			"\tucon\n"
			"\tucon [options] #### original_unit converted_unit\n"
			"\tucon -b [options] [input file]\n\n"
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
				"\tucon -v 1 in mm\n"
				"\tConverts 1 in to mm. Output: 1 in = 25.4 mm\n\n"
				"\tucon -b -oq output.txt input.txt\n"
				"\tPerforms conversions in input.txt and writes results to output.txt. No console output\n\n"
				"THIS PROGRAM IS FREE SOFTWARE LICENSED UNDER GPLv3\n"
				"Copyright (C) 2016 - Blaine Murphy\n"
		);
	}
	else
	{
		printf( "Try \'-h\' or \'--help\' options for more details" );
	}
}

void batch_convert( ProgramOptions *options, UnitNode *units_list )
{

}

void args_convert( ProgramOptions *options, UnitNode *units_list )
{
	int argc = options->argc;
	char **argv = options->argv;

	double conversion = get_conversion( argv[argc-3], argv[argc-2], argv[argc-1], units_list );

	if ( conversion < 0 )
	{
		help( (int)(conversion), options, units_list );
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
			help( OUTPUT_FILE_ERR, options, units_list );
			return;
		}

		if ( fputs( output_str, output_file ) == EOF )
		{
			help( OUTPUT_FILE_ERR, options, units_list );
		}

		fclose( output_file );
	}

	free( output_str );
}

int interactive_mode()
{
	return 0;
}

void convert( int argc, char **argv )
{
	UnitNode *units_list = load_units_list();

	ProgramOptions options;
	int error_code = set_program_options( &options, argc, argv );

	if ( error_code )
	{
		help( error_code, &options, units_list );
		return;
	}

	if ( options.input_mode )
	{
		batch_convert( &options, units_list );
	}
	else
	{
		args_convert( &options, units_list );
	}
}
