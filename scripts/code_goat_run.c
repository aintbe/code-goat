
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stddef.h>

// --- Define structs and functions from rust FFI ---
typedef struct {
    uint32_t memory;
    uint64_t cpu_time;
    uint64_t real_time;
    uint64_t stack;
    uint64_t n_process;
    uint64_t output;
} CResourceLimit;

typedef struct {
    const char *exe_path;
    const char *input_path;
    const char *output_path;
    const char *error_path;
    const char *answer_path;
    const char *args;
    const char *envs;
    CResourceLimit resource_limit;
} CRunSpec;

char* c_judge(CRunSpec spec);

void c_free(char* return_value);
// --------------------------------------------------

#define BASE_DIR "/workspaces/code-goat/tests"
#define EXAMPLE_DIR "/a+b"
#define WORK_PATH(file) BASE_DIR EXAMPLE_DIR "/ac/cpp" file
#define TEST_PATH(file) BASE_DIR EXAMPLE_DIR "/testcases" file

int main() {
    // 1. Rust 함수에 전달할 CResourceLimit 구조체 초기화
    CResourceLimit resource_limit = {
        .memory = 1000 * 1024 * 1024, // 10MB
        .cpu_time = 1000 * 1000,     // 1초
        .real_time = 1000 * 1000,    // 2초
        .stack = 0,                 // 무제한
        .n_process = 0,             // 무제한
        .output = 0,                // 무제한
    };

    // 2. CRunSpec 구조체 초기화 (C 문자열 포인터 사용)
    // args와 envs는 예시를 위해 단순 문자열로 처리하며, 
    // 실제 로직에서는 parse_cffi 함수에서 파싱될 것입니다.
    CRunSpec spec = {
        .exe_path = WORK_PATH("/main.o"),
        .input_path = NULL, // TEST_PATH("1.in")
        .output_path = WORK_PATH("1.out"),
        .error_path = WORK_PATH("1.error"),
        .answer_path = NULL, // TEST_PATH("1.out")
        // 공백으로 구분된 인자 문자열
        .args = "", 
        .envs = "",
        .resource_limit = resource_limit,
    };

    // // 3. 📞 Rust FFI 함수 호출 (메모리 할당 시점)
    printf("C: Rust 함수를 호출하고 결과를 기다립니다...\n");
    char* json_result_ptr = c_judge(spec);

    // 4. 📝 결과 확인 및 사용
    if (json_result_ptr == NULL) {
        printf("C: 오류 발생: Rust로부터 NULL 포인터를 받았습니다.\n");
        return 1;
    }
    
    // Rust가 반환한 JSON 문자열 출력 (C 코드에서 데이터 사용)
    printf("\n--- Rust가 반환한 JSON 결과 ---\n");
    printf("%s\n", json_result_ptr);
    printf("------------------------------\n");

    // 5. 🗑️ 메모리 해제 (사용 완료 직후)
    // 💡 Rust가 할당한 메모리는 반드시 Rust가 제공한 해제 함수를 통해 해제해야 합니다.
    printf("C: Rust가 할당한 메모리를 free_cfii 함수를 통해 해제합니다.\n");
    c_free(json_result_ptr);

    // 6. 메모리 해제 후 포인터를 NULL로 설정하는 것이 좋습니다.
    json_result_ptr = NULL; 

    return 0;
}