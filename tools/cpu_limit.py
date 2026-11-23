"""Keep development checks and their child processes within four logical CPUs."""
import os
import subprocess
import sys

MAX_CPUS = 4


def limit_cpus(count=MAX_CPUS):
    deployment = (os.environ.get("GITHUB_ACTIONS") == "true"
                  and os.environ.get("NTESIM_BUILD_CPUS") == "all")
    if sys.platform == "win32":
        import ctypes
        from ctypes import wintypes

        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.GetCurrentProcess.restype = wintypes.HANDLE
        kernel.GetProcessAffinityMask.argtypes = [
            wintypes.HANDLE, ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_size_t)]
        kernel.SetProcessAffinityMask.argtypes = [wintypes.HANDLE, ctypes.c_size_t]
        process = kernel.GetCurrentProcess()
        current, system = ctypes.c_size_t(), ctypes.c_size_t()
        if not kernel.GetProcessAffinityMask(process, ctypes.byref(current), ctypes.byref(system)):
            raise ctypes.WinError(ctypes.get_last_error())
        cpus = [i for i in range(ctypes.sizeof(current) * 8) if current.value & (1 << i)]
        if not deployment:
            cpus = cpus[:count]
        if not deployment and not kernel.SetProcessAffinityMask(process, sum(1 << i for i in cpus)):
            raise ctypes.WinError(ctypes.get_last_error())
    elif hasattr(os, "sched_setaffinity"):
        cpus = sorted(os.sched_getaffinity(0))
        if not deployment:
            cpus = cpus[:count]
            os.sched_setaffinity(0, cpus)
    else:
        available = os.cpu_count() or 1
        cpus = list(range(available if deployment else min(count, available)))
    if deployment:
        return cpus  # Preserve caller limits; impose neither affinity nor pool defaults.
    for key in ("CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "RAYON_NUM_THREADS", "OMP_NUM_THREADS"):
        try:
            value = int(os.environ.get(key, count))
        except ValueError:
            value = count
        os.environ[key] = str(min(count, value) if value > 0 else count)
    return cpus


def main():
    limit_cpus()
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise SystemExit("cpu_limit.py requires a command after --")
    return subprocess.call(command)


if __name__ == "__main__":
    raise SystemExit(main())
