import React, { useEffect, useState } from 'react';
import useWebSocket from 'react-use-websocket';
import { toast } from 'sonner';

interface Proposal {
    id: string;
    title: string;
    description: string;
    risk_level: string;
    validation_status: string;
}

const WS_URL = 'ws://localhost:8080/ws';

export function SelfImproveView() {
    const [proposals, setProposals] = useState<Proposal[]>([]);
    const [filter, setFilter] = useState({ risk: '', status: '' });
    const [tab, setTab] = useState<'monorepo' | 'proposals'>('proposals');

    const { lastJsonMessage } = useWebSocket(WS_URL, {
        onMessage: (event) => {
            const data = JSON.parse(event.data);
            toast.success(`Nova proposta: ${data.title}`, { duration: 5000 });
            setProposals(prev => [data, ...prev]);
        },
    });

    const fetchProposals = async () => {
        const params = new URLSearchParams(filter);
        const res = await fetch(`/api/proposals?${params}`);
        if(res.ok) {
            setProposals(await res.json());
        }
    };

    useEffect(() => { fetchProposals(); }, [filter]);

    return (
        <div>
            <h1>Self-Improvement</h1>
            <div className="tabs">
                <button onClick={() => setTab('proposals')}>Propostas</button>
                <button onClick={() => setTab('monorepo')}>Monorepo</button>
            </div>

            {tab === 'proposals' && (
                <div>
                    <div className="filters">
                        <select onChange={(e) => setFilter({ ...filter, risk: e.target.value })}>
                            <option value="">Todos os Riscos</option>
                            <option value="Low">Baixo</option>
                            <option value="Medium">Médio</option>
                            <option value="High">Alto</option>
                            <option value="Critical">Crítico</option>
                        </select>
                        <select onChange={(e) => setFilter({ ...filter, status: e.target.value })}>
                            <option value="">Todos os Status</option>
                            <option value="Pending">Pending</option>
                            <option value="Approved">Approved</option>
                        </select>
                    </div>
                    <ul>
                        {proposals.map(p => (
                            <li key={p.id}>
                                <strong>{p.title}</strong> ({p.risk_level}) - {p.validation_status}
                                <p>{p.description}</p>
                            </li>
                        ))}
                    </ul>
                </div>
            )}
            {tab === 'monorepo' && (
                <div>
                    <h2>Análise do Monorepo</h2>
                    <p>Aqui você pode ver uma visão geral da saúde arquitetural do código.</p>
                </div>
            )}
        </div>
    );
}
