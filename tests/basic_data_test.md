# Basic Data Processing Test

This test focuses on creating, manipulating, and accessing data blocks.

## Define Data

[data name:test-numbers format:json]
[1, 2, 3, 4, 5]
[/data]

## Process Data With Python

[code:python name:sum-numbers cache_result:true fallback:sum-numbers-fallback]
import json

numbers = json.loads('''${test-numbers}''')
total = sum(numbers)
print(f"The sum is {total}")
[/code:python]

[code:python name:sum-numbers-fallback]
print("Failed to calculate sum")
[/code:python]

## Expected Output

The expected output should be: "The sum is 15"
