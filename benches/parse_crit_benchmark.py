import sys
import re


def main():
    input_data = sys.stdin.read()
    table_header = '\n\n| Test Description | Times | Outliers |\n| --- | --- | --- |\n'
    table_body = ''

    benchmarks = re.findall(r'^(.+)\n?\s+time:\s+\[(.+)\]([\s\S]+?)Found (\d+) outliers',
                            input_data, re.MULTILINE)

    for benchmark in benchmarks:
        test_description = benchmark[0]
        time = benchmark[1]
        outliers = benchmark[3]

        table_body += f'| {test_description} | {time} | {outliers} |\n'

    markdown_table = table_header + table_body
    print(markdown_table)


if __name__ == '__main__':
    main()
