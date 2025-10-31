#include <stdint.h>

typedef struct {
    uint32_t memory;
    uint64_t cpu_time;
    uint32_t real_time;
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

char* judger_judge(CRunSpec spec);

void judger_free(char* return_value);