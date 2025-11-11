#!/bin/bash

set -e

# =================================
PROBLEM="BOJ-20183"
SUBMISSION="ac"
LANGUAGE="cpp"
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
TEST_PATH="$WORKSPACE/tests/$PROBLEM/$SUBMISSION/$LANGUAGE"

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

# 5. Run CLI program to run judgers and evaluate results
cd $WORKSPACE/evaluator
sudo env "QOJ_LIBJUDGER_PATH=$WORKSPACE/qingdao-judger/output/libjudger.so" \
    "QOJ_LIBJUDGER_V2_PATH=$WORKSPACE/qingdao-judger-v2/output/libjudger.so" \
    "CONTAINER_ID=9f20c17da8cf2a75750fee824e83b634f9cdffcac0eb1eae736dc30b420f7f76" \
    go run . \
    --problem       $PROBLEM \
    --submission    $SUBMISSION \
    --language      $LANGUAGE
