#include <stdio.h>
#include <string.h>

#define BUFFER_SIZE 1024

int main() {
    char buffer[BUFFER_SIZE];
    fgets(buffer, BUFFER_SIZE, stdin);
    int length = strlen(buffer);
    for (int i = 0; i < length / 2; i++) {
        char tmp = buffer[i];
        int end_index = length - i;
        buffer[i] = buffer[end_index];
        buffer[end_index] = tmp;
    }
    printf("%s\n", buffer);
}