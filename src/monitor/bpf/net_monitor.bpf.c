// SPDX-License-Identifier: GPL-2.0-only
// net_monitor.bpf.c — eBPF program for tracing network operations
//
// Attaches kprobes to TCP/IP functions to capture connect and bind
// operations from monitored processes.
//
// Compile with: clang -O2 -target bpf -c net_monitor.bpf.c -o net_monitor.bpf.o

#include <linux/bpf.h>
#include <linux/in.h>
#include <linux/in6.h>
#include <linux/socket.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#define MAX_PATH_LEN 256
#define MAX_ARGS_LEN 512
#define EVENT_TYPE_NET_CONNECT 6
#define EVENT_TYPE_NET_BIND    7

// Shared event structure (must match Rust RawBpfEvent)
struct bpf_event {
    __u32 event_type;
    __u32 pid;
    __u32 ppid;
    __u32 uid;
    __u64 timestamp_ns;
    char path[MAX_PATH_LEN];
    char path2[MAX_PATH_LEN];
    char args[MAX_ARGS_LEN];
    __u64 arg_num; // Lower 16 bits: port, bits 16-23: protocol
};

// Hash map of PIDs to monitor
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 4096);
    __type(key, __u32);
    __type(value, __u32);
} monitored_pids SEC(".maps");

// Perf event array for sending events to userspace
struct {
    __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
    __uint(key_size, sizeof(__u32));
    __uint(value_size, sizeof(__u32));
} events SEC(".maps");

static __always_inline int is_monitored(void)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    return bpf_map_lookup_elem(&monitored_pids, &pid) != NULL;
}

static __always_inline void fill_common(struct bpf_event *event, __u32 type)
{
    event->event_type = type;
    event->pid = bpf_get_current_pid_tgid() >> 32;
    event->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    event->timestamp_ns = bpf_ktime_get_ns();

    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    event->ppid = BPF_CORE_READ(task, real_parent, tgid);
}

// Helper to format IPv4 address as string
static __always_inline int format_ipv4(char *buf, int buf_len, __u32 addr)
{
    // addr is in network byte order
    unsigned char *bytes = (unsigned char *)&addr;
    int len = 0;

    // Simple formatting: write each octet
    // BPF verifier limits prevent using snprintf, so we write manually
    #pragma unroll
    for (int i = 0; i < 4; i++) {
        unsigned char b = bytes[i];
        if (b >= 100) {
            if (len < buf_len) buf[len++] = '0' + (b / 100);
            if (len < buf_len) buf[len++] = '0' + ((b / 10) % 10);
        } else if (b >= 10) {
            if (len < buf_len) buf[len++] = '0' + (b / 10);
        }
        if (len < buf_len) buf[len++] = '0' + (b % 10);
        if (i < 3 && len < buf_len) buf[len++] = '.';
    }
    if (len < buf_len) buf[len] = '\0';

    return len;
}

// kprobe: tcp_connect — intercept outgoing TCP connections
SEC("kprobe/tcp_connect")
int trace_tcp_connect(struct pt_regs *ctx)
{
    if (!is_monitored())
        return 0;

    struct bpf_event event = {};
    fill_common(&event, EVENT_TYPE_NET_CONNECT);

    // First argument is struct sock *sk
    struct sock *sk = (struct sock *)PT_REGS_PARM1(ctx);

    // Read destination address and port
    __u32 daddr = BPF_CORE_READ(sk, __sk_common.skc_daddr);
    __u16 dport = BPF_CORE_READ(sk, __sk_common.skc_dport);

    // Format IP address into args buffer
    format_ipv4(event.args, sizeof(event.args), daddr);

    // Pack port and protocol into arg_num
    // Lower 16 bits: port (convert from network byte order)
    // Bits 16-23: protocol (6 = TCP)
    __u16 port_host = __builtin_bswap16(dport);
    event.arg_num = (__u64)port_host | (6ULL << 16);

    // Read process executable path
    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    struct file *exe_file = BPF_CORE_READ(task, mm, exe_file);
    if (exe_file) {
        struct dentry *dentry = BPF_CORE_READ(exe_file, f_path.dentry);
        bpf_probe_read_kernel_str(&event.path, sizeof(event.path),
                                   BPF_CORE_READ(dentry, d_name.name));
    }

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));
    return 0;
}

// kprobe: inet_bind — intercept port binding
SEC("kprobe/inet_bind")
int trace_inet_bind(struct pt_regs *ctx)
{
    if (!is_monitored())
        return 0;

    struct bpf_event event = {};
    fill_common(&event, EVENT_TYPE_NET_BIND);

    // Second argument is struct sockaddr *uaddr
    struct sockaddr_in *addr = (struct sockaddr_in *)PT_REGS_PARM2(ctx);
    __u32 saddr = BPF_CORE_READ(addr, sin_addr.s_addr);
    __u16 sport = BPF_CORE_READ(addr, sin_port);

    format_ipv4(event.args, sizeof(event.args), saddr);

    __u16 port_host = __builtin_bswap16(sport);
    event.arg_num = (__u64)port_host;

    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    struct file *exe_file = BPF_CORE_READ(task, mm, exe_file);
    if (exe_file) {
        struct dentry *dentry = BPF_CORE_READ(exe_file, f_path.dentry);
        bpf_probe_read_kernel_str(&event.path, sizeof(event.path),
                                   BPF_CORE_READ(dentry, d_name.name));
    }

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
