/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <math.h>

#include "protocol_extension.h"

/*
 * This is an example on how you can add your own commands into the
 * ascii protocol. You load the extensions into memcached by using
 * the -X option:
 * ./memcached -X .libs/example_protocol.so -E .libs/default_engine.so
 *
 * @todo add an example that require extra userspace data, and communicates
 *       with the engine by getting the engine descriptor.
 */

static const char *get_name(const void *cmd_cookie);
static bool accept_command(const void *cmd_cookie, void *cookie,
                           int argc, token_t *argv, size_t *ndata,
                           char **ptr);
static bool execute_command(const void *cmd_cookie, const void *cookie,
                            int argc, token_t *argv,
                            bool (*response_handler)(const void *cookie,
                                                     int nbytes,
                                                     const char *dta));
static void abort_command(const void *cmd_cookie, const void *cookie);

static EXTENSION_ASCII_PROTOCOL_DESCRIPTOR publish_descriptor = {
    .get_name = get_name,
    .accept = accept_command,
    .execute = execute_command,
    .abort = abort_command,
    .cookie = &publish_descriptor
};

static EXTENSION_ASCII_PROTOCOL_DESCRIPTOR subscribe_descriptor = {
    .get_name = get_name,
    .accept = accept_command,
    .execute = execute_command,
    .abort = abort_command,
    .cookie = &subscribe_descriptor
};

SERVER_HANDLE_V1 *server_api;

#define INITIAL_CAPACITY 10

typedef bool (*RespHandler)(const void*, int, const char*);

typedef struct ListElement {
    const void *value;
    struct ListElement *next;
} ListElement;

typedef struct HashTableEntry {
    char *key;
    ListElement *head;
    struct HashTableEntry *next;
} HashTableEntry;

typedef struct HashTable {
    HashTableEntry **table;
    int capacity;
    int size;
} HashTable;

HashTable *chan_ht = NULL;
HashTable *clnt_ht = NULL;

static HashTable *create_table(int capacity)
{
    HashTable *ptr = (HashTable *)malloc(sizeof(HashTable));
    if (ptr == NULL) return NULL;

    ptr->table = malloc(sizeof(HashTableEntry *) * capacity);
    if (ptr->table == NULL) {
        free(ptr);
        return NULL;
    }

    ptr->capacity = capacity;
    ptr->size = 0;

    for (int i = 0; i < capacity; i++) {
        ptr->table[i] = NULL;
    }

    return ptr;
}

static inline ListElement *is_elem_exist(HashTableEntry *entry,
                                         const void *value, ListElement **prev)
{
    ListElement *curr = entry->head;
    while (curr != NULL) {
        if (curr->value == value) break;
        if (prev) *prev = curr;
        curr = curr->next;
    }
    return curr;
}

static inline bool do_unsubscribe(const void *cookie, HashTableEntry *entry) {
    ListElement *prev = NULL;
    ListElement *curr = is_elem_exist(entry, cookie, &prev);
    if (curr == NULL) return false;
    if (prev == NULL) {
        entry->head = curr->next;
    } else {
        prev->next = curr->next;
    }
    free(curr);
    return true;
}

static ENGINE_ERROR_CODE
process_publish_command(char *channel, const size_t nchannel,
                        char *message, const size_t nmessage, RespHandler response_handler)
{
    uint32_t hashval = server_api->core->hash(channel, nchannel, 0);
    HashTableEntry *entry = chan_ht->table[hashval % chan_ht->capacity];

    while (entry != NULL) {
        if (strcmp(entry->key, channel) == 0) break;
        entry = entry->next;
    }

    if (entry != NULL) {
        ListElement *curr = entry->head;
        int msg_count = (nmessage == 0) ? 1 : (int)log10(nmessage) + 1;
        int buf_size = nchannel + msg_count + nmessage + 19;
        char *buf = (char *)malloc(buf_size);
        if (buf == NULL) {
            return ENGINE_ENOMEM;
        } else {
            snprintf(buf, buf_size, "CHANNEL %s %zu\r\n%s\r\nEND\r\n", channel, nmessage, message);
        }

        while (curr != NULL) {
            int sfd = server_api->core->get_socket_fd(curr->value);
            ssize_t bytes = send(sfd, buf, buf_size - 1, 0);
            if (bytes < 0) {
                return ENGINE_FAILED;
            }
            curr = curr->next;
        }
        free(buf);
    }

    return ENGINE_SUCCESS;
}

