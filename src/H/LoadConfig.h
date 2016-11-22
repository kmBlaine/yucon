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
 * LoadConfig.h
 *
 *  Created on: Nov 18, 2016
 *      Author: kbm1271
 */

#ifndef H_LOADCONFIG_H_
#define H_LOADCONFIG_H_

#include "UnitList.h"
#define MAX_LINE_LENGTH 512

UnitNode *load_units_list();

#endif /* H_LOADCONFIG_H_ */
