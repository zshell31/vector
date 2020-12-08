package metadata

remap: {
	#Timestamp: {
		#Formats: [Category=string]: [Format=string]: {
			category: Category
			format:   Format
			example?: string
		}

		type: "timestamp"

		reference: "Sunday, July 8th, 2001"

		specifiers: #Formats
		specifiers: {
			"Date": {
				"%Y": {
					description: "The full [proleptic Gregorian](\(urls.gregorian) year, zero-padded to 4 digits. Negative years are allow in formatting but not in parsing."
					example:     "2001"
				}
				"%C": {
					description: "The full [proleptic Gregorian](\(urls.gregorian) year divided by 100, zero-padded to 2 digits. Uses floor division, so 100 BCE (year number -99) prints `-1`."
					example:     "20"
				}
				"%y": {
					description: "The [proleptic Gregorian](\(urls.gregorian)) year modulo 100, zero-padded to 2 digits. Uses floor division, so 100 BCE (number -99) prints 99."
					example:     "01"
				}
				"%m": {
					description: "Month number (`01` through `12`), zero-padded to 2 digits."
					example:     "07"
				}
				"%b": {
					description: "Abbreviated month name. Always 3 letters."
					example:     "Jul"
				}
				"%B": {
					description: "Full month name. Also accepts corresponding abbreviation in parsing."
					example:     "July"
				}
				"%h": {
					description: "Same as `%b`"
					example:     "Jul"
				}
				"%d": {
					description: "Day number (`01` to `31`), zero-padded to 2 digits."
					example:     "08"
				}
				"%e": {
					description: "Same as `%d` but space padded. Also the same as `%_d`."
					example:     "8"
				}
				"%a": {
					description: "Abbreviated weekday name. Always 3 letters."
					example:     "Sun"
				}
				"%A": {
					description: "Full weekday name. Also accepts corresponding abbreviation in parsing."
					example:     "Sunday"
				}
				"%w": {
					description: "The weekday as an integer with Sunday as `0` and Saturday as `7`."
					example:     "0"
				}
				"%u": {
					description: "The weekday as an integer with Monday as `0` and Sunday as `7`. [ISO 8601](\(urls.iso_8601)) compliant."
					example:     "7"
				}
				"%U": {
					description: "The week number starting with Sunday (`00` through `53`), zero-padded to 2 digits. Week 1 starts with the first Sunday in that year. Week 0 represents days before the first Sunday."
					example:     "28"
				}
				"%W": {
					description: "Like `%U` except week 1 starts with the first Monday in that year instead."
					example:     "27"
				}
				"%G": {
					description: "The same as `%Y` but uses the year number in the [ISO 8601](\(urls.iso_8601)) week date. "
					example:     "2001"
				}
				"%g": {
					description: "The same as `%y` but uses the year number in the [ISO 8601](\(urls.iso_8601)) week date."
					example:     "01"
				}
				"%V": {
					description: "The same as `%U` but uses the week number in [ISO 8601](\(urls.iso_8601)) week date (`01` to `53`)."
					example:     "27"
				}
				"%j": {
					description: "Day of the year (`001` to `366`), zero-padded to 3 digits."
					example:     "189"
				}
				"%D": {
					description: "Month-day-year format ([ISO 8601](\(urls.iso_8601)). Equivalent to `%m/%d/%y`."
					example:     "07/08/01"
				}
				"%x": {
					description: "The same as `%D`."
					example:     "07/08/01"
				}
				"%F": {
					description: "Year-month-day format ([ISO 8601](\(urls.iso_8601)). Equivalent to `%Y-%m-%d`."
					example:     "2001-07-08"
				}
				"%v": {
					description: "Day-month-year format. Equivalent to `%e-%b-%Y`."
					example:     "8-Jul-2001"
				}
			}
			"Time": {
				"%H": {
					description: "The hour number (`00` through `23`), zero-padded to 2 digits."
					example:     "00"
				}
				"%k": {
					description: "Equivalent to `%H` but space-padded. Also equivalent to `%_H`."
					example:     "0"
				}
				"%I": {
					description: "Hour number for 12-hour clocks (`01` to `12`), zero-padded to 2 digits."
					example:     "12"
				}
				"%l": {
					description: "Equivalent to `%I` but spaced padded. Also equivalent to `%_I`."
					example:     "12"
				}
				"%P": {
					description: "`am` or `pm` for 12-hour clocks."
					example:     "am"
				}
				"%p": {
					description: "`AM` or `PM` for 12-hour clocks."
					example:     "AM"
				}
				"%M": {
					description: "Minute number (`00` to `59`), zero-padded to 2 digits."
					example:     "34"
				}
				"%S": {
					description: "Second number (`00`--60), zero-padded to 2 digits."
					example:     "60"
				}
				"%f": {
					description: "The fractional seconds (in nanoseconds) since last whole second."
					example:     "026490000"
				}
				"%.f": {
					description: "Similar to .%f but left-aligned. These all consume the leading dot. "
					example:     ".026490"
				}
				"%.3f": {
					description: " 	Similar to .%f but left-aligned but fixed to a length of 3."
					example:     ".026"
				}
				"%.6f": {
					description: " 	Similar to .%f but left-aligned but fixed to a length of 6."
					example:     ".026490"
				}
				"%.9f": {
					description: "Similar to .%f but left-aligned but fixed to a length of 9."
					example:     ".026490000"
				}
				"%3f": {
					description: " 	Similar to %.3f but without the leading dot."
					example:     "026"
				}
				"%6f": {
					description: "Similar to %.6f but without the leading dot."
					example:     "026490"
				}
				"%9f": {
					description: "Similar to %.9f but without the leading dot."
					example:     "026490000"
				}
				"%R": {
					description: "Hour-minute format. Same to %H:%M."
					example:     "00:34"
				}
				"%T": {
					description: "Hour-minute format. Same to %H:%M."
					example:     "00:34:60"
				}
				"%X": {
					description: "Hour-minute-second format. Same to %H:%M:%S."
					example:     "00:34:60"
				}
				"%r": {
					description: "Hour-minute-second format in 12-hour clocks. Same to %I:%M:%S %p. "
					example:     "12:34:60 AM"
				}
			}
			"Time zone": {
				"%Z": {
					description: "Local time zone name."
					example:     "ACST"
				}
				"%z": {
					description: "Offset from the local time to UTC (with UTC being `+0000`)."
					exampl:      "+0930"
				}
				"%:z": {
					description: "Same to %z but with a colon."
					example:     "+09:30"
				}
				"%#z": {
					description: "Same to %z but allows minutes to be missing or present."
					example:     "+09"
				}
			}
			"Date and time": {
				"%c": {
					description: "ctime date & time format. Same to %a %b %e %T %Y sans `\n`."
					example:     "Sun Jul 8 00:34:60 2001"
				}
				"%+": {
					description: "ISO 8601 / RFC 3339 date & time format."
					example:     "2001-07-08T00:34:60.026490+09:30"
				}
				"%s": {
					description: "UNIX timestamp, i.e. the number of seconds since 1970-01-01 00:00 UTC."
					example:     "994518299"
				}
			}
			"Special": {
				"%t": {
					description: "Literal tab (`\t`)."
				}
				"%n": {
					description: "Literal newline (`\n`)."
				}
				"%%": {
					description: "Literal percent sign."
				}
			}
		}
	}

	#RemapParameterTypes: "path" | "float" | "integer" | "string" | "timestamp" | "boolean" | "array" | "map" | "regex" | "any"

	#RemapReturnTypes: "float" | "integer" | "string" | "timestamp" | "boolean" | "array" | "map" | "null"

	{
		description: """
			The Timber Remap Language (TRL) is a purpose-driven, Rust-native data
			mapping language that enables Vector users to easily map and reshape data
			without sacrificing performance or safety. It's a middle ground between
			stringing together many fundamental [transforms](\(urls.vector_transforms))
			and a full blown runtime like Lua. Principles of TRL include:

			1. **Performance** - Beyond extremely fast execution, TRL is designed to
			   prevent operators from writing slow scripts.
			2. **Safety** - TRL is Rust-native and performs compile-time checks on
			   boot that ensure safety. In addition, TRL is designed for
			   collaboration. It is intentionally simple, avoiding footguns introduced
			   through complexity.
			3. **Easy** - A TRL script is obvious at first glance. It is designed to
			   have little, if any, learning curve.

			TRL is designed and maintained by Timber and built specific for processing
			data within Vector.
			"""

		errors: [Name=string]: {
			description: string
			name:        Name
		}

		functions: [Name=string]: {
			#Argument: {
				name:        string
				description: string
				required:    bool
				multiple:    bool | *false
				default?:    bool | string | int
				type: [#RemapParameterTypes, ...#RemapParameterTypes]
			}
			#RemapExample: {
				title: string
				configuration?: [string]: string
				input:  #Fields
				source: string
				output: #Fields
			}

			arguments: [...#Argument] // Allow for empty list
			return: [#RemapReturnTypes, ...#RemapReturnTypes]
			category:    "coerce" | "numeric" | "object" | "parse" | "text" | "hash" | "event" | "networking"
			description: string
			examples: [#RemapExample, ...#RemapExample]
			name: Name
		}
	}

	errors: {
		ArgumentError: {
			description: "Raised when the provided input is not a supported type."
		}
		ParseError: {
			description: "Raised when the provided input cannot be parsed."
		}
	}
}
