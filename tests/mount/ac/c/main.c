#include <stdio.h>

int main() {
    // 민감한 파일의 경로
    const char *sensitive_file = "/etc/passwd";
    FILE *file_ptr;

    printf("'%s' 파일을 열려고 시도합니다...\n", sensitive_file);

    // 파일을 읽기 모드("r")로 열기를 시도
    file_ptr = fopen(sensitive_file, "r");

    // fopen() 함수는 파일 열기에 실패하면 NULL을 반환합니다.
    if (file_ptr == NULL) {
        // 실패 이유를 함께 출력합니다 (예: "Permission denied")
        perror("파일 열기 실패");
        printf("예상된 결과입니다. 민감한 파일은 루트(root) 권한 없이는 접근할 수 없습니다. 👍\n");
        return 1; // 실패를 의미하는 1을 반환하며 종료
    }

    // 만약 파일 열기에 성공했다면 (예: 루트 권한으로 실행했다면)
    printf("파일 열기 성공! (루트 권한으로 실행하셨나요?)\n");
    fclose(file_ptr); // 파일을 닫아줍니다.

    return 0; // 성공을 의미하는 0을 반환하며 종료
}