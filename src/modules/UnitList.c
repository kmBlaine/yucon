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

/* File: UnitsList.c
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * The UnitsList.c module maintains the internal database of units.
 * It provides methods for fetching units in a variety of ways. The
 * intent of this module is to allow the method for internal storage
 * of units to change while leaving the lest of the program unchanged
 */

#include "../H/UnitList.h"
#include <stdlib.h>
#include <stdio.h>

void delete_names_list( char **names_list )
{
	for ( int pos = 0; names_list[pos] != NULL; pos++ )
	{
		free( names_list[pos] );
	}

	free( names_list );
}

/* DELETE_UNIT
 *
 * Purpose: frees the system memory resources associated with an element of unit
 *
 * Parameters: Unit *unit - pointer to unit to delete
 *
 * Returns: nothing
 */
void delete_unit( Unit *unit )
{
	//deallocate all names as well
	delete_names_list( unit->unit_name );
	free( unit );
}

/* DELETE_UNIT_NODE
 *
 * Purpose: frees the system memory resources associated with a UnitNode.
 *   returns a pointer to the unit contained by the node
 *
 * Parameters: UnitNode *unit_node - pointer to unit node to delete
 *
 * Returns: Unit* - pointer to unit contained by node
 */
Unit *delete_unit_node( UnitNode *unit_node )
{
	Unit *unit = unit_node->unit; //copy the pointer before deletion
	free( unit_node );

	return unit;
}

/* DELETE_UNITS_LIST
 *
 * Purpose: given a pointer to the head node of a list, deletes ALL elements
 *   in the list.
 *
 * Parameters: UnitNode *head_node - pointer to head node of the list
 *
 * Returns: nothing
 */
void delete_units_list( UnitNode *head_node )
{
	while ( head_node != NULL )
	{
		UnitNode *next_node = head_node->next_unit; //copy pointer to next node
		delete_unit( delete_unit_node( head_node ) ); //delete node
		head_node = next_node;
	}
}

/* add_unit
 *
 * Purpose: adds a unit at the given index to the given list and returns
 *   an int to indicate success or failure
 *
 * Paremeters:
 *   unit - unit to add
 *   index - index in list to add at
 *   head - head of the list to add to
 *
 * Returns: Int - 1 on success. 0 on failure
 */
int add_unit( Unit* unit, int index, UnitNode* head )
{
	//check head for NULL to prevent segfault
	if ( head == NULL )
	{
		return 0;
	}

	//list uses dummy head node variant. pump the while loop to account for this
	UnitNode *prev = head;
	head = head->next_unit;

	//find position to add at
	while ( (index != 0) && head )
	{
		prev = head;
		head = head->next_unit;
		index--;
	}

	//if we walked off the end of the list, exit to prevent segfault
	if ( index != 0 )
	{
		return 0;
	}

	//add the unit otherwise
	UnitNode *unit_to_add = __NEW_UNIT_NODE;
	unit_to_add->unit = unit;
	prev->next_unit = unit_to_add;
	unit_to_add->next_unit = head;
	return 1;
}

/* remove_unit
 *
 * Purpose: removes the unit at index from the given list. returns pointer
 *   to unit removed or null pointer if unsuccessful
 *
 * Parameters:
 *   int index - index to remove unit from
 *   UnitNode *head - pointer to list to remove from
 *
 * Returns: Unit* - pointer to Unit removed. NULL if unsuccessful
 */
Unit *remove_unit( int index, UnitNode* head )
{
	//if invalid list, return to prevent segfault
	if ( head == NULL )
	{
		return NULL;
	}

	//list is dummy head node variant, pump loop to account for this
	UnitNode *prev = head;
	head = head->next_unit;

	//while not at correct position, find correct position to remove from
	while ( (index != 0) && head )
	{
		prev = head;
		head = head->next_unit;
		index--;
	}

	//if attempting to remove out of bounds unit
	if ( (index != 0) || (head == NULL) )
	{
		return NULL;
	}

	//drop unit from list
	prev->next_unit = head->next_unit;
	return delete_unit_node( head );
}

/* str_match
 *
 * Purpose: indicates if two strings match each other
 *
 * Parameters:
 *   char *str1, str2 - strings to match
 *
 * Returns: int - 1 if strings match, 0 if no match
 */
int str_match( char *str1, char *str2 )
{
	//assume strings are found. start checking at char 0
	int match = 1;
	int pos = 0;

	//while strings match and still more characters to check
	while ( (match == 1) && (str1[pos] != NULL_CHAR) && (str2[pos] != NULL_CHAR) )
	{
		//if characters at pos do not match, indicate so
		if ( str1[pos] != str2[pos] )
		{
			match = 0;
		}

		pos++;
	}

	//if one string is longer than the other but the first parts match
	//strings do not match
	if ( (str1[pos] != NULL_CHAR) || (str2[pos] != NULL_CHAR) )
	{
		match = 0;
	}

	return match;
}

/* get_unit_by_name
 *
 * Purpose: given a name and a list of units, gets the Unit that has
 *   a matching name
 *
 * Parameters:
 *   char *name - name string
 *   UnitNode *head - pointer to list to search for units
 *
 * Returns: Unit - Unit with matching name if found. Null pointer otherwise.
 */
Unit *get_unit_by_name( char *name, UnitNode *head )
{
	if ( head == NULL )
	{
		return NULL;
	}

	head = head->next_unit;
	Unit *unit = NULL;
	int found = 0;

	while ( head && (found == 0) )
	{
		unit = head->unit;

		for ( int pos = 0; (found == 0) && (unit->unit_name[pos] != NULL); pos++ )
		{
			found = str_match( name, unit->unit_name[pos] );
		}

		head = head->next_unit;
	}

	if ( found )
	{
		return unit;
	}
	else
	{
		return NULL;
	}
}

/* print_units_list
 *
 * Purpose: prints the units database to the console for debugging purposes
 *
 * Parameters:
 *   UnitNode *head - pointer to head of the units list
 *
 * Returns: nothing
 */
void print_units_list( UnitNode *head )
{
	head = head->next_unit;

	while ( head )
	{
		Unit *current_unit = head->unit;

		for ( int pos = 0; current_unit->unit_name[pos]; pos++ )
		{
			printf( "%s,", current_unit->unit_name[pos] );
		}

		printf( "type: %d,factor: %f\n", current_unit->unit_type, current_unit->conversion_factor );

		head = head->next_unit;
	}
}



