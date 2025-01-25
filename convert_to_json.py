import json
import sys

register_file = sys.argv[1]

data_structure = {}

with open(register_file) as f:
    content = f.readlines()

    for line in content:
        split_line = line.strip("\n").split("\t")
        data_structure[split_line[0]] = split_line[-1]

print(json.dumps(data_structure, indent=4))
