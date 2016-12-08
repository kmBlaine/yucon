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
/* File: Interpreter.h
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * Public declarations for the Interpreter.c module.
 */

#ifndef H_INTERPRETER_H_
#define H_INTERPRETER_H_

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

int  set_program_options( ProgramOptions*, int, char** );
void interactive_mode( ProgramOptions* );
void batch_convert( ProgramOptions* );
void args_convert( ProgramOptions* );
void help( int, ProgramOptions* );

#endif /* H_INTERPRETER_H_ */
