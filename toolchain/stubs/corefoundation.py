"""CoreFoundation's hashing, as the published CFString.c and CFBasicHash
behave: the order a CoreFoundation dictionary or set enumerates its string
keys in, which the property lists and archives Apple's tools write
follow. Shared by the stubs installed beside it."""

_M64 = (1 << 64) - 1
_CF_SIZES = [3, 7, 13, 23, 41, 71, 127, 191, 251, 383, 631, 1087]
_CF_CAPACITIES = [3, 6, 11, 19, 32, 52, 85, 118, 155, 237, 390, 672]


def cf_string_hash(text):
    """CFHash of a string (the published CFString.c)."""
    raw = text.encode("utf-16-le")
    u = [raw[i] | (raw[i + 1] << 8) for i in range(0, len(raw), 2)]
    n = len(u)
    r = n

    def four(r, i):
        return (r * 67503105 + u[i] * 16974593 + u[i + 1] * 66049 + u[i + 2] * 257 + u[i + 3]) & _M64
    if n <= 96:
        i = 0
        while i < (n & ~3):
            r = four(r, i)
            i += 4
        while i < n:
            r = (r * 257 + u[i]) & _M64
            i += 1
    else:
        for start in (0, (n >> 1) - 16, n - 32):
            for i in range(start, start + 32, 4):
                r = four(r, i)
    return (r + (r << (n & 31))) & _M64


def cf_dictionary_order(keys):
    """The order a mutable CoreFoundation dictionary filled with `keys`, in
    that order, enumerates them: linear probing over the published
    CFBasicHash sizes, a full 3-bucket table growing before the next
    insertion and larger ones as soon as one takes them past their
    capacity, rehashing in enumeration order."""
    hashes = {k: cf_string_hash(k) for k in keys}
    level = 0
    table = [None] * _CF_SIZES[level]

    def place(tbl, k):
        i = hashes[k] % len(tbl)
        while tbl[i] is not None:
            i = (i + 1) % len(tbl)
        tbl[i] = k

    def grow():
        nonlocal level, table
        level += 1
        old = [x for x in table if x is not None]
        table = [None] * _CF_SIZES[level]
        for x in old:
            place(table, x)
    count = 0
    for k in keys:
        if _CF_SIZES[level] == 3 and count == 3:
            grow()
        place(table, k)
        count += 1
        if _CF_SIZES[level] > 3 and count > _CF_CAPACITIES[level]:
            grow()
    return [x for x in table if x is not None]


def cf_created_order(keys):
    """The order a CoreFoundation dictionary or set created from all of
    `keys` at once enumerates them: a table sized for them from the start,
    filled in their order without growing."""
    level = 0
    while _CF_CAPACITIES[level] < len(keys):
        level += 1
    table = [None] * _CF_SIZES[level]
    for k in keys:
        i = cf_string_hash(k) % len(table)
        while table[i] is not None:
            i = (i + 1) % len(table)
        table[i] = k
    return [x for x in table if x is not None]
