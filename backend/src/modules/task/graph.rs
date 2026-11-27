//! 任务依赖图的环路检测。
//!
//! 边的方向是「后继 → 前置」（`task_id → depends_on_task_id`）。加入边 `a → b` 会成环
//! 当且仅当从 `b` 出发沿现有边能走回 `a`。这是纯函数，可脱离数据库单测。

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// 邻接表：节点 → 它的前置任务集合
pub type Graph = HashMap<Uuid, Vec<Uuid>>;

/// 由边列表构造邻接表
pub fn build_graph(edges: &[(Uuid, Uuid)]) -> Graph {
    let mut graph: Graph = HashMap::new();
    for (from, to) in edges {
        graph.entry(*from).or_default().push(*to);
    }
    graph
}

/// 判断在现有边上加入 `from → to` 是否形成环。
///
/// 自环（`from == to`）单独处理：它在依赖语义上是「任务依赖自己」，属非法但不算图论环，
/// 调用方需要区分错误信息，故此处只判真环。
pub fn would_cycle(graph: &Graph, from: Uuid, to: Uuid) -> bool {
    if from == to {
        return false;
    }

    // 从 to 出发做 BFS，看能否抵达 from
    let mut visited: HashSet<Uuid> = HashSet::new();
    let mut queue: VecDeque<Uuid> = VecDeque::new();
    queue.push_back(to);
    visited.insert(to);

    while let Some(node) = queue.pop_front() {
        if node == from {
            return true;
        }
        if let Some(next) = graph.get(&node) {
            for candidate in next {
                if visited.insert(*candidate) {
                    queue.push_back(*candidate);
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn direct_cycle_is_detected() {
        let a = id(1);
        let b = id(2);
        // 已有 a → b，再加 b → a 应成环
        let graph = build_graph(&[(a, b)]);
        assert!(would_cycle(&graph, b, a));
    }

    #[test]
    fn diamond_shape_is_acyclic_but_back_edge_is_not() {
        let (a, b, c, d, e) = (id(1), id(2), id(3), id(4), id(5));
        // a → b, a → c, b → d, c → d 是菱形依赖，本身无环
        let graph = build_graph(&[(a, b), (a, c), (b, d), (c, d)]);

        // 新增叶子边 d → e 不成环
        assert!(!would_cycle(&graph, d, e));
        // 但 d → a 会沿 a → b → d 回到 a，构成环
        assert!(would_cycle(&graph, d, a));
    }

    #[test]
    fn long_chain_is_detected() {
        let (a, b, c, d) = (id(1), id(2), id(3), id(4));
        // a → b → c → d，再加 d → a 应成环
        let graph = build_graph(&[(a, b), (b, c), (c, d)]);
        assert!(would_cycle(&graph, d, a));
    }

    #[test]
    fn self_loop_is_not_reported_as_cycle() {
        let a = id(1);
        let graph = build_graph(&[]);
        assert!(!would_cycle(&graph, a, a));
    }

    #[test]
    fn unrelated_edge_is_allowed() {
        let (a, b, c) = (id(1), id(2), id(3));
        let graph = build_graph(&[(a, b)]);
        assert!(!would_cycle(&graph, b, c));
    }
}
