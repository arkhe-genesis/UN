import { ActionProvider, WalletProvider, CreateAction } from "@coinbase/agentkit";
import { z } from "zod";
import { prove_memory_state } from "cathedral-napi"; // napi-rs binding

const PicoAdsSchema = z.object({
  query: z.string().describe("What the user/agent is looking for"),
  hub: z.string().optional().describe("Specific PicoAds hub"),
  maxResults: z.number().optional().default(5),
  requireMemoryProof: z.boolean().optional().default(true),
});

export class PicoAdsActionProvider extends ActionProvider<WalletProvider> {
  private apiKey: string;
  private backendUrl: string;

  constructor(apiKey: string, backendUrl = "http://localhost:8000") {
    super("picoads", []);
    this.apiKey = apiKey;
    this.backendUrl = backendUrl;
  }

  @CreateAction({
    name: "get_picoads_recommendations",
    description: "Get personalized recommendations from PicoAds. Optionally requires a DLA MemoryProof.",
    schema: PicoAdsSchema,
  })
  async getRecommendations(args: z.infer<typeof PicoAdsSchema>): Promise<string> {
    let memoryCommitment: string | null = null;

    // Generate memory proof if required
    if (args.requireMemoryProof) {
      try {
        const proof = await prove_memory_state(); // napi-rs call
        memoryCommitment = proof.merkle_root;
        console.log(`[MemoryProof] Generated: ${memoryCommitment}`);
      } catch (err) {
        console.warn("[MemoryProof] Failed to generate proof");
        if (args.requireMemoryProof) {
          return JSON.stringify({ success: false, error: "Memory proof required but generation failed" });
        }
      }
    }

    // Call FastAPI backend
    const headers: Record<string, string> = {
      "Authorization": `Bearer ${this.apiKey}`,
      "Content-Type": "application/json",
    };
    if (memoryCommitment) {
      headers["X-Memory-Commitment"] = memoryCommitment;
    }

    const body = {
      query: args.query,
      hub: args.hub,
      max_results: args.maxResults,
      require_memory_proof: args.requireMemoryProof,
    };

    try {
      const response = await fetch(`${this.backendUrl}/picoads/recommendations`, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        const errorText = await response.text();
        return JSON.stringify({ success: false, error: `HTTP ${response.status}: ${errorText}` });
      }

      const data = await response.json();
      return JSON.stringify({
        success: true,
        recommendations: data.recommendations,
        memoryProofUsed: !!memoryCommitment,
        memoryCommitment,
      });
    } catch (err: any) {
      return JSON.stringify({ success: false, error: err.message });
    }
  }

  supportsNetwork = () => true;
}

export const picoAdsActionProvider = (apiKey: string, backendUrl?: string) =>
  new PicoAdsActionProvider(apiKey, backendUrl);
