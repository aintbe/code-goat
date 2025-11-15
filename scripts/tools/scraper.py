import requests
from bs4 import BeautifulSoup
import os
import sys
import yaml


TEST_DIR = "/workspace/tests"
DOMAIN = "https://www.acmicpc.net"
HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
    "Referer": DOMAIN,
}


def scrape(problem_id: str):
    # 1. Fetch & parse target problem page
    try:
        response = requests.get(f"{DOMAIN}/problem/{problem_id}", headers=HEADERS)
        response.raise_for_status()  
        soup = BeautifulSoup(response.text, 'html.parser')
    except requests.exceptions.RequestException as e:
        print(f"❌ Failed to fetch: {e}")
        return

    # 2. Find problem info and build ResourceLimit object.
    try:
        config_table = soup.find(id="problem-info")
        td_list = config_table.find('tbody').find('tr').find_all('td')

        time_limit_td = td_list[0].get_text(strip=True)
        memory_limit_td = td_list[1].get_text(strip=True)
    except Exception as e:
        print(f"❌ Could not find problem info: {e}")
    
    config = {
        "limit": {
            "memory": 0,
            "cpu_time": 0,
            "real_time": 0,
            "stack": 0,
            "n_process": 0,
            "output": 0,
        }
    }

    if time_limit_td:
        # time_limit_td: "? 초"
        tokens = time_limit_td.split()
        if len(tokens) >= 2 and tokens[1] == "초":
            time_limit = int(float(tokens[0]) * 1000)
            config["limit"]["cpu_time"] = time_limit
            config["limit"]["real_time"] = time_limit
    
    if config["limit"]["real_time"] == 0:
        print(f"⚠️ Could not parse time limit, please update it manually.")
    
    if memory_limit_td:
        # time_limit_td: "? MB"
        tokens = memory_limit_td.split()
        if len(tokens) >= 2:
            unit = {
                "B": 1,
                "KB": 1 << 10,
                "MB": 1 << 10 << 10,
                "GB": 1 << 10 << 10 << 10,
            }.get(tokens[1], 0)
            config["limit"]["memory"] = int(float(tokens[0]) * unit)
    
    if config["limit"]["memory"] == 0:
        print(f"⚠️ Could not parse memory limit, please update it manually.")

    # 3. Check all public sample testcases.
    sample_tags = soup.find_all(class_="sampledata")
    tc_list = []
    for tag in sample_tags:
        key = tag.get("id", "")
        tokens = key.split('-')

        if len(tokens) >= 3:
            _, ext, num = tokens
            ext = ext[:-3] if ext.endswith("put") else ext

            tc_list.append({
                "name": f"{num}.{ext}",
                "content": tag.get_text()
            })
    
    if not tc_list:
        print(f"⚠️ No testcase is found; Is this expected?")

    # 4. Store parsed config and testcases.
    problem_dir = f"{TEST_DIR}/BOJ-{problem_id}"
    write(
        problem_dir,
        "config.yaml",
        yaml.dump(config, sort_keys=False)
    )

    for file in tc_list:
        write(
            f"{problem_dir}/testcases",
            file["name"],
            file["content"]
        )


def write(dir_path: str, file_name: str, content: str):
    if not os.path.exists(dir_path):
        os.makedirs(dir_path)

    try:
        file_path = os.path.join(dir_path, file_name)
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print(f"✅ Added {file_name}")
    except IOError as e:
        print(f"❌ Failed to create {file_name}: {e}")


if __name__ == "__main__":
    if len(sys.argv) <= 1:
        print("Usage: python3 boj.py [problem_id]")   
    else:
        scrape(sys.argv[1])