#!/bin/bash

cd "$(dirname "$0")/.."

function get_time_us() {
    date +%s%6N
}

# ================= Global Envs ===================
ROOT="tests"
EXAMPLE="long-loop"
VERSION="ac"
# =================================================

# # ====================== C ======================
# TARGET_PATH="$ROOT/$EXAMPLE/$VERSION/c"
# gcc $TARGET_PATH/main.c -o $TARGET_PATH/main.o

# START_TIME=$(get_time_us)
# sudo ./qingdao-judge/output/libjudger.so \
#     --max_cpu_time=1000 \
#     --max_real_time=2000 \
#     --max_memory=10000000 \
#     --max_process_number=200 \
#     --max_output_size=134217728 \
#     --exe_path="$TARGET_PATH/main.o" \
#     --input_path="$ROOT/$EXAMPLE/testcases/1.in" \
#     --output_path="$TARGET_PATH/1.out" \
#     --error_path="$TARGET_PATH/1.error" \
#     --log_path="$TARGET_PATH/1.log" \
#     --uid=65534 \
#     --gid=65534 \
#     --seccomp_rule_name="c_cpp_file_io"
# END_TIME=$(get_time_us)
# echo "$((END_TIME - START_TIME))ms elapsed"
# # =================================================

# ===================== C++ =======================
TARGET_PATH="$ROOT/$EXAMPLE/$VERSION/cpp"
g++ $TARGET_PATH/main.cpp -o $TARGET_PATH/main.o
sudo ./qingdao-judge/output/libjudger.so \
    --max_cpu_time=10000000 \
    --max_real_time=1000000000 \
    --max_memory=10000000 \
    --max_process_number=200 \
    --max_output_size=134217728 \
    --exe_path="$TARGET_PATH/main.o" \
    # --input_path="$ROOT/$EXAMPLE/testcases/1.in" \
    --output_path="$TARGET_PATH/1.out" \
    --error_path="$TARGET_PATH/1.error" \
    --log_path="$TARGET_PATH/1.log" \
    --uid=0 \
    --gid=0 
    # \
    # --seccomp_rule_name=""

# ==== Python ====
# sudo ./qingdao-judge/output/libjudger.so \
#     --max_cpu_time=1000 \
#     --max_real_time=2000 \
#     --max_memory=10000000 \
#     --max_process_number=200 \
#     --max_output_size=134217728 \
#     --exe_path="/usr/bin/python3" \
#     --args="examples/read-and-print/correct/main.py" \
#     --input_path="testcases/$EXAMPLE/1.in" \
#     --output_path="testcases/$EXAMPLE/$VERSION.out" \
#     --error_path="testcases/$EXAMPLE/$VERSION.error" \
#     --log_path="testcases/$EXAMPLE/$VERSION.log" \
#     --uid=0 \
#     --gid=0 \
#     --seccomp_rule_name="general"

# ==== JAVA ====
# sudo ./qingdao-judge/output/libjudger.so \
    # --max_cpu_time=1000 \
    # --max_real_time=2000 \
    # --max_memory=-1 \
    # --max_process_number=200 \
    # --max_output_size=134217728 \
    # --exe_path="/usr/bin/java" \
    # --args="-cp" \
    # --args="examples/read-and-print/correct" \
    # --args="-Dfile.encoding=UTF-8" \
    # --args="-Djava.awt.headless=true" \
    # --args="-Djava.security.policy==java-policy" \
    # --args="-XX:+UseSerialGC" \
    # --args="Main" \
    # --input_path="testcases/$EXAMPLE/1.in" \
    # --output_path="testcases/$EXAMPLE/$VERSION.out" \
    # --error_path="testcases/$EXAMPLE/$VERSION.error" \
    # --log_path="testcases/$EXAMPLE/$VERSION.log" \
    # --uid=0 \
    # --gid=0 \
    # --seccomp_rule_name="general"

# gcc examples/$EXAMPLE/$VERSION/main.c -o examples/$EXAMPLE/correct/c

# C: --exe_path="examples/$EXAMPLE/$VERSION/c" \
# JAVA: java -cp examples/read-and-print/correct/ -Dfile.encoding=UTF-8 -Djava.awt.headless=true -Djava.security.policy==java-policy -XX:+UseSerialGC -DBOJ Main
# 