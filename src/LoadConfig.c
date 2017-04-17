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

/* File: LoadConfig.c
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * This module handles the loading of configurations from files. This
 * includes program configrations as well as building the units list
 * from the data files.
 */

#include "LoadConfig.h"
#include "UnitList.h"

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

const char *length = "length";
const char *volume = "volume";
const char *area = "area";
const char *energy = "energy";
const char *power = "power";
const char *mass = "mass";
const char *force = "force";
const char *torque = "torque";
const char *speed = "speed";
const char *pressure = "pressure";
const char *temp = "temperature";
const char *feconomy = "fuel economy";

/* names_count
 *
 * Purpose: counts up the number of names in the names line of units.cfg
 *
 * Parameters:
 *   char *line - line from the input file to examine
 *
 * Returns: int - number of names (commas + 1)
 */
int names_count( char *line )
{
	int count = 1; //assume at least one name

	for ( int pos = 0; line[pos] != NULL_CHAR; pos++ )
	{
		if ( line[pos] == ',' )
		{
			count++;
		}
	}

	return count;
}

/* str_safe_copy
 *
 * Purpose: safely copies a string. the given string will be copied to
 *   a non-volatile block of memory
 *
 * Parameters:
 *   char *str - string to copy
 *
 * Returns: char* - pointer to the copy of the string
 */
char *str_safe_copy( char *str )
{
	int str_size = 1; //loop does not count null terminator.

	//find string size and replace newline character
	for ( int pos = 0; (str[pos] != NULL_CHAR) && (str[pos] != '\n'); pos++ )
	{
		str_size++;
	}

	//allocate block of memory to copy to
	char *str_copy = calloc( str_size, sizeof(char) );

	//copy string
	for ( int pos = 0; (str[pos] != NULL_CHAR) && (str[pos] != '\n'); pos++ )
	{
		str_copy[pos] = str[pos];
	}

	return str_copy;
}

/* get_names_list
 *
 * Purpose: given the list of names in CSV format, tokenize and return array of strings
 *
 * Parameters:
 *   char *str - string of names in CSV format
 *
 * Returns: char** - array of names (strings)
 */
char **get_names_list( char *str )
{
	//create new names list
	char **names_list = calloc( names_count(str) + 1, sizeof(char*) );

	//get first name
	char *name = strtok( str, "," );
	int pos = 0;

	//while there are names left to read, copy the names to the list
	while ( name )
	{
		names_list[pos] = str_safe_copy( name );
		name = strtok( NULL, "," );
		pos++;
	}

	names_list[pos] = NULL; //terminate the pointer array

	return names_list;
}

/* get_unit_type
 *
 * Purpose: converts the common unit type (ie force, mass, length) to the
 *   program's internal numeric representation
 *
 * Parameters:
 *   char *str - common unit type name (ie force, mass, length)
 *
 * Returns: int - internal numeric representation if match is found. -1 if not
 */
int get_unit_type( char *str )
{
	if ( strncmp( str, length, 6) == 0 )
	{
		return LENGTH;
	}

	if ( strncmp( str, volume, 6) == 0 )
	{
		return VOLUME;
	}

	if ( strncmp( str, area, 4) == 0 )
	{
		return AREA;
	}

	if ( strncmp( str, energy, 6) == 0 )
	{
		return ENERGY;
	}

	if ( strncmp( str, power, 5) == 0 )
	{
		return POWER;
	}

	if ( strncmp( str, mass, 4) == 0 )
	{
		return MASS;
	}

	if ( strncmp( str, force, 5) == 0 )
	{
		return FORCE;
	}

	if ( strncmp( str, torque, 6) == 0 )
	{
		return TORQUE;
	}

	if ( strncmp( str, speed, 5 ) == 0 )
	{
		return SPEED;
	}

	if ( strncmp( str, pressure, 8 ) == 0 )
	{
		return PRESSURE;
	}

	if ( strncmp( str, temp, 11) == 0 )
	{
		return TEMP;
	}
	if ( strncmp( str, feconomy, 12 ) == 0 )
	{
		return FECONOMY;
	}

	return -1; //return -1 on failure
}

