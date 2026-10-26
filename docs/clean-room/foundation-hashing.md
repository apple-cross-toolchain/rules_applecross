# Hash-table orders

Several Apple formats write a hash table's contents in enumeration order:
binary property lists their dictionaries' keys, NIB archives their key
tables. Both start from CoreFoundation's string hash (`CFHash` of a string),
as published in CFString.c; `-[NSString hash]` on Xcode 27's macOS returns
the same values.

## CoreFoundation (binary property lists)

Linear probing over bucket counts 3, 7, 13, 23, 41, 71, ... from the
published CFBasicHash, rehashing in enumeration order. A full 3-bucket
table grows before the next insertion; larger ones as soon as an insertion
takes them past their capacity (6, 11, 19, 32, 52, ...), so the entry that
does so is rehashed with the rest rather than added afterwards. A
storyboard's 31 identifiers, whose 20th insertion collides, show the
difference; actool's table of identifiers grows the same way (258
catalogs). The compiled storyboard's Info.plist and WatchKit interface
follow it. A dictionary
created whole (as reading a binary property list creates them) starts at
the size its entries need and is filled in order without rehashing; a
linked WatchKit interface follows that.

## Foundation's NSMutableSet (NIB key tables)

Measured with objects whose `-hash` was chosen and whose enumeration order
was read back, for sets of 1-420 objects: buckets `hash mod size` with
linear probing, sizes 3, 7, 13, 23, 41, 71, 127, 191, 251, 383, and growth
that rehashes in enumeration order. The 3 and 7 bucket tables grow when an
insertion would overfill them; larger ones as soon as an insertion brings
them to 13, 21, 34, 54, 87, 120, 157 or 239 entries. Sets of 400 or more are
not established.
