#!/bin/bash

set -e

# =================================
EXAMPLE="long-loop"
VERSION="ac"
LANGUAGE="cpp"
TESTCASE=1
# ==================================

# 1. Process arguments
BUILD_FLAG=0
COMPILE_FLAG=0
GRADE_FLAG=0

while [ "$1" != "" ]; do
    case "$1" in
        -b | --build )
            BUILD_FLAG=1
            ;;
        -c | --compile )
            COMPILE_FLAG=1
            ;;
        -g | --grade )
            GRADE_FLAG=1
            ;;
        -h | --help )
            echo "Usage: $0 [-b, --build] [-c, --compile]"
            exit 0
            ;;
        * )
            echo "Invalid option: $1"
            echo "Try '--help' for more information."
            exit 1
            ;;
    esac
    shift # To next argument
done

# 2. Set up test environments
WORKSPACE=$(readlink -f "$(dirname $0)/..")
TEST_PATH="$WORKSPACE/tests/$EXAMPLE/$VERSION/$LANGUAGE"

# 3. Perform building libjudger.so if requested
if [ $BUILD_FLAG -eq 1 ]; then
    cd $WORKSPACE/code-goat
    cargo build --release -p judger
    sudo cp /$WORKSPACE/code-goat/target/release/libjudger.so /usr/local/lib

    cd $WORKSPACE/qingdao-judger
    rm -rf build && rm -f CMakeCache.txt
    cmake . && make && sudo make install
fi

# 4. Perform compilation if requested or the executable file does not exists
if [ "$COMPILE_FLAG" -eq 1 ] || [ ! -f "$TEST_PATH/main" ]; then

    case "$LANGUAGE" in
        "c" )
            gcc "$TEST_PATH/main.c" -o "$TEST_PATH/main"
            ;;
        "cpp" )
            g++ "$TEST_PATH/main.cpp" -o "$TEST_PATH/main"
            ;;
        # "go")
        #     go build -o "$TEST_PATH/main" "$TEST_PATH/main"
        #     ;;
        # "java")
        #     # Also need to change how to run it 
        #     javac -d "$TEST_PATH" "$TEST_PATH/Main.java"
        #     ;;  
        * )
            echo "Error: Unsupported language '$LANGUAGE'"
            exit 1 # Terminate
            ;;
    esac
fi

# 5. Adjust input & output arguments based on existing files
INPUT_PATH="$WORKSPACE/tests/$EXAMPLE/testcases/$TESTCASE.in"
if [ ! -f "$INPUT_PATH" ]; then
    INPUT_PATH=""
fi

ANSWER_PATH="$WORKSPACE/tests/$EXAMPLE/testcases/$TESTCASE.out"
if [ "$GRADE_FLAG" -eq 0 ] || [ ! -f "$ANSWER_PATH" ]; then
    ANSWER_PATH=""
fi

# 4. Run CLI program to run judgers and evaluate results
cd $WORKSPACE/evaluator
sudo env "LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH" go run . \
    --exe-path          "$TEST_PATH/main" \
    --input-path        "$INPUT_PATH" \
    --output-path       "$TEST_PATH/$TESTCASE.out" \
    --error-path        "$TEST_PATH/$TESTCASE.error" \
    --answer-path       "$ANSWER_PATH" \
    --args              "" \
    --args              "" \
    --envs              "" \
    --envs              "" \
    --memory-limit      100000000 \
    --cpu-time-limit    1000 \
    --real-time-limit   2000 \
    --stack-limit       0 \
    --n-process-limit   0 \
    --output-limit      100000000
