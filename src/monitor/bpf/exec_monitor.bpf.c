// SPDX-License-Identifier: GPL-2.0-only
// exec_monitor.bpf.c — eBPF program for tracing process execution
//
// Attaches to the sched:sched_process_exec tracepoint to capture all
// execve() and posix_spawn() calls from monitored processes.
//
// Compile with: clang -O2 -target bpf -c exec_monitor.bpf.c -o exec_monitor.bpf.o

#include <linux/bpf.h>
#include <linux/sched.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#define MAX_PATH_LEN 256
#define MAX_ARGS_LEN 512
#define EVENT_TYPE_EXEC 1

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
    __u64 arg_num;
};

// Hash map of PIDs to monitor (populated from userspace)
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

SEC("tracepoint/sched/sched_process_exec")
int trace_exec(struct trace_event_raw_sched_process_exec *ctx)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    __u32 *monitored = bpf_map_lookup_elem(&monitored_pids, &pid);

    // Skip if PID is not in the monitored set
    // (empty map = monitor nothing, not everything)
    if (!monitored)
        return 0;

    struct bpf_event event = {};
    event.event_type = EVENT_TYPE_EXEC;
    event.pid = pid;
    event.uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    event.timestamp_ns = bpf_ktime_get_ns();

    // Read the task struct for ppid
    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    event.ppid = BPF_CORE_READ(task, real_parent, tgid);

    // Read the executable filename from the tracepoint context
    // The filename is at a variable offset in the tracepoint data
    unsigned int fname_off = ctx->__data_loc_filename & 0xFFFF;
    bpf_probe_read_str(&event.path, sizeof(event.path),
                       (void *)ctx + fname_off);

    // Read first few arguments from /proc/self/cmdline equivalent
    // (args are on the user stack, accessed via current->mm->arg_start)
    __u64 arg_start = BPF_CORE_READ(task, mm, arg_start);
    __u64 arg_end = BPF_CORE_READ(task, mm, arg_end);
    __u64 arg_len = arg_end - arg_start;
    if (arg_len > MAX_ARGS_LEN)
        arg_len = MAX_ARGS_LEN;

    bpf_probe_read_user(&event.args, arg_len, (void *)arg_start);

    // Count environment variables
    __u64 env_start = BPF_CORE_READ(task, mm, env_start);
    __u64 env_end = BPF_CORE_READ(task, mm, env_end);
    event.arg_num = env_end - env_start; // Approximate env count

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));

    return 0;
}

char LICENSE[] SEC("license") = "GPL";
