"""为 M04 前端联调准备一份可重复使用的数据，并打印 token / project_id。

固定账号，重复运行只会追加新项目（列表按 updated_at 倒序，最新项目排第一）。
"""

from __future__ import annotations

import json
import sys
import urllib.error
import urllib.request

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8090/api/v1"
EMAIL = "ui-m04@example.com"
PASSWORD = "P@ssw0rd123"


def call(method: str, path: str, token: str | None = None, body: dict | None = None):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(f"{BASE}{path}", data=data, method=method)
    req.add_header("Content-Type", "application/json")
    if token:
        req.add_header("Authorization", f"Bearer {token}")

    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as exc:
        return exc.code, json.loads(exc.read().decode())


def data(payload):
    return payload["data"]


status, payload = call(
    "POST", "/auth/register", body={"email": EMAIL, "password": PASSWORD, "display_name": "联调账号"}
)
if status == 200:
    token = data(payload)["token"]
else:
    status, payload = call("POST", "/auth/login", body={"email": EMAIL, "password": PASSWORD})
    if status != 200:
        raise SystemExit(f"登录失败 {status} {payload}")
    token = data(payload)["token"]

project = data(
    call(
        "POST",
        "/projects",
        token,
        {"name": "2027 考研数学二", "project_type": "study", "importance_level": 5},
    )[1]
)
pid = project["id"]
call("POST", f"/projects/{pid}/activate", token)

main = data(call("POST", f"/projects/{pid}/goals", token, {"title": "数学二稳定 110 分", "goal_type": "performance"})[1])
data(
    call(
        "POST",
        f"/projects/{pid}/goals",
        token,
        {"title": "高数基础一轮吃透", "parent_goal_id": main["id"], "goal_type": "completion"},
    )[1]
)
data(
    call(
        "POST",
        f"/projects/{pid}/goals",
        token,
        {"title": "线代零基础过一遍", "parent_goal_id": main["id"], "goal_type": "completion"},
    )[1]
)

roadmap = data(call("POST", f"/projects/{pid}/roadmaps", token, {"source": "user", "change_summary": "基础→强化→冲刺三段式"})[1])
rid = roadmap["id"]

names = ["高数基础一轮", "高数强化", "真题冲刺"]
stages = []
for index, name in enumerate(names, start=1):
    stage = data(
        call(
            "POST",
            f"/projects/{pid}/roadmaps/{rid}/stages",
            token,
            {"name": name, "sequence_no": index, "description": f"{name}阶段的说明"},
        )[1]
    )
    stages.append(stage)

call(
    "PUT",
    f"/projects/{pid}/goals/{main['id']}/stages",
    token,
    {"links": [{"stage_id": stages[0]["id"], "contribution": "primary"}]},
)

milestone = data(
    call(
        "POST",
        f"/projects/{pid}/milestones",
        token,
        {"stage_id": stages[0]["id"], "title": "极限与连续全部通关"},
    )[1]
)
call("PATCH", f"/projects/{pid}/milestones/{milestone['id']}", token, {"status": "completed"})
data(
    call(
        "POST",
        f"/projects/{pid}/milestones",
        token,
        {"stage_id": stages[0]["id"], "title": "导数与微分"},
    )[1]
)

call("POST", f"/projects/{pid}/roadmaps/{rid}/confirm", token)
call("POST", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[0]['id']}/start", token)

tasks = [
    {"title": "做完极限 30 题", "stage_id": stages[0]["id"], "priority": 5, "estimated_minutes": 90},
    {"title": "整理错题本", "milestone_id": milestone["id"], "priority": 4},
    {"title": "背公式清单", "priority": 3, "due_at": "2026-12-05T12:00:00Z"},
    {"title": "复盘第一阶段", "priority": 2, "stage_id": stages[0]["id"]},
]
created = [data(call("POST", f"/projects/{pid}/tasks", token, body)[1]) for body in tasks]
call("POST", f"/projects/{pid}/tasks/{created[0]['id']}/dependencies", token, {"depends_on_task_id": created[1]["id"]})
call("PATCH", f"/projects/{pid}/tasks/{created[3]['id']}", token, {"status": "in_progress"})

print(json.dumps({"token": token, "project_id": pid}, ensure_ascii=False))
