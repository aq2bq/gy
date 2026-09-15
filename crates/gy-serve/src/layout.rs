//! Laying out the graph (n-796b, d-b93b): a deterministic force layout per
//! scope, packed as bubbles.
use gy_ledger::{Node, NodeKind};
use serde::Serialize;
use std::collections::HashMap;

/// The force layout's weights, as the reference had them (template.html).
const STEPS: usize = 220;
const GOLDEN: f64 = 2.39996;
const SPRING: f64 = 26.0;
const SPRING_PULL: f64 = 0.05;
const STEP: f64 = 0.6;
const DECAY: f64 = 0.5;
const PULL: f64 = 0.01;
const PULL_GROWTH: f64 = 0.002;
const REPULSION: f64 = 900.0;
const FAR: f64 = 120.0;
const FAR_FACTOR: f64 = 0.15;
const RADIUS_PAD: f64 = 14.0;
const RADIUS_MIN: f64 = 40.0;
const RADIUS_KEEP: f64 = 0.96;
const BUBBLE_PAD: f64 = 30.0;
const BUBBLE_GAP: f64 = 26.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Placed {
    pub id: String,
    pub kind: NodeKind,
    pub scope: String,
    pub degree: usize,
    /// The state word the list view prints; `null` for a decision.
    pub state: Option<String>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bubble {
    pub scope: String,
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub count: usize,
}

/// The bubbles, the nodes in world coordinates, and the edges.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Graph {
    pub seq: u64,
    pub bubbles: Vec<Bubble>,
    pub nodes: Vec<Placed>,
    pub edges: Vec<(String, String)>,
    pub cross: Vec<(String, String)>,
}

/// Lay out every scope of `nodes` (d-b93b); only its stored edges count.
pub fn layout(nodes: &[Node], states: &HashMap<String, String>, seq: u64) -> Graph {
    let (bubbles, placed, edges) = assemble(nodes, states);
    Graph {
        seq,
        bubbles,
        nodes: placed,
        edges,
        cross: cross(nodes),
    }
}

/// Every scope's bubble, its nodes in world coordinates, and its edges.
fn assemble(
    nodes: &[Node],
    states: &HashMap<String, String>,
) -> (Vec<Bubble>, Vec<Placed>, Vec<(String, String)>) {
    let mut bubbles = Vec::new();
    let mut placed = Vec::new();
    let mut edges = Vec::new();
    for scope in scopes(nodes) {
        let members: Vec<&Node> = nodes.iter().filter(|node| node.scope() == scope).collect();
        let local = place(&members);
        let radius = local.radius + BUBBLE_PAD;
        let (x, y) = pack(radius, &bubbles);
        bubbles.push(Bubble {
            scope: scope.clone(),
            x,
            y,
            r: radius,
            count: members.len(),
        });
        for (at, node) in members.iter().enumerate() {
            placed.push(Placed {
                id: node.id().to_string(),
                kind: node.kind(),
                scope: scope.clone(),
                degree: local.degree[at],
                state: states.get(&node.id().to_string()).cloned(),
                x: x + local.pos[at].0,
                y: y + local.pos[at].1,
            });
        }
        for (from, to) in &local.edges {
            edges.push((
                members[*from].id().to_string(),
                members[*to].id().to_string(),
            ));
        }
    }
    (bubbles, placed, edges)
}

