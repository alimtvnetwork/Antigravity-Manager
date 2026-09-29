import sys
import os
import time
import datetime
import subprocess

def get_pid_file(log_path):
    return log_path + ".pid"

def run_loop(prompt_id, instance_id, log_path, interval_sec):
    log_abs = os.path.abspath(log_path)
    os.makedirs(os.path.dirname(log_abs), exist_ok=True)
    pid = os.getpid()
    pid_file = get_pid_file(log_abs)
    with open(pid_file, "w", encoding="utf-8") as pf:
        pf.write(str(pid))
    
    iteration = 1
    # If log file exists, detect existing iteration
    if os.path.exists(log_path):
        try:
            with open(log_path, "r", encoding="utf-8") as lf:
                lines = [l.strip() for l in lf.readlines() if l.strip()]
                for l in reversed(lines):
                    if "Iteration: " in l:
                        part = l.split("Iteration: ")[1].split(" ")[0]
                        iteration = int(part) + 1
                        break
        except Exception:
            iteration = 1

    try:
        while True:
            utc_now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
            entry = f"[{utc_now}] [PID: {pid}] Instance: {instance_id} | Prompt: {prompt_id} | Iteration: {iteration} | Status: RUNNING | Goal: Long-running task active\n"
            with open(log_path, "a", encoding="utf-8") as f:
                f.write(entry)
                f.flush()
            iteration += 1
            time.sleep(interval_sec)
    except KeyboardInterrupt:
        pass
    finally:
        if os.path.exists(pid_file):
            try:
                os.remove(pid_file)
            except Exception:
                pass

def main():
    if len(sys.argv) < 2:
        print("Usage: prompt_heartbeat_runner.py <start|run|check|stop|latest> ...")
        sys.exit(1)
        
    action = sys.argv[1].lower()
    
    if action == "run":
        # Internal loop runner
        prompt_id = sys.argv[2]
        instance_id = sys.argv[3]
        log_path = sys.argv[4]
        interval = float(sys.argv[5]) if len(sys.argv) > 5 else 5.0
        run_loop(prompt_id, instance_id, log_path, interval)
        
    elif action == "start":
        prompt_id = sys.argv[2]
        instance_id = sys.argv[3]
        log_path = os.path.abspath(sys.argv[4])
        interval = sys.argv[5] if len(sys.argv) > 5 else "5.0"
        
        # Stop any existing runner for this log_path
        pid_file = get_pid_file(log_path)
        if os.path.exists(pid_file):
            try:
                with open(pid_file, "r") as pf:
                    old_pid = int(pf.read().strip())
                if sys.platform == "win32":
                    subprocess.run(["taskkill", "/F", "/PID", str(old_pid)], capture_output=True)
                else:
                    os.kill(old_pid, 9)
            except Exception:
                pass
            if os.path.exists(pid_file):
                try:
                    os.remove(pid_file)
                except Exception:
                    pass
                    
        # Launch detached background process
        flags = 0
        if sys.platform == "win32":
            flags = subprocess.CREATE_NEW_PROCESS_GROUP | 0x08000000  # DETACHED_PROCESS
            
        proc = subprocess.Popen(
            [sys.executable, os.path.abspath(__file__), "run", prompt_id, instance_id, log_path, interval],
            creationflags=flags,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            close_fds=True
        )
        print(f"STARTED:{proc.pid}")
        
    elif action == "check":
        log_path = os.path.abspath(sys.argv[2])
        pid_file = get_pid_file(log_path)
        is_running = False
        active_pid = None
        if os.path.exists(pid_file):
            try:
                with open(pid_file, "r") as pf:
                    active_pid = int(pf.read().strip())
                # Check if process is alive
                if sys.platform == "win32":
                    out = subprocess.run(["tasklist", "/FI", f"PID eq {active_pid}"], capture_output=True, text=True)
                    is_running = str(active_pid) in out.stdout
                else:
                    os.kill(active_pid, 0)
                    is_running = True
            except Exception:
                is_running = False
                
        last_entry = ""
        total_entries = 0
        last_mtime = 0
        if os.path.exists(log_path):
            last_mtime = os.path.getmtime(log_path)
            with open(log_path, "r", encoding="utf-8") as f:
                lines = [l.strip() for l in f.readlines() if l.strip()]
                total_entries = len(lines)
                if lines:
                    last_entry = lines[-1]
                    
        age_sec = time.time() - last_mtime if last_mtime > 0 else 999999
        is_fresh = age_sec <= 10.0 and is_running
        print(f"RUNNING={is_running}|FRESH={is_fresh}|PID={active_pid}|TOTAL={total_entries}|AGE={age_sec:.1f}|LAST={last_entry}")
        
    elif action == "stop":
        log_path = os.path.abspath(sys.argv[2])
        pid_file = get_pid_file(log_path)
        stopped = False
        if os.path.exists(pid_file):
            try:
                with open(pid_file, "r") as pf:
                    target_pid = int(pf.read().strip())
                if sys.platform == "win32":
                    subprocess.run(["taskkill", "/F", "/PID", str(target_pid)], capture_output=True)
                else:
                    os.kill(target_pid, 9)
                stopped = True
            except Exception:
                pass
            if os.path.exists(pid_file):
                try:
                    os.remove(pid_file)
                except Exception:
                    pass
        print(f"STOPPED={stopped}")
        
    elif action == "latest":
        log_path = os.path.abspath(sys.argv[2])
        if os.path.exists(log_path):
            with open(log_path, "r", encoding="utf-8") as f:
                lines = [l.strip() for l in f.readlines() if l.strip()]
                if lines:
                    print(lines[-1])
                else:
                    print("EMPTY")
        else:
            print("NOT_FOUND")

if __name__ == "__main__":
    main()
