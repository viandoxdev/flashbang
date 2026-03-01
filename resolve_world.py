import sys

with open("fb-core/src/world.rs", "r") as f:
    lines = f.readlines()

new_lines = []
skip = False
for line in lines:
    if line.startswith("<<<<<<< HEAD"):
        # Keep HEAD until =======
        pass
    elif line.startswith("======="):
        # Skip until >>>>>>>
        skip = True
    elif line.startswith(">>>>>>> origin/main"):
        skip = False
    else:
        if not skip and not line.startswith("<<<<<<< HEAD"):
            new_lines.append(line)

with open("fb-core/src/world.rs", "w") as f:
    f.writelines(new_lines)
