#!/bin/bash

pattern=$1
file=$2
context=$3


# Get all line numbers
line_numbers=$(rg --color=never --line-number --no-heading "$pattern" "$file" | cut -d: -f1)

if [ -z "$line_numbers" ]; then
    echo "No matches found"
    exit 1
fi

language="${file##*.}"

count=0
for line_num in $line_numbers; do
    count=$((count + 1))
    
    if [ $count -gt 1 ]; then
        echo
        echo "--------------------------------------------"
        echo
    fi

    bat -r "$line_num:+$context" --style=numbers --color=always --language="$language" "$file"

done