/* load_units_list
 *
 * Purpose: loads the units list from the units.cfg file and
 *   returns pointer to the top of the linked list of units
 *
 * Parameters: none
 *
 * Returns: UnitNode* - pointer to head of list
 */
int load_units_list()
{
	int end_of_list = 0;

	FILE *units_cfg = NULL;

//change path based on the platform this will be built and run on
//Linux will expect config file in /etc/yucon/
#if defined(__unix__) || defined(__linux__) || defined(__gnu_linux__)
	units_cfg = fopen( "/etc/yucon/units.dat", "r" );

#pragma message("Using UNIX file path for units.dat file. Yucon expects it at /etc/yucon/units.dat")
//if other system, default to loading from the current file path
#else
	units_cfg = fopen( "units.dat", "r" );

#pragma message("Using application launch path for units.dat file. Yucon expects it in same folder as executable.")
#endif

	//exit early if config file does not exist
	if ( units_cfg == NULL )
	{
		return UNITS_FILE_MISSING;
	}

	/* CFG file formatted in following manner.
	 *
	 *   ...
	 *
	 *   names=[list of names in CSV format]
	 *   type=[type]
	 *   factor=[floating point value]
	 *   offset=[floating point value]
	 *
	 *   ...
	 *
	 * names, type, factor and offset must appear as a cluster in that order,
	 * else the unit and properties enumerated on these lines will be discarded.
	 * all lines that do not begin with 'names=', 'type=', 'factor=', or 'offset=' will
	 * be interpreted as a comment.
	 * for instance if a unit was formatted in the file as:
	 *   names=inch,in
	 *   factor=25.4
	 *   type=length
	 *   offset=0
	 *
	 * all lines would be ignored and discarded. similarly white space or comments
	 * breaking a valid sequence will cause the unit to be discarded. Ex:
	 *   names=[names]
	 *   this is a comment
	 *   type=[type]
	 *   factor=[factor]
	 *   offset=[offset]
	 *
	 * this unit would be discarded.
	 */

	//while more units in the file
	while ( feof( units_cfg ) == 0 )
	{
		//temp storage for unit properties
		char **names_list = NULL;
		int unit_type = -1;
		double conversion_factor = 0;
		double offset = 0;

		char line_buffer[MAX_LINE_LENGTH]; //buffer to read file into
		fgets( line_buffer, MAX_LINE_LENGTH, units_cfg );

		//if names line, read names. else go to top of loop
		if ( strncmp( line_buffer, "names=", 6 ) == 0 )
		{
			names_list = get_names_list( line_buffer + 6 );
		}
		else
		{
			continue;
		}

		fgets( line_buffer, MAX_LINE_LENGTH, units_cfg );

		//if type line, get type. else delete names list to prevent
		//memory leak and go to top of loop
		if ( strncmp( line_buffer, "type=", 5 ) == 0 )
		{
			unit_type = get_unit_type( line_buffer + 5 );
		}
		else
		{
			delete_names_list( names_list );
			continue;
		}

		fgets( line_buffer, MAX_LINE_LENGTH, units_cfg );

		//if conversion factor line, get factor. else delete names
		//to prevent memory leak and go to top of loop
		if ( strncmp( line_buffer, "factor=", 7 ) == 0 )
		{
			conversion_factor = atof( line_buffer + 7 );
		}
		else
		{
			delete_names_list( names_list );
			continue;
		}

		fgets( line_buffer, MAX_LINE_LENGTH, units_cfg );

		//if offset line, get offset. else delete names to prevent memory leak
		if ( strncmp( line_buffer, "offset=", 7) == 0 )
		{
			offset = atof( line_buffer + 7 );
		}
		else
		{
			delete_names_list( names_list );
			continue;
		}

		//create new unit and add it to list
		Unit *next_unit = __NEW_UNIT;
		next_unit->unit_name = names_list;
		next_unit->unit_type = unit_type;
		next_unit->conversion_factor = conversion_factor;
		next_unit->offset = offset;

		add_unit( next_unit, end_of_list++ );
	}

	fclose( units_cfg );

	return EXIT_SUCCESS;
}
