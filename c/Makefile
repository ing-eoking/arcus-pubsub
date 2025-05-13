CC = gcc
CFLAGS = -fPIC -Wall -I./include
LDFLAGS = -levent -shared

SRC = pubsub.c
OBJ = $(SRC:.c=.o)
TARGET = pubsub.so

all: $(TARGET)

$(TARGET): $(OBJ)
	$(CC) $(LDFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJ) $(TARGET)

%.o: %.h
	$(CC) $(CFLAGS) -c $< -o $@
