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

#include "GlobalDefines.h"
#include "Interpreter.h"
#include "UnitList.h"
#include "Convert.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#define STDOUT_MODE        0
#define VERBOSE_MODE       1
#define QUIET_MODE         2
#define SIMPLE_FORMAT      0
#define DESCRIPTIVE_FORMAT 1
#define VERBOSE_FORMAT     2

#define MAX_BUFFER_SIZE 128
#define MAX_TOKENS      4

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

/* print_version
 *
 * Purpose: prints the program version and license info from the console
 *   implmented to prevent needless text duplication between the help
 *   methods
 *
 * Parameters: none
 *
 * Returns: nothing
 */
void print_version()
{
	printf( PROGRAM_TITLE
			"    "COPYRIGHT_NOTICE
			"    Released: "RELEASE_DATE"\n"
			"    Source code available at <https://github.com/kmBlaine/yucon>\n"
			"    See changelog in the \'README\' for version-specific details\n\n"
			"LICENSE NOTICE:\n"
			"This program is free software: you can redistribute it and/or modify\n"
			"it under the terms of the GNU General Public License as published by\n"
			"the Free Software Foundation, either version 3 of the License, or\n"
			"(at your option) any later version.\n\n"
			"This program is distributed in the hope that it will be useful,\n"
			"but WITHOUT ANY WARRANTY; without even the implied warranty of\n"
			"MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the\n"
			"GNU General Public License for more details.\n\n"
			"You should have received a copy of the GNU General Public License\n"
			"along with this program.  If not, see <http://www.gnu.org/licenses/>.\n"
	);
}

int is_double( char *str )
{
	char *input_end = NULL;
	strtod( str, &input_end );

	//if the conversion was unsuccessful, input_end will be not be '\0' and 'recall last' will not be present
	if ( (strcmp(str, ":") != 0) && input_end && (input_end[0] != NULL_CHAR) )
	{
		return 0;
	}

	return 1;
}

/* check_nondash_arg
 *
 * Purpose: checks if nonspecial arguments appear in an expected way and
 *   returns an appropriate action code. this avoids duplicated code for properly
 *   interpreting negative versus non-negative conversions
 *
 * Parameters:
 *   ProgramOptions *options - pointer to program runtime options struct
 *   int arg - arg to start checking at
 *
 * Returns: 0 or TRY_ARGS_CONVER - no error. Nonzero int for error.
 */
