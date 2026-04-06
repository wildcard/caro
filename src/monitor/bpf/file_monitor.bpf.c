// SPDX-License-Identifier: GPL-2.0-only
// file_monitor.bpf.c — eBPF program for tracing file operations
//
// Attaches kprobes to VFS functions to capture file open, write, delete,
// and rename operations from monitored processes.
//
// Compile with: clang -O2 -target bpf -c file_monitor.bpf.c -o file_monitor.bpf.o

#include <linux/bpf.h>
#include <linux/fs.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#define MAX_PATH_LEN 256
#define MAX_ARGS_LEN 512
#define EVENT_TYPE_FILE_OPEN   2
#define EVENT_TYPE_FILE_WRITE  3
#define EVENT_TYPE_FILE_DELETE 4
#define EVENT_TYPE_FILE_RENAME 5

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

// Hash map of PIDs to monitor (shared with exec_monitor)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 4096);
    __type(key, __u32);
    __type(value, __u32);
} monitored_pids SEC(".maps");

// Perf event array for sending events to userspace (shared)
struct {
    __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
    __uint(key_size, sizeof(__u32));
    __uint(value_size, sizeof(__u32));
} events SEC(".maps");

// Helper to check if current process is monitored
static __always_inline int is_monitored(void)
{
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    return bpf_map_lookup_elem(&monitored_pids, &pid) != NULL;
}

// Helper to fill common event fields
static __always_inline void fill_common(struct bpf_event *event, __u32 type)
{
    event->event_type = type;
    event->pid = bpf_get_current_pid_tgid() >> 32;
    event->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    event->timestamp_ns = bpf_ktime_get_ns();

    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    event->ppid = BPF_CORE_READ(task, real_parent, tgid);
}

// kprobe: vfs_open — intercept file open operations
SEC("kprobe/vfs_open")
int trace_vfs_open(struct pt_regs *ctx)
{
    if (!is_monitored())
        return 0;

    struct bpf_event event = {};
    fill_common(&event, EVENT_TYPE_FILE_OPEN);

    // First argument to vfs_open is struct path *
    struct path *p = (struct path *)PT_REGS_PARM1(ctx);
    struct dentry *dentry = BPF_CORE_READ(p, dentry);

    // Read the filename from dentry
    bpf_probe_read_kernel_str(&event.path, sizeof(event.path),
                               BPF_CORE_READ(dentry, d_name.name));

    // Open flags are in the file struct (second parameter)
    // For simplicity, we capture the flags from the second arg
    event.arg_num = PT_REGS_PARM2(ctx); // open flags

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));
    return 0;
}

// kprobe: vfs_write — intercept file write operations
SEC("kprobe/vfs_write")
int trace_vfs_write(struct pt_regs *ctx)
{
    if (!is_monitored())
        return 0;

    struct bpf_event event = {};
    fill_common(&event, EVENT_TYPE_FILE_WRITE);

    // First argument is struct file *
    struct file *f = (struct file *)PT_REGS_PARM1(ctx);
    struct dentry *dentry = BPF_CORE_READ(f, f_path.dentry);

    bpf_probe_read_kernel_str(&event.path, sizeof(event.path),
                               BPF_CORE_READ(dentry, d_name.name));

    // Third argument is count (bytes to write)
    event.arg_num = PT_REGS_PARM3(ctx);

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));
    return 0;
}

// kprobe: vfs_unlink — intercept file deletion
SEC("kprobe/vfs_unlink")
int trace_vfs_unlink(struct pt_regs *ctx)
{
    if (!is_monitored())
        return 0;

    struct bpf_event event = {};
    fill_common(&event, EVENT_TYPE_FILE_DELETE);

    // Arguments depend on kernel version:
    // Older: vfs_unlink(struct inode *dir, struct dentry *dentry)
    // Newer: vfs_unlink(struct user_namespace *mnt_userns, struct inode *dir, struct dentry *dentry)
    // We use the last dentry parameter
    struct dentry *dentry = (struct dentry *)PT_REGS_PARM2(ctx);

    bpf_probe_read_kernel_str(&event.path, sizeof(event.path),
                               BPF_CORE_READ(dentry, d_name.name));

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));
    return 0;
}

// kprobe: vfs_rename — intercept file rename/move
SEC("kprobe/vfs_rename")
int trace_vfs_rename(struct pt_regs *ctx)
{
    if (!is_monitored())
        return 0;

    struct bpf_event event = {};
    fill_common(&event, EVENT_TYPE_FILE_RENAME);

    // vfs_rename(struct renamedata *rd) on newer kernels
    // We read source and destination dentry names
    // For simplicity, using the older API signature
    struct dentry *old_dentry = (struct dentry *)PT_REGS_PARM2(ctx);
    struct dentry *new_dentry = (struct dentry *)PT_REGS_PARM4(ctx);

    bpf_probe_read_kernel_str(&event.path, sizeof(event.path),
                               BPF_CORE_READ(old_dentry, d_name.name));
    bpf_probe_read_kernel_str(&event.path2, sizeof(event.path2),
                               BPF_CORE_READ(new_dentry, d_name.name));

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU,
                          &event, sizeof(event));
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
