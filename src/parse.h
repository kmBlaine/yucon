/* File: parse.h
 *   Author: Blaine Murphy
 *   Created: 2017-04-23
 *
 * DESCRIPTON:
 * Public declarations for parse.c. For helper parsing functions
 */

#ifndef PARSE_H_
#define PARSE_H_

extern const char *search( const char*, const char*[], int, int, int* );
extern void replace_char( char*, char, char );
extern int is_double( char* );

#endif /* PARSE_H_ */
