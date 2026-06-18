use clap::{Subcommand};

#[derive(Debug, Subcommand)]
pub enum DeSciCommand {
    /// Publica um Research Object no DeSci
    #[command(name = "desci-publish")]
    Publish {
        /// Caminho para o Research Object
        #[arg(short, long)]
        path: String,
        /// Título da publicação
        #[arg(short, long)]
        title: String,
        /// Autores (npub)
        #[arg(short, long)]
        authors: String,
    },

    /// Realiza revisão por pares de um Research Object
    #[command(name = "desci-review")]
    Review {
        /// O dPID do objeto a ser revisado
        #[arg(short, long)]
        dpid: String,
    },

    /// Mostra o perfil DeSci
    #[command(name = "desci-profile")]
    Profile,

    /// Evolui um nó DeSci
    #[command(name = "desci-evolve")]
    Evolve {
        /// O ID do node DeSci para evoluir
        #[arg(short, long)]
        node_id: String,
        /// A métrica alvo a otimizar
        #[arg(short, long)]
        target_metric: String,
    },
}
