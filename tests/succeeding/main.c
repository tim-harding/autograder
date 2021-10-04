#include <stdio.h>
#include <string.h>

#define BUFFER_SIZE 1024

int main() {
    char buffer[BUFFER_SIZE];
    char* read_start = buffer;
    while (1) {
        char* read_result = fgets(read_start, BUFFER_SIZE, stdin);
        if (read_result == NULL) {
            break;
        }
        unsigned length = strlen(read_start);
        read_start += length;
    }
    int length = strlen(buffer);
    for (int i = 0; i < length / 2; i++) {
        char tmp = buffer[i];
        int end_index = length - i - 1;
        buffer[i] = buffer[end_index];
        buffer[end_index] = tmp;
    }
    printf("%s\n", buffer);
}