#include <stdio.h>
#include <sys/socket.h>
#include <stdlib.h>

int create_socket() {
    // AF_INET: IPv4 인터넷 프로토콜
    // SOCK_STREAM: TCP (스트림 소켓)
    // 0: 프로토콜 (일반적으로 0)
    int sockfd = socket(AF_INET, SOCK_STREAM, 0); 
    return sockfd;
}

int main() {
    int socket_descriptor = create_socket();

    if (socket_descriptor < 0) {
        // socket() 호출 실패 시 -1 반환
        perror("Socket creation failed");
        return EXIT_FAILURE;
    }

    printf("Socket created successfully.\n");
    printf("New socket descriptor: %d\n", socket_descriptor);

    // 실제 사용하지 않을 것이므로 닫습니다.
    close(socket_descriptor);

    return EXIT_SUCCESS;
}