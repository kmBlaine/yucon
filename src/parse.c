/* File: parse.c
 *   Author: Blaine Murphy
 *   Created: 2017-04-23
 *
 * DESCRIPTON:
 * Contains generalized helper functions for parsing such as binary search
 * of an array of strings, replacing newline characters, etc
 */

#include <stdlib.h>
#include <string.h>


/* search
 *
 * Purpose: Performs binary search on an array of strings.
 *
 * Parameters:
 *   const char *str - string to search for
 *   const char *array[] - array of strings to search for str in
 *   int start - index to start searching at. This should generally be 0; function is recursive
 *   int end - index of end + 1. This should generally be number of elements in the array; function is recursive
 *   int *index - if pointer to an integer is given, the index of a matching string will be written to the integer
 *                if you do not wish to do this you may simply enter NULL
 *
 * Returns: Pointer to matching element in the array. NULL if no matching element is found
 *          Writes index of matching item to *index if given.
 */
const char *search( const char *str, const char *array[], int start, int end, int *index )
{
	if ( start == end )
	{
		return NULL;
	}

	int mid = (start + end) / 2;

	int cmp = strcmp( str, array[mid] );

	if ( cmp == 0 )
	{
		if ( index ){ *index = mid; }
		return array[mid];
	}
	else if ( cmp > 0 )
	{
		return search( str, array, mid+1, end, index ); //we know the item is not mid. increment to avoid
	}
	else
	{
		return search( str, array, start, mid, index );
	}
}

/* replace_char
 *
 * Purpose: In the given string, replaces the char given by replace with the
 *   char given by with in the given string
 *
 * Parameters:
 *   char *str - string to replace characters in
 *   char replace - character being replaced
 *   char with - character to replace with
 *
 * Returns: nothing
 */
void replace_char( char *str, char replace, char with )
{
	for ( int pos = 0; str[pos] != '\0'; pos++ )
	{
		if ( str[pos] == replace )
		{
			str[pos] = with;
		}
	}
}

int is_double( char *str )
{
	char *input_end = NULL;
	strtod( str, &input_end );

	//if the conversion was unsuccessful, input_end will be not be '\0' and 'recall last' will not be present
	if ( (strcmp(str, ":") != 0) && input_end && (input_end[0] != '\0') )
	{
		return 0;
	}

	return 1;
}

