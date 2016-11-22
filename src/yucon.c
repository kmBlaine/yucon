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
 * main.c
 *
 *  Created on: Nov 5, 2016
 *      Author: kbm1271
 */


#include <stdlib.h>
#include <stdio.h>

#include "H/Interpreter.h"


int main( int argc, char *argv[] )
{
	if ( argc < 2 )
	{
		while ( interactive_mode() );
	}
	else
	{
		convert( argc, argv );
	}

	return EXIT_SUCCESS;
}