static ENGINE_ERROR_CODE
process_subscribe_command(const void *cookie,
                          char *channel, const size_t nchannel)
{
    uint32_t hashval = server_api->core->hash(channel, nchannel, 0);
    HashTableEntry **table = &chan_ht->table[hashval % chan_ht->capacity];
    HashTableEntry *entry = *table;

    while (entry != NULL) {
        if (strlen(entry->key) == nchannel &&
            strncmp(entry->key, channel, nchannel) == 0) {
            break;
        }
        entry = entry->next;
    }

    if (entry != NULL) {
        if (is_elem_exist(entry, cookie, NULL) != NULL) {
            return ENGINE_ELEM_EEXISTS;
        } else {
            ListElement *new_elem = (ListElement *)malloc(sizeof(ListElement));
            if (new_elem == NULL) {
                return ENGINE_ENOMEM;
            } else {
                new_elem->value = cookie;
                new_elem->next = entry->head;
                entry->head = new_elem;
            }
        }
    } else {
        entry = (HashTableEntry *)malloc(sizeof(HashTableEntry));
        char *buf = (char *)malloc(nchannel + 1);
        ListElement *new_elem = (ListElement *)malloc(sizeof(ListElement));

        if (entry == NULL || buf == NULL || new_elem == NULL) {
            if (entry)    free(entry);
            if (buf)      free(buf);
            if (new_elem) free(new_elem);
            return ENGINE_ENOMEM;
        }

        snprintf(buf, nchannel + 1, "%s", channel);
        new_elem->value = cookie;
        new_elem->next = entry->head;

        entry->key = buf;
        entry->head = new_elem;

        *table = entry;
    }

    return ENGINE_SUCCESS;
}

static ENGINE_ERROR_CODE
process_unsubscribe_command(const void *cookie, char *channel, const size_t nchannel)
{
    uint32_t hashval = server_api->core->hash(channel, nchannel, 0);
    HashTableEntry *entry = chan_ht->table[hashval % chan_ht->capacity];

    while (entry != NULL) {
        if (strcmp(entry->key, channel) == 0) break;
        entry = entry->next;
    }

    if (entry == NULL || !do_unsubscribe(cookie, entry)) {
        return ENGINE_KEY_ENOENT;
    }

    return ENGINE_SUCCESS;
}

static const char *get_name(const void *cmd_cookie) {
    if (cmd_cookie == &publish_descriptor) {
        return "publish";
    } else {
        return "subscribe";
    }
}

static bool accept_command(const void *cmd_cookie, void *cookie,
                           int argc, token_t *argv, size_t *ndata,
                           char **ptr) {
    if (cmd_cookie == &publish_descriptor) {
        if (argc == 3 && strcmp(argv[0].value, "publish") == 0) return true;
    } else if (cmd_cookie == &subscribe_descriptor) {
        if (argc >= 2 && (strcmp(argv[0].value, "subscribe") == 0 ||
                          strcmp(argv[0].value, "unsubscribe") == 0)) return true;
    }

    return false;
}

static bool execute_command(const void *cmd_cookie, const void *cookie,
                            int argc, token_t *argv, RespHandler response_handler) {
    if (cmd_cookie == &publish_descriptor) {
        process_publish_command(argv[1].value, argv[1].length,
                                argv[2].value, argv[2].length, response_handler);
        return true;
    } else if (cmd_cookie == &subscribe_descriptor) {
        ENGINE_ERROR_CODE rc;
        for (int i = 1; i < argc; i++) {
            if (argv[0].value[0] == 's') {
                rc = process_subscribe_command(cookie, argv[i].value, argv[i].length);
                switch (rc) {
                    case ENGINE_ENOMEM:
                        response_handler(cookie, 24, "SERVER_ERROR no memory\r\n"); break;
                    case ENGINE_ELEM_EEXISTS:
                        response_handler(cookie, 16, "CHANNEL_EXISTS\r\n"); break;
                    case ENGINE_SUCCESS:
                        response_handler(cookie, 12, "SUBSCRIBED\r\n"); break;
                    default: break;
                }
            } else {
                rc = process_unsubscribe_command(cookie, argv[i].value, argv[i].length);
                switch (rc) {
                    case ENGINE_KEY_ENOENT:
                        response_handler(cookie, 11, "NOT_FOUND\r\n"); break;
                    case ENGINE_SUCCESS:
                        response_handler(cookie, 14, "UNSUBSCRIBED\r\n"); break;
                    default: break;
                }
            }
        }
        return true;
    }
    return false;
}

static void abort_command(const void *cmd_cookie, const void *cookie)
{

}

static void unsubscribe_all(const void *cookie, ENGINE_EVENT_TYPE type,
                            const void *event_data, const void *cb_data)
{
    for (int i = 0; i < chan_ht->capacity; i++) {
        HashTableEntry *entry = chan_ht->table[i];
        while (entry != NULL) {
            do_unsubscribe(cookie, entry);
            entry = entry->next;
        }
    }
}

MEMCACHED_PUBLIC_API
EXTENSION_ERROR_CODE memcached_extensions_initialize(const char *config,
                                                     GET_SERVER_API get_server_api) {
    server_api = get_server_api();
    if (server_api == NULL) {
        return EXTENSION_FATAL;
    }

    if (!server_api->extension->register_extension(EXTENSION_ASCII_PROTOCOL,
                                                   &publish_descriptor)) {
        return EXTENSION_FATAL;
    }

    if (!server_api->extension->register_extension(EXTENSION_ASCII_PROTOCOL,
                                                   &subscribe_descriptor)) {
        return EXTENSION_FATAL;
    }

    chan_ht = create_table(INITIAL_CAPACITY);
    clnt_ht = create_table(INITIAL_CAPACITY);

    if (chan_ht == NULL || clnt_ht == NULL) {
        if (chan_ht) free(chan_ht);
        if (clnt_ht) free(clnt_ht);
        return EXTENSION_FATAL;
    }

    server_api->callback->register_callback(NULL, ON_DISCONNECT, unsubscribe_all, NULL);

    return EXTENSION_SUCCESS;
}
