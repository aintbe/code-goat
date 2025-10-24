#!/bin/bash

# =========================================================
# strace 출력에서 고유한 시스템 호출 이름 추출
#
# 사용법: ./parse_syscall.sh ./your_program_name [program_args...]
# 예시: ./parse_syscall.sh ls -l /
# =========================================================
if [ "$#" -eq 0 ]; then
    echo "오류: 실행할 프로그램과 인자를 입력하세요." >&2
    echo "사용법: $0 ./your_program_name [program_args...]" >&2
    exit 1
fi

# Call strace and redirect to stderr
strace -f "$@" 2>&1 |

# Extract the first word(= syscall) from each line
grep -E '^[a-z0-9_]+\(' |
cut -d'(' -f1 |

# No duplication, in alphabetical order
sort -u