/// The scopes, the one with the most nodes first; ties by name, for order.
fn scopes(nodes: &[Node]) -> Vec<String> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for node in nodes {
        match counts.iter_mut().find(|(scope, _)| scope == node.scope()) {
            Some((_, count)) => *count += 1,
            None => counts.push((node.scope().to_string(), 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    counts.into_iter().map(|(scope, _)| scope).collect()
}

/// The edges whose ends sit in different scopes, one per stored edge.
fn cross(nodes: &[Node]) -> Vec<(String, String)> {
    let scopes: HashMap<String, &str> = nodes
        .iter()
        .map(|node| (node.id().to_string(), node.scope()))
        .collect();
    let mut cross = Vec::new();
    for node in nodes {
        for link in node.links() {
            let target = link.to.to_string();
            if scopes
                .get(&target)
                .is_some_and(|scope| *scope != node.scope())
            {
                cross.push((node.id().to_string(), target));
            }
        }
    }
    cross
}

/// One scope's positions, edges, degrees, and radius.
struct Local {
    pos: Vec<(f64, f64)>,
    edges: Vec<(usize, usize)>,
    degree: Vec<usize>,
    radius: f64,
}

fn place(members: &[&Node]) -> Local {
    let index: HashMap<String, usize> = members
        .iter()
        .enumerate()
        .map(|(at, node)| (node.id().to_string(), at))
        .collect();
    let mut pos: Vec<(f64, f64)> = (0..members.len())
        .map(|at| {
            let angle = at as f64 * GOLDEN;
            let reach = 6.0 * ((at + 1) as f64).sqrt();
            (angle.cos() * reach, angle.sin() * reach)
        })
        .collect();
    let mut speed = vec![(0.0, 0.0); members.len()];
    let mut edges = Vec::new();
    for (at, node) in members.iter().enumerate() {
        for link in node.links() {
            if let Some(&to) = index.get(&link.to.to_string()) {
                edges.push((at, to));
            }
        }
    }
    let mut degree = vec![0usize; members.len()];
    for (from, to) in &edges {
        degree[*from] += 1;
        degree[*to] += 1;
    }
    relax(&mut pos, &mut speed, &edges);
    let radius = fit(&mut pos);
    Local {
        pos,
        edges,
        degree,
        radius,
    }
}

/// The force loop: repulsion, springs, and a pull to the middle, cooling down.
fn relax(pos: &mut [(f64, f64)], speed: &mut [(f64, f64)], edges: &[(usize, usize)]) {
    for step in 0..STEPS {
        let temp = 1.0 - step as f64 / STEPS as f64;
        repel(pos, speed);
        springs(pos, speed, edges);
        settle(pos, speed, temp);
    }
}

/// Every pair pushes the other away; the push fades with the distance.
fn repel(pos: &[(f64, f64)], speed: &mut [(f64, f64)]) {
    for i in 0..pos.len() {
        for j in (i + 1)..pos.len() {
            let (dx, dy) = (pos[j].0 - pos[i].0, pos[j].1 - pos[i].1);
            let d2 = dx * dx + dy * dy + 0.01;
            let d = d2.sqrt();
            let push = REPULSION / d2 * if d < FAR { 1.0 } else { FAR_FACTOR };
            let (nx, ny) = (dx / d, dy / d);
            speed[i].0 -= nx * push;
            speed[i].1 -= ny * push;
            speed[j].0 += nx * push;
            speed[j].1 += ny * push;
        }
    }
}

/// An edge pulls its two ends toward the spring's natural length.
fn springs(pos: &[(f64, f64)], speed: &mut [(f64, f64)], edges: &[(usize, usize)]) {
    for (from, to) in edges {
        let (dx, dy) = (pos[*to].0 - pos[*from].0, pos[*to].1 - pos[*from].1);
        let d = (dx * dx + dy * dy).sqrt() + 0.01;
        let pull = (d - SPRING) * SPRING_PULL;
        let (nx, ny) = (dx / d, dy / d);
        speed[*from].0 += nx * pull;
        speed[*from].1 += ny * pull;
        speed[*to].0 -= nx * pull;
        speed[*to].1 -= ny * pull;
    }
}

/// Pull every point toward the middle, move it, and let its speed decay.
fn settle(pos: &mut [(f64, f64)], speed: &mut [(f64, f64)], temp: f64) {
    let center = PULL + PULL_GROWTH * (pos.len() as f64).sqrt();
    for at in 0..pos.len() {
        speed[at].0 -= pos[at].0 * center;
        speed[at].1 -= pos[at].1 * center;
        pos[at].0 += speed[at].0 * temp * STEP;
        pos[at].1 += speed[at].1 * temp * STEP;
        speed[at].0 *= DECAY;
        speed[at].1 *= DECAY;
    }
}

/// The radius that holds the layout; the points are pulled inside it.
fn fit(pos: &mut [(f64, f64)]) -> f64 {
    let mut distances: Vec<f64> = pos.iter().map(|(x, y)| (x * x + y * y).sqrt()).collect();
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let keep =
        ((distances.len() as f64 * RADIUS_KEEP) as usize).min(distances.len().saturating_sub(1));
    let radius = (distances.get(keep).copied().unwrap_or(0.0) + RADIUS_PAD).max(RADIUS_MIN);
    for point in pos.iter_mut() {
        let d = (point.0 * point.0 + point.1 * point.1).sqrt();
        if d > radius - 10.0 && d > 0.0 {
            let fit = (radius - 10.0) / d;
            point.0 *= fit;
            point.1 *= fit;
        }
    }
    radius
}

/// Where a bubble goes: the first takes the middle, the rest spiral out.
fn pack(radius: f64, bubbles: &[Bubble]) -> (f64, f64) {
    if bubbles.is_empty() {
        return (0.0, 0.0);
    }
    let (mut angle, mut distance): (f64, f64) = (0.0, 0.0);
    loop {
        distance += 4.0;
        angle += 0.35;
        let (x, y) = (angle.cos() * distance, angle.sin() * distance);
        let clear = bubbles.iter().all(|bubble| {
            let (dx, dy) = (bubble.x - x, bubble.y - y);
            (dx * dx + dy * dy).sqrt() > bubble.r + radius + BUBBLE_GAP
        });
        if clear {
            return (x, y);
        }
    }
}
