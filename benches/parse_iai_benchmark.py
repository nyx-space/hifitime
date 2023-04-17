import sys
import re


def main():
    input_data = sys.stdin.read()
    table_header = "\n\n| Benchmark | Instructions | L1 Accesses | L2 Accesses | RAM Accesses | Estimated Cycles |\n| --- | --- | --- | --- | --- | --- |\n"
    table_body = ""

    benchmarks = input_data.split("\n\n")

    for benchmark in benchmarks:
        splt = [s.strip() for s in benchmark.split("\n")]
        if len(splt) != 6:
            continue
        name = splt[0]
        instructions = re.search(r"Instructions:\s+(\d+)", splt[1]).group(1)
        l1_accesses = re.search(r"L1 Accesses:\s+(\d+)", splt[2]).group(1)
        l2_accesses = re.search(r"L2 Accesses:\s+(\d+)", splt[3]).group(1)
        ram_accesses = re.search(r"RAM Accesses:\s+(\d+)", splt[4]).group(1)
        estimated_cycles = re.search(r"Estimated Cycles:\s+(\d+)", splt[5]).group(1)

        table_body += f"| {name} | {instructions} | {l1_accesses} | {l2_accesses} | {ram_accesses} | {estimated_cycles} |\n"

    markdown_table = table_header + table_body
    print(markdown_table)


if __name__ == "__main__":
    main()
