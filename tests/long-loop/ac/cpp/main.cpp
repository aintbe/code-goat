#include <iostream>

int main() {
    // 'volatile' 키워드는 컴파일러가 이 반복문을
    // "아무것도 안 하는 코드"로 판단하고 최적화(삭제)하는 것을 방지합니다.
    // 'volatile' 변수에 값을 쓰면, CPU는 실제로 메모리에 
    // 해당 값을 쓰는 작업을 수행해야 합니다.
    volatile unsigned long long counter = 0;

    // 이 숫자는 컴퓨터의 CPU 속도에 따라 1초 이상 걸리도록
    // 충분히 커야 합니다. 50억 번 정도면 웬만한 최신 CPU에서도 1초를 넘깁니다.
    // (숫자가 너무 작으면 1초 이내에 끝날 수 있습니다.)
    unsigned long long iterations = 50000000000ULL;

    std::cout << "Starting CPU-intensive loop for >1 second..." << std::endl;

    for (unsigned long long i = 0; i < iterations; ++i) {
        counter += 1;
    }

    // 루프가 완료되었음을 알리는 메시지 (실제로는 counter 값을 사용)
    std::cout << "Loop finished. Result: " << counter << std::endl;

    return 4;
}