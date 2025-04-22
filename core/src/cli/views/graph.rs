use crate::cli::utils;
use crate::commands;
use crate::shared::enums::workers::OSEnum;
use dialoguer::{theme::ColorfulTheme, Select};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Direction;

#[derive(Debug, Clone)]
enum Node {
    Core,
    User(String),   // username
    Job(String),    // job_name
    Worker(String), // label
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match self {
            Node::Core => "Core System".into(),
            Node::User(u) => format!("👤 User: {}", u),
            Node::Job(j) => format!("📦 Job: {}", j),
            Node::Worker(w) => format!("🛠️ Worker: {}", w),
        }
    }
}

pub async fn visualize() -> anyhow::Result<()> {
    let user_id: i32 = utils::select_user().await.unwrap();
    println!("\nFetching jobs and workers for user ID: {}\n", user_id);

    // === Graph Building Phase ===
    let mut graph = Graph::<Node, ()>::new();

    // Core node (only one)
    let core = graph.add_node(Node::Core);

    // --- Add user node ---
    let user = graph.add_node(Node::User(format!("{}", user_id)));
    graph.add_edge(user, core, ());

    // --- Fetch jobs for user ---
    let jobs = commands::get_jobs_for_user(user_id)
        .await
        .unwrap_or_default();
    let mut job_nodes = vec![];
    for job in jobs.iter() {
        let jnode = graph.add_node(Node::Job(job.job_name.clone()));
        graph.add_edge(user, jnode, ());
        job_nodes.push((job.id, jnode));
    }

    // --- Fetch workers for user ---
    let workers = commands::get_workers_for_user(user_id)
        .await
        .unwrap_or_default();
    let mut worker_nodes = vec![];
    for worker in workers.iter() {
        let wnode = graph.add_node(Node::Worker(worker.label.clone()));
        graph.add_edge(core, wnode, ());
        worker_nodes.push((worker.id, wnode));
    }

    // --- Fetch assignments ---
    let assignments = commands::get_assignments_for_user(user_id)
        .await
        .unwrap_or_default();
    for assignment in assignments {
        if let (Some((_, jnode)), Some((_, wnode))) = (
            job_nodes.iter().find(|(id, _)| *id == assignment.job_id),
            worker_nodes
                .iter()
                .find(|(id, _)| *id == assignment.worker_id),
        ) {
            graph.add_edge(*jnode, *wnode, ());
        }
    }

    // === Display Phase (basic text mode for now) ===
    println!("\n📊 Graph Structure:");
    for node_idx in graph.node_indices() {
        let node = &graph[node_idx];
        println!("- {}", node.to_string());
        for neighbor in graph.neighbors_directed(node_idx, Direction::Outgoing) {
            println!("    ↳ {}", graph[neighbor].to_string());
        }
    }

    // === Interactive Inspection ===
    let flat_nodes: Vec<(NodeIndex, String)> = graph
        .node_indices()
        .map(|n| (n, graph[n].to_string()))
        .collect();

    let mut names: Vec<String> = flat_nodes.iter().map(|(_, name)| name.clone()).collect();
    names.push("Exit".to_string());

    loop {
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Graph View - Select a node for details")
            .items(&names)
            .default(0)
            .interact()?;

        if names[choice] == "Exit" {
            break;
        }

        let selected_index = flat_nodes[choice].0;
        let selected_node = &graph[selected_index];

        match selected_node {
            Node::User(uid) => println!("\n👤 Full user details: {}\n", uid),
            Node::Job(job_name) => {
                if let Some(job) = jobs.iter().find(|j| j.job_name == *job_name) {
                    println!("\n📦 Job {}:\n{:#?}\n", job.id, job);
                }
            }
            Node::Worker(label) => {
                if let Some(worker) = workers.iter().find(|w| w.label == *label) {
                    println!("\n🛠️ Worker {}:\n{:#?}\n", worker.id, worker);
                }
            }
            Node::Core => println!("\n🧠 The Core handles orchestration and routing.\n"),
        }
    }

    Ok(())
}
