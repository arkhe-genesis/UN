//! TreeManager — Gerenciador de árvore de agentes para detecção de dependências circulares
//! Selo: CATHEDRAL-ARKHE-TREE-v1.0.0-2026-06-21

use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Nó da árvore de agentes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub depth: usize,
    pub metadata: HashMap<String, String>,
}

/// Gerenciador de árvore de agentes — rastreia hierarquia e dependências
pub struct TreeManager {
    nodes: RwLock<HashMap<String, AgentNode>>,
    adjacency: RwLock<HashMap<String, Vec<String>>>,
}

impl TreeManager {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            adjacency: RwLock::new(HashMap::new()),
        }
    }

    /// Adiciona um nó à árvore, com opção de parent_id
    pub async fn add_node(&self, id: String, parent_id: Option<String>) {
        let mut nodes = self.nodes.write().await;
        let mut adjacency = self.adjacency.write().await;

        let depth = if let Some(parent) = &parent_id {
            nodes.get(parent).map(|n| n.depth + 1).unwrap_or(0)
        } else {
            0
        };

        let node = AgentNode {
            id: id.clone(),
            parent_id: parent_id.clone(),
            children: Vec::new(),
            depth,
            metadata: HashMap::new(),
        };
        nodes.insert(id.clone(), node);

        if let Some(parent) = parent_id {
            adjacency.entry(parent).or_default().push(id);
        }
    }

    /// Atualiza metadados de um nó
    pub async fn update_metadata(&self, agent_id: &str, key: String, value: String) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(agent_id) {
            node.metadata.insert(key, value);
        }
    }

    /// Obtém a profundidade de um agente
    pub async fn get_depth(&self, agent_id: &str) -> Option<usize> {
        self.nodes.read().await.get(agent_id).map(|n| n.depth)
    }

    /// Obtém a lista de filhos de um agente
    pub async fn get_children(&self, agent_id: &str) -> Vec<String> {
        self.adjacency.read().await.get(agent_id).cloned().unwrap_or_default()
    }

    /// Obtém a lista de ancestrais de um agente
    pub async fn get_ancestors(&self, agent_id: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = agent_id.to_string();
        let nodes = self.nodes.read().await;

        while let Some(node) = nodes.get(&current) {
            if let Some(parent) = &node.parent_id {
                ancestors.push(parent.clone());
                current = parent.clone();
            } else {
                break;
            }
        }
        ancestors
    }

    /// Detecta dependência circular a partir de um agente
    pub async fn detect_circular_dependency(&self, agent_id: &str) -> bool {
        let ancestors = self.get_ancestors(agent_id).await;
        ancestors.contains(&agent_id.to_string())
    }

    /// Detecta todas as dependências circulares na árvore
    pub async fn detect_all_circular_dependencies(&self) -> Vec<String> {
        let nodes = self.nodes.read().await;
        let mut circular = Vec::new();

        for id in nodes.keys() {
            if self.detect_circular_dependency(id).await {
                circular.push(id.clone());
            }
        }
        circular
    }

    /// Obtém o caminho completo de um agente até a raiz
    pub async fn get_path_to_root(&self, agent_id: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = agent_id.to_string();
        let nodes = self.nodes.read().await;

        while let Some(node) = nodes.get(&current) {
            path.push(current.clone());
            if let Some(parent) = &node.parent_id {
                current = parent.clone();
            } else {
                break;
            }
        }
        path
    }

    /// Obtém a subárvore enraizada em um agente
    pub async fn get_subtree(&self, agent_id: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(agent_id.to_string());

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            let children = self.get_children(&current).await;
            for child in children {
                queue.push_back(child);
            }
        }
        result
    }

    /// Obtém o número total de nós na árvore
    pub async fn total_nodes(&self) -> usize {
        self.nodes.read().await.len()
    }

    /// Obtém o nó raiz (agente sem parent)
    pub async fn get_root(&self) -> Option<String> {
        let nodes = self.nodes.read().await;
        nodes.iter()
            .find(|(_, node)| node.parent_id.is_none())
            .map(|(id, _)| id.clone())
    }

    /// Verifica se a árvore está vazia
    pub async fn is_empty(&self) -> bool {
        self.nodes.read().await.is_empty()
    }

    /// Limpa toda a árvore
    pub async fn clear(&self) {
        let mut nodes = self.nodes.write().await;
        let mut adjacency = self.adjacency.write().await;
        nodes.clear();
        adjacency.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tree_manager() {
        let tm = TreeManager::new();
        tm.add_node("root".to_string(), None).await;
        tm.add_node("child1".to_string(), Some("root".to_string())).await;
        tm.add_node("child2".to_string(), Some("root".to_string())).await;
        tm.add_node("grandchild".to_string(), Some("child1".to_string())).await;

        assert_eq!(tm.get_depth("root").await, Some(0));
        assert_eq!(tm.get_depth("grandchild").await, Some(2));
        assert_eq!(tm.get_children("root").await, vec!["child1", "child2"]);
        assert_eq!(tm.get_ancestors("grandchild").await, vec!["child1", "root"]);
        assert_eq!(tm.total_nodes().await, 4);
        assert_eq!(tm.get_root().await, Some("root".to_string()));
        assert!(!tm.detect_circular_dependency("root").await);
    }

    #[tokio::test]
    async fn test_circular_dependency() {
        let tm = TreeManager::new();
        tm.add_node("A".to_string(), None).await;
        tm.add_node("B".to_string(), Some("A".to_string())).await;
        tm.add_node("C".to_string(), Some("B".to_string())).await;

        // Cria ciclo: C -> B -> A -> C
        tm.add_node("A".to_string(), Some("C".to_string())).await;

        assert!(tm.detect_circular_dependency("C").await);
        assert!(tm.detect_all_circular_dependencies().await.contains(&"C".to_string()));
    }
}
