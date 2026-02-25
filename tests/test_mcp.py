import json
import subprocess
import sys
import time
import os
import threading

def send_request(proc, method, params=None):
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    }
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    line = proc.stdout.readline()
    if not line:
        return None
    res = json.loads(line)
    print(f"DEBUG Response: {res}")
    return res

def log_stderr(proc):
    for line in proc.stderr:
        print(f"SERVER STDERR: {line.strip()}")

def test_mcp_lifecycle():
    location = "/tmp/paradox_test"
    os.makedirs(location, exist_ok=True)
    
    # Start the server with editing permitted
    proc = subprocess.Popen(
        ["paradox-mcp", "--location", location, "--permit-editing"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Start stderr logger thread
    stderr_thread = threading.Thread(target=log_stderr, args=(proc,), daemon=True)
    stderr_thread.start()

    try:
        print("Testing tools/list...")
        res = send_request(proc, "tools/list")
        tools = [t["name"] for t in res["result"]["tools"]]
        assert "create_table" in tools
        assert "insert_record" in tools
        
        table_name = "test_table"
        
        print(f"Testing create_table '{table_name}'...")
        fields = [
            {"name": "ID", "type": "LONG"},
            {"name": "Name", "type": "ALPHA", "length": 20},
            {"name": "Active", "type": "LOGICAL"}
        ]
        res = send_request(proc, "tools/call", {"name": "create_table", "arguments": {"table_name": table_name, "fields": fields}})
        assert "Successfully created" in res["result"]["content"][0]["text"]

        print("Testing list_tables...")
        res = send_request(proc, "tools/call", {"name": "list_tables"})
        assert table_name in res["result"]["content"][0]["text"]

        print("Testing insert_record...")
        record = {"ID": 1, "Name": "Alice", "Active": True}
        res = send_request(proc, "tools/call", {"name": "insert_record", "arguments": {"table_name": table_name, "record": record}})
        assert "Successfully inserted" in res["result"]["content"][0]["text"]

        print("Testing read_table_schema...")
        res = send_request(proc, "tools/call", {"name": "read_table_schema", "arguments": {"table_name": table_name}})
        schema_text = res["result"]["content"][1]["text"]
        assert "ALPHA" in schema_text
        assert "LONG" in schema_text

        print("Testing search_table...")
        res = send_request(proc, "tools/call", {"name": "search_table", "arguments": {"table_name": table_name, "query": {"Name": "Ali"}}})
        search_results = json.loads(res["result"]["content"][1]["text"])
        assert len(search_results) == 1
        assert search_results[0]["Name"].strip() == "Alice"

        print("Testing update_record...")
        update_data = {"Name": "Alicia"}
        res = send_request(proc, "tools/call", {"name": "update_record", "arguments": {"table_name": table_name, "index": 0, "record": update_data}})
        assert "Successfully updated" in res["result"]["content"][0]["text"]

        print("Testing read_table_data...")
        res = send_request(proc, "tools/call", {"name": "read_table_data", "arguments": {"table_name": table_name, "limit": 10}})
        records = json.loads(res["result"]["content"][1]["text"])
        assert len(records) == 1
        assert records[0]["Name"].strip() == "Alicia"

        print("\nAll tests passed successfully! ✅")

    finally:
        proc.terminate()
        # Clean up
        for f in os.listdir(location):
            os.remove(os.path.join(location, f))
        os.rmdir(location)

if __name__ == "__main__":
    test_mcp_lifecycle()
