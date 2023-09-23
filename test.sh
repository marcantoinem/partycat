#!/bin/bash
input=test.txt
while read -r line;
do
    echo "$line";
    sleep 1;
done < "$input"