#!/bin/bash

system_env=/proc/registry/HKEY_LOCAL_MACHINE/SYSTEM/CurrentControlSet/Control/Session\ Manager/Environment
user_env=/proc/registry/HKEY_CURRENT_USER/Environment

echo "$(cat $system_env/Path);$(cat $user_env/Path) \
 | sed -e 's!\\!/!g' \
 | sed -e 's/;/\n/g' \
 | while read line; do cygpath "${line}"; done
