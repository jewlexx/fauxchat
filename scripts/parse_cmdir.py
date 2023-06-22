import sys
import os


args = sys.argv[1:]
if len(args) < 1:
    print("Usage: python3 %s <file>" % (os.path.basename(__file__)))
    sys.exit(1)

file_path = args[0]

old_end = 0

with open(file_path, "r") as f:
    with open(file_path.replace(".cmdir", ".commands"), "w") as f2:
        for line in f:
            if not line.startswith("end_pause"):
                f2.write(line)
            else:
                timestamp = int(line.split("(", 1)[1].split(")", 1)[0])
                if old_end != 0:
                    f2.write(f"sleep({timestamp - old_end})\n")

                old_end = timestamp

os.remove(file_path)
