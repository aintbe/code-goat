#include <stdio.h>
#include <stdlib.h>

// 재귀 횟수를 기록할 전역 변수 (필수는 아니지만 관찰용)
unsigned long long call_count = 0;

void infinite_recursion(int depth) {
    // 탈출 조건이 없는 무한 재귀
    
    // 지역 변수를 사용하여 스택 프레임을 유지하고 최적화를 방해합니다.
    char dummy_data[256]; 
    dummy_data[0] = 'a'; // 사용하지 않으면 최적화될 수 있으므로 간단히 사용

    call_count++;
    
    // 일정 횟수마다 진행 상황을 출력 (선택 사항)
    if (call_count % 100000 == 0) {
        printf("Current recursion depth (call count): %llu\n", call_count);
    }
    
    // 자기 자신을 다시 호출
    infinite_recursion(depth + 1); 
}

int main() {
    printf("Starting infinite recursion. This will likely lead to a Stack Overflow crash.\n");
    
    // 함수 호출 시작
    infinite_recursion(0);

    return EXIT_SUCCESS; // 여기에 도달하지 않을 것입니다.
}