#include <stdio.h>
#include <string>
#include <TargetConditionals.h>
#include "tests/layering_check/base.h"

int user_value() { std::string s = "x"; return base_value() + (int)s.size() + TARGET_OS_IPHONE; }
