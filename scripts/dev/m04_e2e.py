"""M04 端到端联调脚本：真实 HTTP 打本地后端，覆盖目标/路线/阶段/里程碑/任务/依赖。

用法：python scripts/dev/m04_e2e.py [base_url]
不落库断言，纯走接口；失败即抛出并打印响应体。
"""

from __future__ import annotations

import json
import sys
import urllib.error
import urllib.request
import uuid

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8090/api/v1"


def call(method: str, path: str, token: str | None = None, body: dict | None = None):
    url = f"{BASE}{path}"
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Content-Type", "application/json")
    if token:
        req.add_header("Authorization", f"Bearer {token}")

    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            payload = json.loads(resp.read().decode())
            return resp.status, payload
    except urllib.error.HTTPError as exc:
        payload = json.loads(exc.read().decode())
        return exc.code, payload


def ok(method: str, path: str, token: str | None = None, body: dict | None = None):
    status, payload = call(method, path, token, body)
    if status != 200:
        raise SystemExit(f"[FAIL] {method} {path} -> {status} {payload}")
    return payload["data"]


def expect(status_want: int, method: str, path: str, token: str | None = None, body=None):
    status, payload = call(method, path, token, body)
    if status != status_want:
        raise SystemExit(f"[FAIL] {method} {path} 期望 {status_want} 实际 {status} {payload}")
    return payload


suffix = uuid.uuid4().hex[:8]
email = f"m04-{suffix}@example.com"

auth = ok("POST", "/auth/register", body={"email": email, "password": "P@ssw0rd123", "display_name": "M04 联调"})
token = auth["token"]
print(f"✓ 注册并登录 {email}")

project = ok("POST", "/projects", token, {"name": f"M04 项目 {suffix}", "project_type": "study", "importance_level": 4})
pid = project["id"]
ok("POST", f"/projects/{pid}/activate", token)
print(f"✓ 项目 {pid} 已创建并激活")

main_goal = ok("POST", f"/projects/{pid}/goals", token, {"title": "数学二 110 分", "goal_type": "performance"})
child_goal = ok(
    "POST",
    f"/projects/{pid}/goals",
    token,
    {"title": "高数基础一轮", "parent_goal_id": main_goal["id"], "goal_type": "completion"},
)
expect(422, "POST", f"/projects/{pid}/goals", token, {"title": "第三层", "parent_goal_id": child_goal["id"]})
print("✓ 目标：主目标 + 子目标，三层被拒 422")

roadmap = ok("POST", f"/projects/{pid}/roadmaps", token, {"source": "user", "change_summary": "首版"})
rid = roadmap["id"]

stages = []
for index, name in enumerate(["基础", "强化", "冲刺"], start=1):
    stage = ok("POST", f"/projects/{pid}/roadmaps/{rid}/stages", token, {"name": name, "sequence_no": index})
    stages.append(stage)
print(f"✓ 路线 v{roadmap['version']} 下创建 {len(stages)} 个阶段")

ok("PATCH", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[0]['id']}", token, {"description": "教材 + 课后题"})

links = ok(
    "PUT",
    f"/projects/{pid}/goals/{child_goal['id']}/stages",
    token,
    {"links": [{"stage_id": stages[0]["id"], "contribution": "primary"}]},
)
print(f"✓ 目标关联阶段：{len(links['items'])} 条")

# draft 路线不允许推进阶段状态
expect(409, "POST", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[0]['id']}/start", token)
ok("POST", f"/projects/{pid}/roadmaps/{rid}/confirm", token)
print("✓ 草稿路线拒绝阶段动作 409，确认后生效")

started = ok("POST", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[0]['id']}/start", token)
assert started["status"] == "active" and started["started_at"], started
# pending 不能直达 completed：第二个阶段直接 complete 应 409
expect(409, "POST", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[1]['id']}/complete", token)
completed = ok("POST", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[0]['id']}/complete", token)
assert completed["status"] == "completed" and completed["completion_method"] == "user", completed
# 同态幂等：再次 complete 返回 200 且不刷新 completed_at
again = ok("POST", f"/projects/{pid}/roadmaps/{rid}/stages/{stages[0]['id']}/complete", token)
assert again["completed_at"] == completed["completed_at"], (completed, again)
print("✓ 阶段四动作：start / complete / 否定边 409 / 同态幂等不刷新时间戳")

milestone = ok(
    "POST",
    f"/projects/{pid}/milestones",
    token,
    {"stage_id": stages[0]["id"], "title": "极限与连续完成"},
)
done_milestone = ok("PATCH", f"/projects/{pid}/milestones/{milestone['id']}", token, {"status": "completed"})
assert done_milestone["completed_at"], done_milestone
expect(409, "PATCH", f"/projects/{pid}/milestones/{milestone['id']}", token, {"status": "skipped"})
print("✓ 里程碑：completed 后写 completed_at，终态直转 409")

task_a = ok(
    "POST",
    f"/projects/{pid}/tasks",
    token,
    {"title": "做 30 道极限题", "stage_id": stages[0]["id"], "priority": 4, "estimated_minutes": 60},
)
task_b = ok("POST", f"/projects/{pid}/tasks", token, {"title": "整理错题", "milestone_id": milestone["id"]})
expect(
    422,
    "POST",
    f"/projects/{pid}/tasks/{task_a['id']}/dependencies",
    token,
    {"depends_on_task_id": task_a["id"]},
)
ok("POST", f"/projects/{pid}/tasks/{task_a['id']}/dependencies", token, {"depends_on_task_id": task_b["id"]})
deps = ok("GET", f"/projects/{pid}/tasks/{task_a['id']}/dependencies", token)
assert len(deps["items"]) == 1, deps
# 反向依赖会成环
expect(
    422,
    "POST",
    f"/projects/{pid}/tasks/{task_b['id']}/dependencies",
    token,
    {"depends_on_task_id": task_a["id"]},
)
print("✓ 依赖：自依赖 422、正常建立、反向成环 422")

listed = ok("GET", f"/projects/{pid}/tasks?page=1&page_size=20", token)
assert listed["total"] == 2, listed
direct = ok("PATCH", f"/projects/{pid}/tasks/{task_a['id']}", token, {"status": "completed"})
assert direct["completed_at"], direct
print("✓ 任务：todo 直达 completed 允许，completed_at 写入")

ok("DELETE", f"/projects/{pid}/tasks/{task_a['id']}", token)
after_delete = ok("GET", f"/projects/{pid}/tasks?page=1&page_size=20", token)
assert after_delete["total"] == 1, after_delete
expect(404, "GET", f"/projects/{pid}/tasks/{task_a['id']}", token)
print("✓ 软删除：列表与详情都不再返回")

# 越权：另一个账号访问同一项目一律 404
other = ok(
    "POST",
    "/auth/register",
    body={"email": f"m04-other-{suffix}@example.com", "password": "P@ssw0rd123", "display_name": "他人"},
)
expect(404, "GET", f"/projects/{pid}/goals", other["token"])
expect(404, "GET", f"/projects/{pid}/roadmaps", other["token"])
expect(404, "GET", f"/projects/{pid}/tasks", other["token"])
print("✓ 归属隔离：他人访问目标/路线/任务均为 404")

print("\n全部通过")
