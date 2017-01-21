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
/* File: yucon.c
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * main method for the program. loads configurations, units, and
 * then delegates out functionality to the program's other modules
 */


#include <stdlib.h>
#include <stdio.h>

#include "Interpreter.h"
#include "LoadConfig.h"

int error_code;
char *error_msg;

int main( int argc, char *argv[] )
{
	error_code = load_units_list();

	if ( error_code )
	{
		help( NULL, NULL ); //safe to set null for a units.dat error
		return EXIT_SUCCESS;
	}

	ProgramOptions options;
	error_code = set_program_options( &options, argc, argv );

	if ( error_code )
	{
		options.input_mode = ONE_TIME_MODE; //change input mode to force console help messages
		help( &options, NULL );
	}
	else
	{
		switch ( options.input_mode )
		{
		case ONE_TIME_MODE:
			args_convert( &options );
			break;

		case BATCH_MODE:
			batch_convert( &options );
			break;

		case INTERACTIVE_MODE:
			interactive_mode( &options );
			break;

		default:
			break;
		}
	}

	delete_units_list();

	return EXIT_SUCCESS;
}
