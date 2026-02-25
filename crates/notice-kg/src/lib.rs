use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An entity in the knowledge graph (a node).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String, // e.g., "topic", "language", "concept"
    pub weight: f64,         // how strongly this relates to the user
}

/// A relationship between entities (an edge).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_type: String, // e.g., "interested_in", "related_to"
    pub weight: f64,
}

/// In-memory knowledge graph for a single user.
/// Backed by petgraph for fast traversals.
/// Persisted to PostgreSQL (implementation comes with the schema).
pub struct UserKnowledgeGraph {
    pub user_id: Uuid,
    graph: DiGraph<Entity, Relationship>,
    /// Maps entity name → node index for quick lookup
    index: HashMap<String, NodeIndex>,
}

impl UserKnowledgeGraph {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    /// Add or reinforce an entity. If it exists, increment its weight.
    pub fn add_entity(&mut self, name: &str, entity_type: &str) -> NodeIndex {
        if let Some(&idx) = self.index.get(name) {
            // Entity exists — reinforce it
            if let Some(entity) = self.graph.node_weight_mut(idx) {
                entity.weight += 1.0;
            }
            idx
        } else {
            // New entity
            let entity = Entity {
                id: Uuid::new_v4(),
                name: name.to_string(),
                entity_type: entity_type.to_string(),
                weight: 1.0,
            };
            let idx = self.graph.add_node(entity);
            self.index.insert(name.to_string(), idx);
            idx
        }
    }

    /// Add a relationship between two entities.
    pub fn add_relationship(&mut self, from: NodeIndex, to: NodeIndex, rel_type: &str) {
        let relationship = Relationship {
            relationship_type: rel_type.to_string(),
            weight: 1.0,
        };
        self.graph.add_edge(from, to, relationship);
    }

    /// Get the top N entities by weight (user's strongest interests).
    pub fn top_interests(&self, n: usize) -> Vec<&Entity> {
        let mut entities: Vec<&Entity> = self.graph.node_weights().collect();
        entities.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        entities.truncate(n);
        entities
    }

    /// Get all entity names (useful for query context injection).
    pub fn context_terms(&self) -> Vec<String> {
        self.top_interests(10)
            .into_iter()
            .map(|e| e.name.clone())
            .collect()
    }
}
