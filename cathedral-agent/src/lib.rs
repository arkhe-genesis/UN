pub mod cli;
pub mod error_handling;
pub mod event_bus;
pub mod evolution;
pub mod integrations;
pub mod orchestrator;
pub mod skill;
pub mod swarm;

// Mock modules needed to satisfy imports
pub mod eve_client;
pub mod hashtree_adapter;
pub mod sandbox;
pub mod thread_index;
pub mod trace_manager;
pub mod version_manager;

pub mod fastbrain;
pub mod wormgraph_arweave;
