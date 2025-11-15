#!/bin/bash

set -e

PROBLEM=""
SUBMISSION_LIST=()


show_help() {
    echo "Usage: $0 -p <problem> -s <sub1[,sub2,sub3...]>"
    exit 0
}
if [ "$#" -eq 0 ]; then
    show_help
fi

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        # 긴 플래그
        -p | --problem )
            PROBLEM="$2"
            shift
            ;;
        -s | --submissions )
            # 쉼표(,)로 구분된 문자열을 배열로 변환
            IFS=',' read -r -a SUBMISSION_LIST <<< "$2"
            shift
            ;;
        -h | --help )
            show_help
            ;;
        * )
            echo "Invalid option: $1"
            echo "Try '--help' for more information."
            exit 1
            ;;
    esac
    shift # To next argument
done


WORKSPACE=$(readlink -f "$(dirname $0)/..")
LANGUAGE_LIST=(
    "c"
    "cpp"
    "python3"
    "java"
)

for sub in "${SUBMISSION_LIST[@]}"; do
    for lang in "${LANGUAGE_LIST[@]}"; do
        test_path="$WORKSPACE/tests/$PROBLEM/$sub/$lang"

        if [ ! -d "$test_path" ]; then
            mkdir -p "$test_path"
        fi
    done
    echo "✅ Added $PROBLEM/$sub/*"
done


if [[ $PROBLEM == "boj-"* ]]; then
    TOOL_DIR="$WORKSPACE/scripts/tools"
    boj_id="${PROBLEM#boj-}"
    
    $TOOL_DIR/env/bin/python3 $TOOL_DIR/scraper.py $boj_id
fi
