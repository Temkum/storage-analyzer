#!/usr/bin/env bash
# Phase 6 — privileged eBPF validation reference
# (run: sudo bash docs/research/poc_bpf_validate.sh)
#
# WHY THIS EXISTS
#   The /proc polling path (phase6-linux-attribution.md §1.1) gives exact TCP
#   attribution for same-uid sockets only. UDP byte counters and foreign-uid sockets
#   are NOT visible without kernel instrumentation. This script proves the eBPF path
#   that WOULD fill that gap — so the decision to defer it is informed, and a future
#   LinuxEbpfApplicationNetworkProvider has a known-good probe set.
#
# REQUIRES: root / CAP_BPF. (kernel.unprivileged_bpf_disabled=2 on this host demonstrates
#   the v1 design constraint: eBPF is denied for the unprivileged desktop sidecar.)
#   Tools: bpftrace. Kernel has BTF at /sys/kernel/btf/vmlinux.
set -euo pipefail

echo "=== eBPF attribution validation (requires root / CAP_BPF) ==="

if [[ ${EUID} -ne 0 ]]; then
    echo "SKIP: not root (uid=${EUID}). This demonstrates the v1 design constraint:"
    echo "      eBPF is denied for the unprivileged desktop sidecar. Re-run with sudo."
    exit 127
fi

for t in bpftrace; do
    command -v "$t" >/dev/null 2>&1 || { echo "MISSING: $t"; exit 2; }
done
[[ -r /sys/kernel/btf/vmlinux ]] || { echo "MISSING: /sys/kernel/btf/vmlinux"; exit 3; }
echo "kernel BTF: OK; $(bpftrace --version | head -1)"

echo "--- kprobe symbols present: ---"
grep -Ew 'tcp_sendmsg_locked|tcp_recvmsg|udp_sendmsg|udpv6_sendmsg' /proc/kallsyms \
    | awk '{print $3, $1}' | sort -u

gen_traffic() {
    timeout 1 curl -s https://example.com >/dev/null 2>&1 || true
    for i in 1 2 3 4 5; do
        dig @1.1.1.1 +short +time=1 +tries=1 example.com >/dev/null 2>&1 || true
    done
}

cleanup() { [[ -n "${TBP:-}" ]] && kill "$TBP" 2>/dev/null || true; wait "${TBP:-}" 2>/dev/null || true; }
trap cleanup EXIT

echo "--- tracing kprobes while generating TCP+UDP traffic (4s) ---"
timeout 5 bpftrace -e '
kprobe:tcp_sendmsg_locked { @tcp_tx[comm]   += arg->len; }
kprobe:tcp_recvmsg         { @tcp_rx[comm]  += 1;         }
kprobe:udp_sendmsg         { @udp_tx[comm]  += arg->len; }
kprobe:udpv6_sendmsg       { @udpv6_tx[comm]+= arg->len; }
interval:s:4
{
    printf("tcp_send_bytes:   "); for (k,v in @tcp_tx)  printf("%s=%d ", k,v);  printf("\n");
    printf("tcp_recv_samples: "); for (k,v in @tcp_rx)  printf("%s=%d ", k,v);  printf("\n");
    printf("udp_send_bytes:   "); for (k,v in @udp_tx)  printf("%s=%d ", k,v);  printf("\n");
    printf("udpv6_send_bytes: "); for (k,v in @udpv6_tx)printf("%s=%d ", k,v);  printf("\n");
    if (@tcp_tx || @udp_tx)
        printf("VERDICT: eBPF byte-accounting IS possible (with privileges)\n");
    else
        printf("VERDICT: no events captured\n");
    clear(@tcp_tx); clear(@tcp_rx); clear(@udp_tx); clear(@udpv6_tx);
    exit(0);
}' &
TBP=$!
gen_traffic
wait "$TBP" 2>/dev/null || true

echo
echo "=== Conclusion ==="
echo "Nonzero tcp_send_bytes / udp_send_bytes attributed to curl/dig proves that a"
echo "root-mode LinuxEbpfApplicationNetworkProvider could fill the UDP + foreign-uid gap."
echo "v1 does NOT take this path (the sidecar runs unprivileged as uid 1000 with CapEff=0)."