int check_nondash_arg( ProgramOptions *options, int arg )
{
	if ( options->input_mode == BATCH_MODE )
	{
		//if exactly one argument left, interpret as input file name
		if ( arg == (options->argc - 1) )
		{
			options->input_file = options->argv[arg];
			return EXIT_SUCCESS;
		}
		//loop will not reach this point. args must be > 1
		//if we reached this point, too many args
		else
		{
			options->last_arg = options->argv[arg + 1];
			return TOO_MANY_ARGS;
		}
	}
	else
	{
		if ( !is_double( options->argv[arg] ) )
		{
			return NONNUMERIC_INPUT;
		}

		if ( (options->argc - arg) == 3 )
		{
			return TRY_ARGS_CONVERT;
		}
		else if ( (options->argc - arg) < 3 )
		{
			return NOT_ENOUGH_ARGS;
		}
		else
		{
			options->last_arg = options->argv[arg + 3]; //set last_arg to the unexpected trailing argument
			return TOO_MANY_ARGS; //there is a trailing argument or arguments
		}
	}
	/*
	//if in one time mode, and three args left, try to convert
	else if ( (options->argc - arg) == 3 )
	{
		return TRY_ARGS_CONVERT;
	}
	//if there are more than three args, not possible. too many args
	else if ( (options->argc - arg) > 3 )
	{
		return UNRECOGNIZED_ARG;
	}
	else
	{
		return NOT_ENOUGH_ARGS;
	}
	*/
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
	options->input_mode = INTERACTIVE_MODE;
	options->input_file = NULL;
	options->output_mode = STDOUT_MODE;
	options->output_file = NULL;
	options->format = SIMPLE_FORMAT;

	//if any option is -h or --help, return HELP error code
	for ( int arg = 1; arg < argc; arg++ )
	{
		if ( (strcmp( argv[arg], "-h" ) == 0) || (strcmp( argv[arg], "--help") == 0) )
		{
			return HELP_REQUESTED;
		}
		if ( strcmp( argv[arg], "--version" ) == 0 )
		{
			return VERSION_REQUESTED;
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
				if ( arg < (argc - 1) ){ options->output_file = argv[++arg]; }
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
			//arg may simply be a negative value. check it as a non-special arg
			else if ( atof(argv[arg]) )
			{
				int error_code = check_nondash_arg( options, arg );

				if ( error_code == TRY_ARGS_CONVERT )
				{
					options->input_mode = ONE_TIME_MODE;
					break;
				}
				else if ( error_code )
				{
					return error_code;
				}
			}
			else
			{
				return UNRECOGNIZED_ARG; //else unrecognized arg. error
			}
		}
		else //else if non dash argument
		{
			int error_code = check_nondash_arg( options, arg );

			if ( error_code == TRY_ARGS_CONVERT )
			{
				options->input_mode = ONE_TIME_MODE;
				break;
			}
			else if ( error_code )
			{
				return error_code;
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
 *
 * Returns: nothing
 */
void help( int error_code, ProgramOptions *options )
{
	if ( error_code == VERSION_REQUESTED )
	{
		print_version();
		return;
	}

	if ( error_code != HELP_REQUESTED )
	{
		printf( "Error: ");
	}

	switch ( error_code )
	{
	case NOT_ENOUGH_ARGS:
		if ( options->output_mode != STDOUT_MODE )
		{
			printf( "%s: expected output file name\n\n", options->last_arg );
		}
		else
		{
			printf( "incomplete conversion. Expected an input and output unit\n\n" );
		}
		break;

	case UNRECOGNIZED_ARG:
		printf( "unrecognized option: %s\n\n", options->last_arg );
		break;

	case TOO_MANY_ARGS:
		if ( options->input_mode == BATCH_MODE )
		{
			printf( "-b: expects input file as last argument: " );
		}
		printf( "found unexpected trailing argument: %s\n\n", options->last_arg );

		break;

	case NONNUMERIC_INPUT:
		printf( "expected number. Found: %s\n\n", options->last_arg );
		break;

	case INVALID_INPUT:
		printf( "out of range value: %s\n\n", options->argv[options->argc-3] );
		break;

	case UNIT_FROM_NF:
		printf( "converting from unknown unit: %s\n\n", options->argv[options->argc-2] );
		break;

	case UNIT_TO_NF:
		printf( "converting to unknown unit: %s\n\n", options->argv[options->argc-1] );
		break;

	case INCOMPATIBLE_UNITS:
		printf( "incompatible unit types. Attempted to convert %s to %s\n\n",
				get_type_str( get_unit_by_name( options->argv[options->argc-2], INPUT_UNIT )->unit_type ),
				get_type_str( get_unit_by_name( options->argv[options->argc-1], OUTPUT_UNIT )->unit_type )
		);
		break;

	case OUTPUT_FILE_ERR:
		printf( "unable to write output file\n\n" );
		break;

	case UNITS_FILE_MISSING:
		printf( "units.dat file missing or corrupt\n\n" );
		break;

	case INPUT_FILE_ERR:
		printf( "unable to open input file \'%s\': File not found\n\n", options->input_file );
		break;

	case FILE_OUTPUT_NOT_ALLOWED:
		printf( "file output not allowed in interactive mode\n\n");
		break;

	case UNKNOWN_PREFIX:
		printf( "unknown metric prefix\n\n" );
		break;

	case NO_NAME_GIVEN:
		printf( "no name given after metric prefix\n\n" );
		break;

	case NO_NAME_ALLOWED:
		printf( "no name allowed after \':\' (recall last) function\n\n" );
		break;

	case INPUT_UNIT_UNSET:
		printf( "unable to recall last input. Not set\n\n" );
		break;

	case OUTPUT_UNIT_UNSET:
		printf( "unable to recall last output. Not set\n\n" );
		break;

	default:
		break;
	}

	printf( PROGRAM_TITLE
			"Usage:\n"
			"    yucon [options]\n"
			"    yucon [options] #### <input_unit> <output_unit>\n"
			"    yucon -b [options] [input file]\n\n"
	);

	if ( error_code == HELP_REQUESTED )
	{
		printf( "    In first form, run an interactive session for converting units\n"
				"    In second form, perform the conversion specified on the command line\n"
				"    In third form, perform a batch conversion from file or from pipe if no file\n"
				"      is specified\n\n"
				"Options:\n"
				"    -b         - batch conversion. convert units from input file. last\n"
				"                 argument is expected to be input file. if no file is specified,\n"
				"                 STDIN is used\n\n"
				"    -o[q] name - output to file specified. q suboption cancels console output\n\n"
				"    -d         - descriptive output (includes output unit)\n\n"
				"    -v         - verbose output. (include original value, input&output units)\n\n"
				"    -h, --help - prints this help message\n\n"
				"    --version  - print version and license info\n\n"
				"Examples:\n"
				"    $ yucon -v 1 in mm\n"
				"      Outputs: 1 in = 25.4 mm\n\n"
				"    $ yucon -b -oq output.txt input.txt\n"
				"      Performs conversions in input.txt and writes results to output.txt. No\n"
				"      console output\n\n"
				"This is free software licensed under the GNU General Public License v3.\n"
				"Use \'--version\' option for more details.\n"
				COPYRIGHT_NOTICE
		);
	}
	else
	{
		printf( "Try \'-h\' or \'--help\' options for more details\n" );
	}
}

/* help_interactive
 *
 * Purpose: prints help messages specific to the interactive mode
 *   implemented to avoid numerous checks within the previous help
 *   method and because the parameters are not the same
 *
 * Parameters:
 *   int error_code - internal error code to retrieve help on
 *   ProgramOptions *options - pointer to program runtime options struct
 *   char **token - array of user input tokens
 *
 * Returns: nothing
 */
void help_interactive( int error_code, ProgramOptions *options, char **token )
{
	if ( error_code == VERSION_REQUESTED )
	{
		print_version();
		return;
	}

	if ( error_code != HELP_REQUESTED )
	{
		printf( "Error: ");
	}

	switch ( error_code )
	{
	case NOT_ENOUGH_ARGS:
		printf( "Not enough arguments\n\n" );
		break;

	case UNRECOGNIZED_ARG:
		printf( "unrecognized argument\n\n" );
		break;

	case TOO_MANY_ARGS:
		printf( "too many arguments\n\n" );
		break;

	case NONNUMERIC_INPUT:
		printf( "unrecognized command\n\n" );
		break;

	case INVALID_INPUT:
		printf( "value out of range\n\n" );
		break;

	case UNIT_FROM_NF:
		printf( "converting from unknown unit: %s\n\n", token[1] );
		break;

	case UNIT_TO_NF:
		printf( "converting to unknown unit: %s\n\n", token[2] );
		break;

	case INCOMPATIBLE_UNITS:
		printf( "incompatible unit types. Attempted to convert %s to %s\n\n",
				get_type_str( get_unit_by_name( token[1], INPUT_UNIT )->unit_type ),
				get_type_str( get_unit_by_name( token[2], OUTPUT_UNIT )->unit_type )
		);
		break;

	case UNKNOWN_PREFIX:
		printf( "unknown metric prefix\n\n" );
		break;

	case NO_NAME_GIVEN:
		printf( "no name given after metric prefix\n\n" );
		break;

	case NO_NAME_ALLOWED:
		printf( "no name allowed after \':\' (recall last) function\n\n" );
		break;

	case INPUT_UNIT_UNSET:
		printf( "unable to recall last input. Not set\n\n" );
		break;

	case OUTPUT_UNIT_UNSET:
		printf( "unable to recall last output. Not set\n\n" );
		break;

	default:
		if ( error_code != HELP_REQUESTED ){ printf( "unknown error.\n\n" ); }
		break;
	}

	if ( error_code == HELP_REQUESTED )
	{
		printf( "Enter a conversion or command. Conversions expected in format:\n"
				"    #### <input_unit> <output_unit>\n\n"
				"Commands:\n"
				"    exit    - exit the program\n"
				"    help    - print this help message\n"
				"    version - print version and license info\n\n"
				"This is free software licensed under the GNU General Public License v3.\n"
				"Type \'version\' for more details.\n"
				COPYRIGHT_NOTICE
		);
	}
	else
	{
		printf( "Type \'help' for assistance.\n" );
	}
}

/* generate_output
 *
 * Purpose: this function handles output generation for each of the routines
 *   to avoid code duplication
 *
 * Parameters:
 *   ProgramOptions *options - pointer to program runtime options struct
 *   FILE *output - output file if any
 *   char **token - array of tokens to be passed when using batch or interactive mode
 *                  tokens must appear in this order:
 *                    0 - number in valid double format
 *                    1 - original unit
 *                    2 - converted unit
 *
 * Returns: nothing
 */
void generate_output( ProgramOptions *options, FILE *output, char **token )
{
	int argc = options->argc;
	char **argv = options->argv;

	char *token0;
	char *token1;
	char *token2;

	if ( options->input_mode != ONE_TIME_MODE )
	{
		token0 = token[0];
		token1 = token[1];
		token2 = token[2];
	}
	else
	{
		token0 = argv[argc-3];
		token1 = argv[argc-2];
		token2 = argv[argc-1];
	}

	double conversion = 0;
	int error_code = get_conversion( token0, token1, token2, &conversion );

	if ( error_code )
	{
		if ( options->input_mode == ONE_TIME_MODE )
		{
			help( error_code, options );
		}
		else if ( options->input_mode == INTERACTIVE_MODE )
		{
			help_interactive( error_code, options, token );
		}
		return;
	}

	char *output_str = NULL;

	switch ( options->format )
	{
	case SIMPLE_FORMAT:
		output_str = simple_output_str( conversion );
		break;

	case DESCRIPTIVE_FORMAT:
		output_str = descriptive_output_str( conversion, token2 );
		break;

	case VERBOSE_FORMAT:
		output_str = verbose_output_str( conversion, token0, token1, token2 );
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
		if ( fputs( output_str, output ) == EOF )
		{
			if ( options->input_mode != BATCH_MODE )
			{
				help( OUTPUT_FILE_ERR, options );
			}
		}
	}

	free( output_str );
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
	FILE *input = NULL;
	FILE *output = NULL;

	if ( options->input_file != NULL )
	{
		input = fopen( options->input_file, "r" );

		if ( input == NULL )
		{
			help( INPUT_FILE_ERR, options );
			return;
		}
	}
	else
	{
		input = stdin;
	}

	if ( options->output_mode > 0 )
	{
		output = fopen( options->output_file, "w" );

		if ( output == NULL )
		{
			help( OUTPUT_FILE_ERR, options );
			fclose( input );
			return;
		}
	}

	while ( feof( input ) == 0 )
	{
		char line_buffer[MAX_BUFFER_SIZE];

		fgets( line_buffer, MAX_BUFFER_SIZE, input );

		//replace newline character before proceeding
		for ( int pos = 0; line_buffer[pos] != NULL_CHAR; pos++ )
		{
			if ( line_buffer[pos] == '\n' )
			{
				line_buffer[pos] = NULL_CHAR;
				break;
			}
		}

		char *token[3];
		token[0] = NULL;
		token[1] = NULL;
		token[2] = NULL;

		token[0] = strtok( line_buffer, " " );
		token[1] = strtok( NULL, " " );
		token[2] = strtok( NULL, " " );

		if ( (token[0] != NULL && !is_double(token[0])) || (token[1] == NULL) || (token[2] == NULL) )
		{
			continue;
		}

		generate_output( options, output, token );
	}

	if ( input != stdin ) fclose( input );
	if ( output ) fclose( output );

	delete_recall_data();
}

/* args_convert
 *
 * Purpose: performs a unit conversion specified on the command line
 *
 * Parameters:
 *   ProgramOptions *options - options that the program was run with
 *
 * Returns: nothing
 */
void args_convert( ProgramOptions *options )
{
	FILE *output;

	if ( options->output_mode != STDOUT_MODE )
	{
		output = fopen( options->input_file, "w" );

		if ( output == NULL )
		{
			help( OUTPUT_FILE_ERR, options );
			return;
		}
	}

	generate_output( options, output, NULL );

	if ( options->output_mode != STDOUT_MODE )
	{
		fclose( output );
	}

	delete_recall_data();
}

/* run_command
 *
 * Purpose: given a string representing some command for interactive mode
 *   decompose the command and execute accordingly. return error code if
 *   command cannot be executed or to signal the program in some way
 *
 * Parameters:
 *   char *str - line of user input
 *   ProgramOptions *options - pointer to program runtime options struct
 *
 * Returns: 0 - success. Nonzero - failure or signal
 */
int run_command( char *str, ProgramOptions *options )
{
	//if empty line, do nothing
	if ( str[0] == '\n' )
	{
		return EXIT_SUCCESS;
	}

	//replace newline character before proceeding
	for ( int pos = 0; str[pos] != NULL_CHAR; pos++ )
	{
		if ( str[pos] == '\n' )
		{
			str[pos] = NULL_CHAR;
			break;
		}
	}

	//tokenize the input line. first initialize the pointer array
	char *token[MAX_TOKENS];
	for ( int pos = 0; pos < MAX_TOKENS; pos++ )
	{
		token[pos] = NULL;
	}

	//get initial token
	token[0] = strtok( str, " " );

	//get remaining tokens if any. exit loop if more tokens appeared
	for ( int pos = 1; token[pos-1]; pos++ )
	{
		//CHANGE THIS TO ERROR RETURN?? Blaine M. - 3 DEC 2016
		if ( pos >= MAX_TOKENS )
		{
			break;
		}

		token[pos] = strtok( NULL, " " );
	}

	if ( strcmp( token[0], "exit" ) == 0 )
	{
		return EXIT_PROGRAM;
	}
	else if ( strcmp( token[0], "help" ) == 0 )
	{
		return HELP_REQUESTED;
	}
	else if ( strcmp( token[0], "version" ) == 0 )
	{
		return VERSION_REQUESTED;
	}
	else
	{
		if ( !is_double( token[0] ) )
		{
			return NONNUMERIC_INPUT;
		}

		if ( (token[1] == NULL) || (token[2] == NULL) )
		{
			return NOT_ENOUGH_ARGS;
		}
	}

	generate_output( options, NULL, token );

	return 0;
}

/* interactive_mode
 *
 * Purpose: runs an interactive terminal session for unit conversion
 *
 * Parameters:
 *   ProgramOptions *options - pointer to program runtime options struct
 *
 * Returns: nothing
 */
void interactive_mode( ProgramOptions *options )
{
	if (options->output_mode != STDOUT_MODE )
	{
		//help function distinguishes between input modes
		//set to non-interactive mode to print command line help messages instead of interactive messages
		//options->input_mode = ONE_TIME_MODE;
		help( FILE_OUTPUT_NOT_ALLOWED, options );
		return;
	}

	printf( PROGRAM_TITLE
			"Type \'help\' for assistance.\n"
	);

	//user may have accidentally piped input in which case reading from stdin will cause errors
	while ( feof(stdin) == 0 )
	{
		printf( "\n> " );

		char line_buffer[MAX_BUFFER_SIZE];

		fgets( line_buffer, MAX_BUFFER_SIZE, stdin );

		int error_code = run_command( line_buffer, options );

		if ( error_code == EXIT_PROGRAM )
		{
			break;
		}
		else if ( error_code )
		{
			help_interactive( error_code, options, NULL );
		}
	}

	delete_recall_data();
}
