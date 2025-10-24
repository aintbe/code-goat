#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <errno.h>      // errno를 사용하기 위해 추가
#include <string.h>     // strerror를 사용하기 위해 추가
#include <sys/wait.h>

int main() {
    
    pid_t pid = fork();
    
    int x;
    x = 0;
    
    if(pid > 0) {  // 부모 코드
        x = 1;
        printf("부모 PID : %ld,  x : %d , pid : %d\n",(long)getpid(), x, pid);

        int status;
        wait(&status);
    }
    else if(pid == 0){  // 자식 코드
        x = 2;
        printf("자식 PID : %ld,  x : %d\n",(long)getpid(), x);
    }
    else {  // fork 실패
        // ⭐️⭐️ 실패 원인 출력 ⭐️⭐️
        printf("fork Fail! (errno: %d, Message: %s)\n", errno, strerror(errno));
        return -1;
    }
    
    return 0;
}