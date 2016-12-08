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

/* File: Convert.h
 *   Author: Blaine Murphy
 *   Created: 2016-11-22
 *
 * DESCRIPTION:
 *
 * This is the front end for the Convert.c module. The conversion utilities
 * and formatting are made separate so different parts of the program may
 * use the same code for unit conversion and formatting.
 */

#ifndef H_CONVERT_H_
#define H_CONVERT_H_

#define OUTPUT_STR_SIZE 128

int get_conversion( char*, char*, char*, double* );

char *simple_output_str( double );
char *descriptive_output_str( double, char* );
char *verbose_output_str( double, char*, char*, char* );

#endif /* H_CONVERT_H_ */
