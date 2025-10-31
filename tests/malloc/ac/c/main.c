#include <stdio.h>
#include <stdlib.h>
#include <string.h> // memset을 사용하기 위해 포함
#include <unistd.h>

#define MEGABYTE (1024 * 1024)
#define SIZE_IN_BYTES (24 * MEGABYTE)

int main() {
    size_t array_size = SIZE_IN_BYTES;
    char *big_array = (char *)malloc(array_size);

    if (big_array == NULL) {
        perror("Memory allocation failed");
        return EXIT_FAILURE;
    }

    // 🌟 이 부분이 중요합니다!
    // 할당된 24MB 전체 영역을 'A' 문자로 채웁니다.
    // 이렇게 하면 운영체제는 24MB 전체에 대해 물리적 메모리를 할당하게 됩니다.
    printf("Writing 'A' to all 24MB to force physical memory allocation...\n");
    memset(big_array, 'A', array_size);
    printf("Write operation completed.\n");

    /* * 이제 시스템 모니터링 도구(예: top, htop, task manager)에서 
    * 이 프로세스의 실제 메모리 사용량(Resident Set Size, RSS)을 확인해 보세요.
    * 24MB에 가까운 숫자가 보일 것입니다.
    */

    // 잠시 멈춰서 사용자가 메모리 사용량을 확인할 시간을 줍니다.
    // printf("Program paused. Press Enter to free memory and exit...\n");
    // getchar(); 
    
    free(big_array);
    printf("Memory freed successfully.\n");

    return EXIT_SUCCESS;
}