#include <stdint.h>

typedef struct {
    uint64_t memory;
    uint32_t cpu_time;
    uint32_t real_time;
    uint32_t stack;
    uint16_t n_process;
    uint32_t output;
} CResourceLimit;

typedef struct {
    const char *exe_path;
    const char *input_path;
    const char *answer_path;
    const char *output_path;
    const char *error_path;
    const char *args;
    const char *envs;
    uint8_t scmp_policy;
    CResourceLimit resource_limit;
} CJudgeSpec;

char* judger_judge(CJudgeSpec spec);

void judger_free(char* return_value);

int judger_configure_logger(const char* log_path);