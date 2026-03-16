import {
  chatRequestSchema,
  chatResponseSchema,
  healthSnapshotSchema,
  type ChatRequest,
  type ChatResponse,
  type HealthSnapshot,
} from "@adaptive/shared-types";

type OperatorSdkOptions = {
  orchestratorBaseUrl?: string;
};

export class OperatorSdk {
  private orchestratorBaseUrl: string;

  constructor(options: OperatorSdkOptions = {}) {
    this.orchestratorBaseUrl =
      options.orchestratorBaseUrl ?? "http://127.0.0.1:7001";
  }

  async chat(payload: ChatRequest): Promise<ChatResponse> {
    const request = chatRequestSchema.parse(payload);
    const response = await fetch(`${this.orchestratorBaseUrl}/v1/chat/request`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`chat request failed with ${response.status}`);
    }

    const body = await response.json();
    return chatResponseSchema.parse(body);
  }

  async health(): Promise<HealthSnapshot> {
    const response = await fetch(`${this.orchestratorBaseUrl}/health`);
    if (!response.ok) {
      throw new Error(`health request failed with ${response.status}`);
    }

    const body = await response.json();
    return healthSnapshotSchema.parse(body);
  }
}

