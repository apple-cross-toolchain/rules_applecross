#include <stddef.h>

#include "tests/layering_check/base.h"

size_t user_value(void) { return (size_t)base_value(); }
