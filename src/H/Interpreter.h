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
 * Interpreter.h
 *
 *  Created on: Nov 19, 2016
 *      Author: kbm1271
 */

#ifndef H_INTERPRETER_H_
#define H_INTERPRETER_H_

#include "Convert.h"
#include "LoadConfig.h"
#include "UnitList.h"

int interactive_mode();
void convert( int, char** );

#endif /* H_INTERPRETER_H_ */
