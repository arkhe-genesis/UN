pub mod cli;
pub mod error_handling;
pub mod event_bus;
pub mod evolution;
pub mod orchestrator;
pub mod skill;
pub mod swarm;
pub mod integrations;

// Mock modules needed to satisfy imports
pub mod hashtree_adapter;
pub mod trace_manager;
pub mod thread_index;
pub mod eve_client;
pub mod sandbox;
pub mod version_manager;
