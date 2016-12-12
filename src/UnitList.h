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

/* File: UnitList.h
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * Public declarations for the UnitList.c module. Serves as a front end
 * to the database of units so that the method of unit storage may be
 * changed in the future if deemed necessary so that minimal code modification
 * is necessary.
 */

#ifndef H_UNITLIST_H_
#define H_UNITLIST_H_

#include "GlobalDefines.h"

#define INPUT_UNIT  0
#define OUTPUT_UNIT 1

typedef struct Unit Unit;
struct Unit
{
	char **unit_name;
	int unit_type;
	double conversion_factor;
	double offset;
};

#define __NEW_UNIT  calloc(1,sizeof(Unit))

void delete_names_list( char** );
void delete_unit( Unit* );
void delete_units_list();

int  add_unit( Unit*, int );
Unit *remove_unit( int );
void print_units_list();

Unit *get_unit_by_name( char*, int );

#endif /* H_UNITLIST_H_ */

