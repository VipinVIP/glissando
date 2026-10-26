// LD_PRELOAD guard used by tests/privacy/check.sh: records and refuses every
// attempt to create a non-local socket (anything but AF_UNIX / AF_NETLINK)
// and every name resolution. The log path comes from NETGUARD_LOG.
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <unistd.h>

static void note(const char *what, int v) {
    const char *p = getenv("NETGUARD_LOG");
    FILE *f = p ? fopen(p, "a") : stderr;
    if (!f) return;
    fprintf(f, "pid %d: %s %d\n", (int)getpid(), what, v);
    if (f != stderr) fclose(f);
}

int socket(int domain, int type, int protocol) {
    static int (*real)(int, int, int);
    if (!real) real = dlsym(RTLD_NEXT, "socket");
    if (domain != AF_UNIX && domain != AF_NETLINK) {
        note("socket domain", domain);
        errno = EACCES;
        return -1;
    }
    return real(domain, type, protocol);
}

int getaddrinfo(const char *node, const char *service, const struct addrinfo *hints, struct addrinfo **res) {
    (void)node; (void)service; (void)hints; (void)res;
    note("getaddrinfo", 0);
    return EAI_FAIL;
}

struct hostent *gethostbyname(const char *name) {
    (void)name;
    note("gethostbyname", 0);
    return NULL;
}
