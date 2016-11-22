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
 * UnitList.h
 *
 * Contains the abstract declarations for the units and the linked list of units for the program
 * Links the UnitList.c file
 */

#ifndef H_UNITLIST_H_
#define H_UNITLIST_H_

typedef struct Unit Unit;
struct Unit
{
	char **unit_name;
	int unit_type;
	double conversion_factor;
};

typedef struct UnitNode UnitNode;
struct UnitNode
{
	Unit *unit;
	UnitNode *next_unit;
};

#define __NEW_UNIT        calloc(1,sizeof(Unit))
#define __NEW_UNIT_NODE   calloc(1,sizeof(UnitNode))

void delete_names_list( char** );
void delete_unit( Unit* );
Unit *delete_unit_node( UnitNode* );
void delete_units_list( UnitNode* );

int add_unit( Unit*, int, UnitNode* );
Unit *remove_unit( int, UnitNode* );
void print_units_list( UnitNode* );

Unit *get_unit_by_name( char*, UnitNode* );

#endif /* H_UNITLIST_H_ */

